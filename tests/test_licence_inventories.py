"""Licence refresh must preserve review boundaries before either file is changed."""
from copy import deepcopy
import importlib
import json
from pathlib import Path
import re
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import verify_mobile_packages
import native_notices


class SdkInventory(unittest.TestCase):
    def test_flutter_sdk_is_recognized_outside_the_development_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            sdk = root / 'tools/flutter/3.50.0'
            (sdk / 'bin').mkdir(parents=True)
            (sdk / 'bin/flutter').touch()
            package = sdk / 'packages/flutter'
            package.mkdir(parents=True)
            (package / 'pubspec.yaml').write_text('name: flutter\nversion: 0.0.0\n')
            (sdk / 'LICENSE').write_text('Redistribution and use in source and binary forms. Neither the name.')
            app = root / 'app'
            (app / '.dart_tool').mkdir(parents=True)
            (app / '.dart_tool/package_config.json').write_text(json.dumps({
                'packages': [{'name': 'flutter', 'rootUri': package.as_uri()}],
            }))
            with patch.object(verify_mobile_packages, 'APP', app):
                self.assertTrue(verify_mobile_packages.inventory()[0]['sdk'])


class LicenceInventories(unittest.TestCase):
    def setUp(self):
        self.refresh = importlib.import_module('renovate_licences')
        self.dart = [{'name': 'example', 'version': '1.0.0', 'licence': 'MIT',
                      'licence_sha256': 'reviewed', 'sdk': False}]
        self.rust = {'rust': '1.100.0', 'runtime_notices_sha256': 'reviewed-runtime',
                     'packages': [{'name': 'example', 'version': '1.0.0', 'licence': 'MIT',
                                   'repository': 'https://example.org/library',
                                   'notices': [{'file': 'LICENSE', 'sha256': 'reviewed'}]}]}

    def test_version_updates_and_removals_are_repeatable(self):
        dart, rust = deepcopy(self.dart), deepcopy(self.rust)
        dart[0]['version'] = '1.0.1'
        rust['packages'][0]['version'] = '2.0.0'
        with tempfile.TemporaryDirectory() as directory:
            app = Path(directory)
            (app / 'dependencies.json').write_text(json.dumps(self.dart))
            (app / 'rust-dependencies.json').write_text(json.dumps(self.rust))
            with patch.object(self.refresh, 'APP', app):
                self.refresh.publish(dart, rust)
                first = {p.name: p.read_bytes() for p in app.iterdir()}
                self.refresh.publish(dart, rust)
                self.assertEqual(first, {p.name: p.read_bytes() for p in app.iterdir()})
                self.refresh.publish([], {**rust, 'packages': []})
                self.assertEqual([], json.loads((app / 'dependencies.json').read_text()))

    def test_unreviewed_changes_preserve_both_existing_inventories(self):
        for change in ('new_package', 'new_dart', 'dart_hash', 'licence', 'repository', 'native_hash', 'runtime'):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as directory:
                app = Path(directory)
                dart, rust = deepcopy(self.dart), deepcopy(self.rust)
                dart[0]['version'] = '1.0.1'
                if change == 'new_package':
                    rust['packages'].append({**rust['packages'][0], 'name': 'unreviewed'})
                elif change == 'new_dart':
                    dart.append({**dart[0], 'name': 'unreviewed'})
                elif change == 'dart_hash':
                    dart[0]['licence_sha256'] = 'changed'
                elif change == 'licence':
                    rust['packages'][0]['licence'] = 'unreviewed'
                elif change == 'repository':
                    rust['packages'][0]['repository'] = 'https://elsewhere.example'
                elif change == 'native_hash':
                    rust['packages'][0]['notices'][0]['sha256'] = 'changed'
                else:
                    rust['runtime_notices_sha256'] = 'changed'
                (app / 'dependencies.json').write_text(json.dumps(self.dart))
                (app / 'rust-dependencies.json').write_text(json.dumps(self.rust))
                before = {p.name: p.read_bytes() for p in app.iterdir()}
                with patch.object(self.refresh, 'APP', app), self.assertRaisesRegex(SystemExit, 'review'):
                    self.refresh.publish(dart, rust)
                self.assertEqual(before, {p.name: p.read_bytes() for p in app.iterdir()})

    def test_multiple_native_versions_cannot_hide_an_unreviewed_package(self):
        rust = deepcopy(self.rust)
        rust['packages'].append({**rust['packages'][0], 'version': '2.0.0'})
        with self.assertRaisesRegex(SystemExit, 'review'):
            self.refresh.validate_refresh(self.rust, rust)

    def test_runner_configuration_limits_commands_files_and_tool_versions(self):
        config = self.refresh.runner_config()
        pattern, = config['allowedCommands']
        self.assertIsNotNone(re.fullmatch(pattern, 'python3 scripts/renovate_licences.py'))
        for command in ('python3 scripts/renovate_licencesXpy',
                        'python3 scripts/renovate_licences.py; echo unexpected',
                        'python3 scripts/renovate_licences.py --write'):
            self.assertIsNone(re.fullmatch(pattern, command))
        rule, = config['packageRules']
        self.assertEqual(['cargo', 'pub'], rule['matchManagers'])
        pins = json.loads((self.refresh.APP / 'toolchain.json').read_text())
        self.assertEqual(pins['rust_version'], rule['constraints']['rust'])
        self.assertEqual(pins['flutter_version'], rule['constraints']['flutter'])
        task = rule['postUpgradeTasks']
        self.assertEqual('branch', task['executionMode'])
        self.assertEqual(['clients/flutter/dependencies.json', 'clients/flutter/rust-dependencies.json'], task['fileFilters'])
        self.assertEqual({'python', 'rust', 'flutter'}, set(task['installTools']))

    def test_wrong_flutter_revision_stops_before_resolution_or_publication(self):
        pins = json.loads((self.refresh.APP / 'toolchain.json').read_text())
        wrong = json.dumps({'frameworkVersion': pins['flutter_version'], 'frameworkRevision': 'wrong'})
        with patch.object(self.refresh.subprocess, 'check_output', return_value=wrong), \
                patch.object(self.refresh.subprocess, 'run') as run, \
                self.assertRaisesRegex(SystemExit, 'pinned Flutter'):
            self.refresh.refresh()
        run.assert_not_called()


class RustRuntime(unittest.TestCase):
    def test_containerbase_compiler_needs_no_rustup(self):
        with patch.object(native_notices.shutil, 'which', return_value=None), \
                patch.object(native_notices.subprocess, 'check_output',
                             side_effect=['rustc 1.100.0 (synthetic)\n', '/tools/rust/1.100.0\n']) as run:
            self.assertEqual(Path('/tools/rust/1.100.0'), native_notices.rust_sysroot('1.100.0'))
        self.assertEqual(['rustc', '--version'], run.call_args_list[0].args[0])

    def test_wrong_global_rust_version_is_rejected(self):
        with patch.object(native_notices.shutil, 'which', return_value=None), \
                patch.object(native_notices.subprocess, 'check_output', return_value='rustc 1.101.0\n'), \
                self.assertRaisesRegex(SystemExit, 'pinned Rust'):
            native_notices.rust_sysroot('1.100.0')


if __name__ == '__main__':
    unittest.main()
