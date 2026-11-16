#!/usr/bin/env python3
"""Project-local Android development; never resets an AVD or a Nextcloud instance."""
import argparse
import hashlib
import json
import os
import resource
import shutil
import tempfile
import time
from pathlib import Path
import subprocess
import tarfile
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / 'var/mobile'
APP = ROOT / 'clients/flutter'
CONFIG = json.loads((APP / 'toolchain.json').read_text())
SDK = BASE / 'android-sdk'
USER = BASE / 'android-home'
FLUTTER = BASE / 'flutter'
ADB = SDK / 'platform-tools/adb'
SERIAL = 'emulator-' + str(CONFIG['emulator_console_port'])


def environment():
    if not BASE.resolve().is_relative_to(ROOT / 'var'):
        raise SystemExit('Refusing a mobile tool directory outside the repository var directory')
    env = os.environ | {
        'ANDROID_HOME': str(SDK), 'ANDROID_SDK_ROOT': str(SDK),
        'ANDROID_USER_HOME': str(USER), 'ANDROID_EMULATOR_HOME': str(USER),
        'ANDROID_AVD_HOME': str(USER / 'avd'),
        'ANDROID_ADB_SERVER_PORT': str(CONFIG['adb_server_port']),
        'ADB_SERVER_SOCKET': 'tcp:127.0.0.1:' + str(CONFIG['adb_server_port']),
        'PUB_CACHE': str(BASE / 'pub-cache'), 'GRADLE_USER_HOME': str(BASE / 'gradle'),
        'DASH__SUPPRESS_ANALYTICS': 'true',
    }
    info = BASE / 'jdk-info.json'
    if info.exists():
        env['JAVA_HOME'] = json.loads(info.read_text())['home']
        env['PATH'] = env['JAVA_HOME'] + '/bin:' + env['PATH']
    env['PATH'] = str(FLUTTER / 'bin') + ':' + env['PATH']
    return env


def run(args, **kwargs):
    return subprocess.run([str(a) for a in args], env=environment(), check=True, **kwargs)


def download(url, sha256):
    target = BASE / url.rsplit('/', 1)[-1]
    if not target.exists():
        urllib.request.urlretrieve(url, target)
    with target.open('rb') as stream:
        actual = hashlib.file_digest(stream, 'sha256').hexdigest()
    if actual != sha256:
        raise SystemExit('Official archive checksum mismatch; refusing to execute it')
    return target


def ocr_models():
    manifest = json.loads((APP / 'ocr-models.json').read_text())
    libraries = json.loads((APP / 'capture-libraries.json').read_text())
    libroot = BASE / 'capture-libs'
    libroot.mkdir(parents=True, exist_ok=True)
    for filename, library in libraries.items():
        path = libroot / filename
        if not path.exists():
            path.write_bytes(download(library['url'], library['sha256']).read_bytes())
        with path.open('rb') as stream:
            if hashlib.file_digest(stream, 'sha256').hexdigest() != library['sha256']:
                raise SystemExit('Capture library checksum mismatch; refusing this build')
    target = APP / 'android/app/src/main/assets/tessdata'
    target.mkdir(parents=True, exist_ok=True)
    for language, model in manifest['models'].items():
        path = target / (language + '.traineddata')
        if not path.exists():
            path.write_bytes(download(model['url'], model['sha256']).read_bytes())
        with path.open('rb') as stream:
            if hashlib.file_digest(stream, 'sha256').hexdigest() != model['sha256']:
                raise SystemExit('OCR model checksum mismatch; refusing this build')


def bootstrap(include_emulator=True):
    BASE.mkdir(parents=True, exist_ok=True)
    BASE.chmod(0o700)
    ocr_models()
    USER.mkdir(exist_ok=True)
    tools = SDK / 'cmdline-tools/latest'
    if not (tools / 'bin/android').exists():
        archive = download(**CONFIG['command_line_tools'])
        with zipfile.ZipFile(archive) as bundle:
            for member in bundle.infolist():
                parts = Path(member.filename).parts
                if not parts or parts[0] != 'cmdline-tools' or '..' in parts:
                    raise SystemExit('Unexpected archive path')
                if member.is_dir():
                    continue
                target = tools.joinpath(*parts[1:])
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(bundle.read(member))
                if parts[1] == 'bin':
                    target.chmod(0o755)
    if not (BASE / 'jdk-info.json').exists():
        archive = download(CONFIG['jdk']['url'], CONFIG['jdk']['sha256'])
        with tarfile.open(archive) as bundle:
            name = bundle.getmembers()[0].name.split('/')[0]
            bundle.extractall(BASE / 'jdk', filter='data')
        (BASE / 'jdk-info.json').write_text(json.dumps({'home': str(BASE / 'jdk' / name)}))
    if not FLUTTER.exists():
        run(['git', 'init', FLUTTER], stdout=subprocess.DEVNULL)
        run(['git', '-C', FLUTTER, 'remote', 'add', 'origin', 'https://github.com/flutter/flutter.git'])
        run(['git', '-C', FLUTTER, 'fetch', '--depth=1', 'origin', 'tag', CONFIG['flutter_version']])
        run(['git', '-C', FLUTTER, 'checkout', '--detach', 'FETCH_HEAD'])
    check_sdk()
    run([FLUTTER / 'bin/flutter', '--disable-analytics'])
    packages = ['platform-tools', 'platforms/android-36',
                'build-tools/36.0.0', 'ndk/28.2.13676358', 'cmake/3.22.1']
    if include_emulator:
        packages += ['emulator', f"system-images/android-{CONFIG['emulator_api']}/google_apis/x86_64"]
    run([tools / 'bin/sdkmanager', '--sdk_root=' + str(SDK), '--licenses'],
        input='y\n' * 100, text=True)
    run([tools / 'bin/android', '--no-metrics', '--sdk=' + str(SDK), 'sdk', 'install', *packages])
    run([FLUTTER / 'bin/flutter', 'pub', 'get', '--enforce-lockfile'], cwd=APP)
    run(['rustup','toolchain','install',CONFIG['rust_version'],'--profile','minimal',
         '--target','x86_64-linux-android,aarch64-linux-android,armv7-linux-androideabi'])
    # Fetch locked sources during setup; licence verification/builds stay offline-capable.
    run(['cargo','+'+CONFIG['rust_version'],'fetch','--locked'],cwd=ROOT)


def check_sdk():
    commit = subprocess.check_output(['git', '-C', str(FLUTTER), 'rev-parse', 'HEAD'], text=True).strip()
    if commit != CONFIG['flutter_commit']:
        raise SystemExit('Flutter revision differs from clients/flutter/toolchain.json')


def adb(*args, **kwargs):
    return run([ADB, '-P', CONFIG['adb_server_port'], '-s', SERIAL, *args], **kwargs)


def prepare_emulator():
    name = subprocess.check_output(
        [str(ADB), '-P', str(CONFIG['adb_server_port']), '-s', SERIAL, 'emu', 'avd', 'name'],
        env=environment(), text=True, timeout=15).splitlines()[0].strip()
    if name != CONFIG['emulator_avd']:
        raise SystemExit('Refusing to change a device outside the dedicated test emulator')
    adb('emu', 'power', 'ac', 'on')
    adb('shell', 'settings', 'put', 'global', 'stay_on_while_plugged_in', '3')
    adb('shell', 'input', 'keyevent', '224')
    adb('shell', 'wm', 'dismiss-keyguard')
    adb('reverse', 'tcp:8080', 'tcp:8080')


def emulator(headless=False, host_gpu=False):
    if not os.access('/dev/kvm', os.R_OK | os.W_OK):
        raise SystemExit('KVM access is required; enable virtualization and grant this user /dev/kvm access')
    avd = USER / 'avd' / (CONFIG['emulator_avd'] + '.avd')
    if not (avd / 'config.ini').exists():
        run([SDK / 'cmdline-tools/latest/bin/avdmanager', 'create', 'avd', '-n', CONFIG['emulator_avd'],
             '-k', f"system-images;android-{CONFIG['emulator_api']};google_apis;x86_64", '-d', 'pixel_6', '-p', avd], input='no\n', text=True)
    # Android must advertise a physical keyboard for desktop typing to reach text fields.
    config = avd / 'config.ini'
    values = config.read_text().splitlines()
    values = [line for line in values if not line.startswith('hw.keyboard=')]
    if CONFIG['emulator_avd'] == 'Mealshop_AppOnly_API36':
        values = [line for line in values if not line.startswith('disk.dataPartition.size=')]
        values.append('disk.dataPartition.size=2G')
    config.write_text('\n'.join([*values, 'hw.keyboard=yes']) + '\n')
    devices = subprocess.check_output([str(ADB), '-P', str(CONFIG['adb_server_port']), 'devices'], env=environment(), text=True)
    if SERIAL not in devices:
        with (BASE / 'emulator.log').open('ab') as log:
            subprocess.Popen([str(SDK / 'emulator/emulator'), '-avd', CONFIG['emulator_avd'],
                '-port', str(CONFIG['emulator_console_port']), '-accel', 'on', '-gpu', 'host' if host_gpu else 'swiftshader', '-feature', '-Vulkan,-GuestAngle',
                '-memory', '2048' if CONFIG['emulator_avd'] == 'Mealshop_AppOnly_API36' else '3072',
                '-cores', '2' if CONFIG['emulator_avd'] == 'Mealshop_AppOnly_API36' else '4', '-no-audio', '-no-boot-anim', '-no-metrics', '-crash-report-mode', 'disabled', '-no-snapshot-load', '-no-snapshot-save', *(['-no-window'] if headless else [])],
                env=environment(), stdout=log, stderr=subprocess.STDOUT, start_new_session=True,
                preexec_fn=lambda: resource.setrlimit(resource.RLIMIT_CORE, (0, 0)))
    print('Dedicated emulator:', SERIAL, '— wait for sys.boot_completed=1 using status')


def apk_identity(apk):
    return subprocess.check_output(
        [str(SDK / 'cmdline-tools/latest/bin/apkanalyzer'), 'manifest', 'application-id', str(apk)],
        env=environment(), text=True, stderr=subprocess.DEVNULL, timeout=30).strip()


def verify_integration_apk(apk):
    if apk_identity(apk) != 'org.mealshop.pocket.verification':
        raise SystemExit('Refusing integration cleanup of any package except org.mealshop.pocket.verification')


def integration(arguments):
    global SERIAL
    local = "--local" in arguments
    host_gpu = "--host-gpu" in arguments
    arguments = [value for value in arguments if value not in ("--local","--host-gpu")]
    target = "app_only_test.dart" if local else "shopping_smoke_test.dart"
    if any(value.startswith(('--use-application-binary', '--target', '--driver', '--device-id', '-d', '-t')) for value in arguments):
        raise SystemExit('Integration target, device, driver and verified binary cannot be overridden')
    os.environ['MEALSHOP_ANDROID_VERIFICATION'] = '1'
    if local:
        # A separate clean document provider avoids exposing retained files from the daily AVD.
        CONFIG['emulator_avd'] = 'Mealshop_AppOnly_API36'
        CONFIG['emulator_console_port'] = 5558
        SERIAL = 'emulator-5558'
    devices = subprocess.check_output([str(ADB), '-P', str(CONFIG['adb_server_port']), 'devices'], env=environment(), text=True)
    if SERIAL in devices:
        prepare_emulator()
        adb('emu', 'kill')
        for _ in range(30):
            devices = subprocess.check_output([str(ADB), '-P', str(CONFIG['adb_server_port']), 'devices'], env=environment(), text=True)
            if SERIAL not in devices: break
            time.sleep(1)
    from native_core import build as build_core
    build_core(['x86_64'])
    with tempfile.TemporaryDirectory(prefix='verification-', dir=BASE) as folder:
        upgrade = Path(folder) / 'normal-verification.apk'
        if local:
            run([FLUTTER / 'bin/flutter', 'build', 'apk', '--debug', '--target=lib/main.dart',
                 '--target-platform=android-x64'], cwd=APP)
            shutil.copyfile(APP / 'build/app/outputs/flutter-apk/app-debug.apk', upgrade)
            verify_integration_apk(upgrade)
        run([FLUTTER / 'bin/flutter', 'build', 'apk', '--debug', '--target=integration_test/' + target,
             '--target-platform=android-x64'], cwd=APP)
        binary = Path(folder) / 'verification.apk'
        shutil.copyfile(APP / 'build/app/outputs/flutter-apk/app-debug.apk', binary)
        verify_integration_apk(binary)
        run([APP / 'android/gradlew', '--stop'], cwd=APP / 'android')
        emulator(headless=True, host_gpu=host_gpu)
        for _ in range(120):
            ready = subprocess.run([str(ADB), '-P', str(CONFIG['adb_server_port']), '-s', SERIAL,
                                    'shell', 'getprop', 'sys.boot_completed'],
                                   env=environment(), capture_output=True, text=True, timeout=15)
            if ready.stdout.strip() == '1': break
            time.sleep(2)
        else: raise SystemExit('Verification emulator failed to boot')
        prepare_emulator()
        if local:
            from app_only_verification import drive
            drive(__import__(__name__), binary, upgrade, arguments)
        else:
            run([FLUTTER / 'bin/flutter', 'drive', '--driver=test_driver/integration_test.dart',
                 '--target=integration_test/' + target, '-d', SERIAL, '--no-dds',
                 '--use-application-binary=' + str(binary), *arguments], cwd=APP)


def signing_key():
    key = BASE / 'development.keystore'
    if not key.exists():
        run([Path(environment()['JAVA_HOME']) / 'bin/keytool', '-genkeypair', '-keystore', key,
             '-storepass', 'android', '-alias', 'androiddebugkey', '-keypass', 'android', '-keyalg', 'RSA',
             '-keysize', '2048', '-validity', '10000', '-dname', 'CN=Mealshop Local Development,O=Mealshop,C=DE'],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        key.chmod(0o600)
    os.environ['MEALSHOP_ANDROID_KEYSTORE'] = str(key)


def main():
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['bootstrap', 'doctor', 'emulator', 'status', 'build', 'test', 'integration', 'install', 'browser', 'flutter'])
    parser.add_argument('arguments', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.action in ('build', 'integration', 'test'):
        ocr_models()
    if args.action == 'bootstrap':
        bootstrap(include_emulator="--build-only" not in args.arguments)
        return
    check_sdk()
    if args.action == 'emulator':
        emulator(headless="--headless" in args.arguments, host_gpu="--host-gpu" in args.arguments)
    elif args.action == 'browser':
        prepare_emulator()
        browser=CONFIG['emulator_browser']
        adb('shell', 'pm', 'path', browser['package'])
        adb('shell','cmd','role','add-role-holder','android.app.role.BROWSER',browser['package'])
    elif args.action == 'status':
        adb('shell', 'getprop', 'sys.boot_completed')
        adb('emu', 'avd', 'name')
    elif args.action == 'install':
        prepare_emulator()
        apk = APP / 'build/app/outputs/flutter-apk/app-x86_64-debug.apk'
        if apk_identity(apk) != 'org.mealshop.pocket.dev':
            raise SystemExit('Refusing an upgrade APK with a different application ID')
        adb('install', '-r', '--no-streaming', apk)
        adb('shell', 'am', 'start', '-n', 'org.mealshop.pocket.dev/.MainActivity')
    else:
        signing_key()
        if args.action == 'integration':
            integration(args.arguments)
            return
        if args.action == 'build':
            from native_core import build as build_core, TARGETS
            build_core(list(TARGETS))
        command = {
            'doctor': ['doctor', '-v'], 'build': ['build', 'apk', '--debug', '--split-per-abi', '--target=lib/main.dart'],
            'test': ['test'],
            'flutter': [],
        }[args.action] + args.arguments
        run([FLUTTER / 'bin/flutter', *command], cwd=APP)
        if args.action == 'build':
            commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
            for apk in (APP / 'build/app/outputs/flutter-apk').glob('app-*-debug.apk'):
                with apk.open('rb') as stream:
                    digest = hashlib.file_digest(stream, 'sha256').hexdigest()
                apk.with_suffix('.apk.sha256').write_text(digest + '  ' + apk.name + '\n')
                versions = {field.replace('-', '_'): subprocess.check_output(
                    [str(SDK / 'cmdline-tools/latest/bin/apkanalyzer'), 'manifest', field, str(apk)],
                    env=environment(), text=True, stderr=subprocess.DEVNULL).strip()
                    for field in ['application-id', 'version-name', 'version-code']}
                if versions['application_id'] != 'org.mealshop.pocket.dev':
                    raise SystemExit('Refusing to publish development APK metadata for a different package')
                from native_core import verify
                abi = apk.name.removeprefix('app-').removesuffix('-debug.apk')
                verify([abi])
                core = json.loads((APP/'android/app/src/main/assets/mealshop-core-build.json').read_text())[abi]
                with zipfile.ZipFile(apk) as archive:
                    if hashlib.sha256(archive.read('lib/'+abi+'/libmealshop_core.so')).hexdigest() != core['binary_sha256']:
                        raise SystemExit('Packaged native core differs from its verified build')
                apk.with_suffix('.apk.build.json').write_text(json.dumps({'commit': commit, 'flutter': CONFIG['flutter_version'], **versions, 'sha256': digest, 'core':core, 'dirty': bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip())}, indent=2) + '\n')


if __name__ == '__main__':
    main()
