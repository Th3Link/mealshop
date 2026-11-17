#!/usr/bin/env python3
"""Render Renovate's runner policy or refresh already-reviewed licence identities."""
import argparse
from collections import Counter
import json
from pathlib import Path
import shutil
import subprocess

import native_notices
import verify_mobile_packages

ROOT = Path(__file__).resolve().parents[1]
APP = ROOT / 'clients/flutter'
COMMAND = 'python3 scripts/renovate_licences.py'


def runner_config():
    pins = json.loads((APP / 'toolchain.json').read_text())
    return {
        'allowedCommands': ['^'+COMMAND.replace('.', r'\.')+'$'],
        'packageRules': [{
            'matchManagers': ['cargo', 'pub'],
            'constraints': {'python': '>=3.13', 'rust': pins['rust_version'],
                            'flutter': pins['flutter_version']},
            'postUpgradeTasks': {
                'commands': [COMMAND], 'executionMode': 'branch',
                'fileFilters': ['clients/flutter/dependencies.json',
                                'clients/flutter/rust-dependencies.json'],
                'installTools': {'python': {}, 'rust': {}, 'flutter': {}},
            },
        }],
    }


def validate_refresh(before, after):
    if isinstance(before, dict):
        if {k: v for k, v in before.items() if k != 'packages'} != {
                k: v for k, v in after.items() if k != 'packages'}:
            raise SystemExit('Licence review required for changed Rust runtime notices')
        before, after = before['packages'], after['packages']
    def identities(entries):
        return Counter(json.dumps({k: v for k, v in entry.items() if k != 'version'},
                                  sort_keys=True) for entry in entries)
    unreviewed = identities(after) - identities(before)
    if unreviewed:
        names = sorted({json.loads(identity)['name'] for identity in unreviewed})
        raise SystemExit('Licence review required for: '+', '.join(names)+
                         '; review sources before using the explicit inventory --write commands')


def publish(dart, rust):
    candidates = {APP / 'dependencies.json': dart, APP / 'rust-dependencies.json': rust}
    for path, value in candidates.items():
        validate_refresh(json.loads(path.read_text()), value)
    for path, value in candidates.items():
        path.write_text(json.dumps(value, indent=2)+'\n')
    print('PASS: refreshed', len(dart), 'Dart and', len(rust['packages']),
          'native licence records; reviewed identities preserved')


def refresh():
    pins = json.loads((APP / 'toolchain.json').read_text())
    flutter = ROOT / 'var/mobile/flutter/bin/flutter'
    if not flutter.is_file():
        flutter = shutil.which('flutter')
    if not flutter:
        raise SystemExit('Install the Flutter toolchain pinned in clients/flutter/toolchain.json')
    version = json.loads(subprocess.check_output(
        [str(flutter), '--version', '--machine'], text=True))
    if (version['frameworkVersion'] != pins['flutter_version'] or
            version['frameworkRevision'] != pins['flutter_commit']):
        raise SystemExit('Licence refresh requires the pinned Flutter version and revision')
    subprocess.run([str(flutter), 'pub', 'get', '--enforce-lockfile'], cwd=APP, check=True)
    subprocess.run(['cargo', 'fetch', '--locked'], cwd=ROOT, check=True)
    dart = verify_mobile_packages.inventory()
    rust, _ = native_notices.collect()
    publish(dart, rust)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--config', action='store_true', help='Print the self-hosted runner configuration')
    args = parser.parse_args()
    if args.config:
        print(json.dumps(runner_config(), indent=2))
    else:
        refresh()
