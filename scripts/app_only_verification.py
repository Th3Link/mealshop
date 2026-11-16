"""Drive synthetic app-only verification in its own AVD and package, including Android documents."""
from pathlib import Path
import re
import subprocess
import tempfile
import threading
import time
import xml.etree.ElementTree as ET

PACKAGE = 'org.mealshop.pocket.verification'


def drive(mobile, binary, upgrade, arguments):
    if mobile.SERIAL != 'emulator-5558' or mobile.CONFIG['emulator_avd'] != 'Mealshop_AppOnly_API36':
        raise SystemExit('App-only verification requires its separate synthetic AVD')
    mobile.verify_integration_apk(binary)
    mobile.verify_integration_apk(upgrade)
    mobile.prepare_emulator()
    installed = subprocess.run([str(mobile.ADB), '-P', str(mobile.CONFIG['adb_server_port']), '-s', mobile.SERIAL,
                                'shell', 'pm', 'path', PACKAGE], env=mobile.environment(),capture_output=True,text=True,timeout=20)
    if installed.stdout.startswith('package:'): mobile.adb('uninstall', PACKAGE)

    def adb(*args, raw=False):
        return subprocess.check_output([str(mobile.ADB), '-P', str(mobile.CONFIG['adb_server_port']),
                                        '-s', mobile.SERIAL, *args], env=mobile.environment(),
                                       text=not raw, timeout=30)

    def screen():
        adb('shell','rm','-f','/sdcard/mealshop-verification-window.xml')
        adb('shell', 'uiautomator', 'dump', '/sdcard/mealshop-verification-window.xml')
        try:
            return ET.fromstring(adb('shell', 'cat', '/sdcard/mealshop-verification-window.xml'))
        except (subprocess.CalledProcessError,ET.ParseError):
            return ET.Element('hierarchy')

    def tap(node):
        bounds = [int(n) for n in re.findall(r'\d+', node.attrib['bounds'])]
        adb('shell', 'input', 'tap', str((bounds[0]+bounds[2])//2), str((bounds[1]+bounds[3])//2))

    def document(label):
        navigation = False
        for _ in range(20):
            tree = screen()
            nodes = [node for node in tree.iter('node')
                     if node.attrib.get('package') in ('com.google.android.documentsui', 'com.android.documentsui')]
            selected = next((n for n in nodes if n.attrib.get('text', '').lower() == label.lower()
                             and n.attrib.get('enabled') == 'true'), None)
            if selected is not None:
                tap(selected)
                return
            downloads = next((n for n in nodes if n.attrib.get('text') == 'Downloads'), None)
            if downloads is not None and not navigation:
                tap(downloads); navigation = True
            elif not navigation:
                button = next((n for n in nodes if n.attrib.get('content-desc') in ('Show roots', 'Show navigation drawer')), None)
                if button is not None: tap(button)
            time.sleep(.3)
        raise RuntimeError('Expected synthetic document action was not available: ' + label)

    def shot(name):
        target = mobile.BASE/'app-only-evidence'/name
        target.parent.mkdir(exist_ok=True)
        target.write_bytes(adb('exec-out', 'screencap', '-p', raw=True))

    command = [mobile.FLUTTER/'bin/flutter', 'drive', '--driver=test_driver/integration_test.dart',
               '--target=integration_test/app_only_test.dart', '-d', mobile.SERIAL, '--no-dds',
               '--use-application-binary=' + str(binary), '--keep-app-running', *arguments]
    process = subprocess.Popen([str(a) for a in command], cwd=mobile.APP, env=mobile.environment(),
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1)
    timeout = threading.Timer(600, process.terminate)
    timeout.start()
    filename = None
    phases = set()
    try:
        for line in process.stdout:
            print(line, end='', flush=True)
            match = re.search(r'MEALSHOP_VERIFY_(EXPORT|IMPORT|DAMAGED|CRASH|SCREEN)(?: ([0-9a-f]+))?', line)
            if not match: continue
            phase, value = match.groups()
            phases.add(phase)
            if phase == 'EXPORT':
                if value is None or not re.fullmatch('[0-9a-f]{32}', value): raise RuntimeError('Invalid synthetic export id')
                filename = 'mealshop-backup-' + value[:8] + '.age'
                document('Save')
            elif phase == 'IMPORT':
                if filename is None: raise RuntimeError('No completed synthetic backup')
                document(filename)
            elif phase == 'DAMAGED':
                if filename is None: raise RuntimeError('No completed synthetic backup')
                # Only the backup just created in this dedicated synthetic AVD is read.
                encrypted = adb('exec-out', 'cat', '/sdcard/Download/' + filename, raw=True)
                with tempfile.TemporaryDirectory(dir=mobile.BASE) as temporary:
                    damaged = Path(temporary)/'mealshop-verification-damaged.age'
                    damaged.write_bytes(encrypted[:-16])
                    adb('push', str(damaged), '/sdcard/Download/' + damaged.name)
                document('mealshop-verification-damaged.age')
            elif phase == 'CRASH':
                if value is None or not value.isdecimal(): raise RuntimeError('Invalid owned process id')
                # run-as constrains the signal to this verification package's UID.
                executable = adb('shell', 'run-as', PACKAGE, 'cat', '/proc/' + value + '/cmdline')
                if 'libmealshop_core.so\x00--device-stdio\x00' not in executable:
                    raise RuntimeError('Refusing to kill an unrelated process')
                adb('shell', 'run-as', PACKAGE, 'kill', '-9', value)
            else:
                shot('backups.png')
        if process.wait() != 0: raise RuntimeError('Native app-only integration failed')
        if phases != {'EXPORT', 'IMPORT', 'DAMAGED', 'CRASH', 'SCREEN'}:
            raise RuntimeError('Native app-only verification skipped required phases')
    finally:
        timeout.cancel()
        if process.poll() is None: process.terminate(); process.wait(timeout=30)

    # Same package/key, a normal entry-point APK, and deliberately empty Flutter records.
    adb('shell', 'am', 'force-stop', PACKAGE)
    adb('install', '-r', '--no-streaming', str(upgrade))
    for attempt in range(2):
        adb('shell', 'am', 'start', '-n', PACKAGE + '/org.mealshop.pocket.dev.MainActivity')
        consecutive = 0
        for _ in range(35):
            tree = screen()
            if any('Synthetic durable item' in n.attrib.get('content-desc','') or
                   'Synthetic durable item' in n.attrib.get('text','') for n in tree.iter('node')):
                consecutive += 1
                if consecutive >= 3: break
            else: consecutive = 0
            time.sleep(.3)
        else: raise RuntimeError('Canonical shopping item was missing after APK update/restart')
        time.sleep(1)
        shot('updated.png' if attempt == 0 else 'restarted.png')
        adb('shell', 'am', 'force-stop', PACKAGE)
    print('PASS: actual installed core, Android documents, failed restore, process death, APK update and restart')
