#!/usr/bin/env python3
"""Verify the locked Android Rust licence inventory and generate the APK's offline notices."""
import argparse
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
APP = ROOT/'clients/flutter'


class Text(HTMLParser):
    def __init__(self):
        super().__init__(); self.parts = []; self.hidden = False
    def handle_starttag(self, tag, attrs):
        if tag in ('style', 'script'): self.hidden = True
        if tag in ('p', 'div', 'h1', 'h2', 'h3', 'br', 'pre'): self.parts.append('\n')
    def handle_endtag(self, tag):
        if tag in ('style', 'script'): self.hidden = False
    def handle_data(self, data):
        if not self.hidden: self.parts.append(data)


def rust_sysroot(version):
    compiler = ['rustup', 'run', version, 'rustc'] if shutil.which('rustup') else ['rustc']
    actual = subprocess.check_output([*compiler, '--version'], text=True).split()
    if len(actual) < 2 or actual[1] != version:
        raise SystemExit('Native licence generation requires pinned Rust '+version)
    return Path(subprocess.check_output([*compiler, '--print', 'sysroot'], text=True).strip())


def collect():
    config = json.loads((APP/'toolchain.json').read_text())
    supplements = {(p['name'],p['version']):p for p in json.loads((APP/'rust-licence-sources.json').read_text())}
    packages = {}
    for target in ['x86_64-linux-android', 'aarch64-linux-android', 'armv7-linux-androideabi']:
        metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--offline',
            '--features', 'device', '--format-version', '1', '--filter-platform', target], cwd=ROOT))
        nodes = {n['id'] for n in metadata['resolve']['nodes']}
        for package in metadata['packages']:
            if package['id'] in nodes: packages[package['id']] = package
    commit = subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    inventory, notices = [], ['Mealshop corresponding source: https://github.com/Th3Link/mealshop/tree/'+commit+'\n']
    for package in sorted(packages.values(), key=lambda p:(p['name'],p['version'])):
        folder = Path(package['manifest_path']).parent
        files = sorted(p for p in folder.iterdir() if p.is_file() and
                       p.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'NOTICE')))
        material = [(file.name, file.read_bytes()) for file in files]
        source = supplements.get((package['name'],package['version']))
        if source is not None:
            data = (APP/source['file']).read_bytes()
            if hashlib.sha256(data).hexdigest() != source['sha256']: raise SystemExit('Licence supplement changed')
            if source.get('readme_sha256') and hashlib.sha256((folder/'README.md').read_bytes()).hexdigest() != source['readme_sha256']:
                raise SystemExit('Upstream licence declaration changed')
            material.append(('Upstream supplement: '+source['source'],data))
        if not material: raise SystemExit('Missing native licence: '+package['name'])
        inventory.append({'name':package['name'],'version':package['version'],'licence':package['license'],
            'repository':package['repository'],'notices':[{'file':name,'sha256':hashlib.sha256(data).hexdigest()} for name,data in material]})
        notices += ['\n'+'='*72+'\n',package['name']+' '+package['version']+'\n',
                    'Declared licence: '+str(package['license'])+'\n',
                    'Authors: '+', '.join(package['authors'])+'\n',
                    'Source: '+str(package['repository'])+'\n']
        for name, data in material: notices += ['\n'+name+'\n\n',data.decode('utf-8')+'\n']
    runtime = (rust_sysroot(config['rust_version'])/'share/doc/rust/COPYRIGHT-library.html').read_bytes()
    parser = Text(); parser.feed(runtime.decode('utf-8'))
    notices += ['\nRust standard library '+config['rust_version']+'\n', ''.join(parser.parts)]
    value = {'rust':config['rust_version'],'runtime_notices_sha256':hashlib.sha256(runtime).hexdigest(),'packages':inventory}
    return value, ''.join(notices)


def generate(write=False):
    value, notices = collect()
    content = json.dumps(value,indent=2)+'\n'
    target = APP/'rust-dependencies.json'
    if write: target.write_text(content)
    elif not target.is_file() or target.read_text() != content:
        raise SystemExit('Review native licences and run python3 scripts/native_notices.py --write')
    asset = APP/'android/app/src/main/assets/mealshop-core-notices.txt'
    asset.parent.mkdir(parents=True,exist_ok=True)
    asset.write_text(notices)
    print('PASS:',len(value['packages']),'locked Rust package licences and pinned runtime notices bundled')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write',action='store_true')
    generate(parser.parse_args().write)
