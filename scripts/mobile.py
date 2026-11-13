#!/usr/bin/env python3
"""Project-local Android development; never resets an AVD or a Nextcloud instance."""
import argparse
import hashlib
import json
import os
import resource
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


def bootstrap(include_emulator=True):
    BASE.mkdir(parents=True, exist_ok=True)
    BASE.chmod(0o700)
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
    devices = subprocess.check_output([str(ADB), '-P', str(CONFIG['adb_server_port']), 'devices'], env=environment(), text=True)
    if SERIAL not in devices:
        with (BASE / 'emulator.log').open('ab') as log:
            subprocess.Popen([str(SDK / 'emulator/emulator'), '-avd', CONFIG['emulator_avd'],
                '-port', str(CONFIG['emulator_console_port']), '-accel', 'on', '-gpu', 'host' if host_gpu else 'swiftshader', '-feature', '-Vulkan,-GuestAngle',
                '-memory', '3072', '-cores', '4', '-no-audio', '-no-boot-anim', '-no-metrics', '-no-snapshot-save', *(['-no-window'] if headless else [])],
                env=environment(), stdout=log, stderr=subprocess.STDOUT, start_new_session=True,
                preexec_fn=lambda: resource.setrlimit(resource.RLIMIT_CORE, (0, 0)))
    print('Dedicated emulator:', SERIAL, '— wait for sys.boot_completed=1 using status')


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
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['bootstrap', 'doctor', 'emulator', 'status', 'build', 'test', 'integration', 'install', 'browser', 'flutter'])
    parser.add_argument('arguments', nargs=argparse.REMAINDER)
    args = parser.parse_args()
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
        adb('install', '-r', APP / 'build/app/outputs/flutter-apk/app-x86_64-debug.apk')
        adb('shell', 'am', 'start', '-n', 'org.mealshop.pocket.dev/.MainActivity')
    else:
        signing_key()
        if args.action == 'integration':
            prepare_emulator()
        command = {
            'doctor': ['doctor', '-v'], 'build': ['build', 'apk', '--debug', '--split-per-abi', '--target=lib/main.dart'],
            'test': ['test'], 'integration': ['drive', '--driver=test_driver/integration_test.dart', '--target=integration_test/shopping_smoke_test.dart', '-d', SERIAL, '--no-dds'],
            'flutter': [],
        }[args.action] + args.arguments
        run([FLUTTER / 'bin/flutter', *command], cwd=APP)
        if args.action == 'build':
            commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
            for apk in (APP / 'build/app/outputs/flutter-apk').glob('app-*-debug.apk'):
                with apk.open('rb') as stream:
                    digest = hashlib.file_digest(stream, 'sha256').hexdigest()
                apk.with_suffix('.apk.sha256').write_text(digest + '  ' + apk.name + '\n')
                apk.with_suffix('.apk.build.json').write_text(json.dumps({'commit': commit, 'flutter': CONFIG['flutter_version'], 'application_id': 'org.mealshop.pocket.dev', 'sha256': digest, 'dirty': bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip())}, indent=2) + '\n')


if __name__ == '__main__':
    main()
