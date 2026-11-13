#!/usr/bin/env python3
"""Verify the installed, locked Dart package licence inventory; no device data is read."""
import argparse
import hashlib
import json
from pathlib import Path
import re
from urllib.parse import unquote, urljoin, urlparse
import urllib.request

ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'clients/flutter'

def inventory():
    config=json.loads((APP/'.dart_tool/package_config.json').read_text())
    entries=[]
    for package in config['packages']:
        if package['name']=='mealshop':
            continue
        folder=Path(unquote(urlparse(urljoin((APP/'.dart_tool/package_config.json').as_uri(),package['rootUri'])).path))
        spec=(folder/'pubspec.yaml').read_text()
        version=re.search(r'^version:\s*([^\s#]+)',spec,re.M)
        sdk=bool(re.search(r'^resolution:\s*workspace',spec,re.M)) and 'flutter/packages' in str(folder)
        license_file=next((folder/name for name in ['LICENSE','LICENSE.md','LICENSE.txt'] if (folder/name).is_file()),None)
        if license_file is None:
            license_file=next((parent/'LICENSE' for parent in folder.parents if (parent/'LICENSE').is_file() and (parent/'bin/flutter').is_file()),None)
        if license_file is None:
            raise SystemExit('No licence found for '+package['name'])
        data=license_file.read_bytes();text=data.decode().lower()
        if 'permission is hereby granted, free of charge' in text:
            spdx='MIT'
        elif 'redistribution and use in source and binary forms' in text:
            spdx='BSD-3-Clause' if 'neither the name' in text or 'names of its contributors' in text else 'BSD-2-Clause'
        elif 'apache license' in text and 'version 2.0' in text:
            spdx='Apache-2.0'
        else:
            raise SystemExit('Review the licence for '+package['name'])
        if not version:
            sdk=True
        entries.append({'name':package['name'],'version':version.group(1).strip("'\"") if version else 'Flutter SDK',
                        'licence':spdx,'licence_sha256':hashlib.sha256(data).hexdigest(),'sdk':sdk or 'var/mobile/flutter/packages' in str(folder)})
    return sorted(entries,key=lambda value:value['name'])


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write',action='store_true')
    parser.add_argument('--audit',action='store_true')
    args=parser.parse_args()
    entries=inventory();target=APP/'dependencies.json'
    content=json.dumps(entries,indent=2)+'\n'
    if args.write:
        target.write_text(content)
    elif target.read_text()!=content:
        raise SystemExit('Package inventory differs: review licences and run --write')
    print('PASS:',len(entries),'locked package licences verified')
    if args.audit:
        external=[entry for entry in entries if not entry['sdk']]
        request=urllib.request.Request('https://api.osv.dev/v1/querybatch',data=json.dumps({'queries':[{'package':{'name':entry['name'],'ecosystem':'Pub'},'version':entry['version']} for entry in external]}).encode(),headers={'Content-Type':'application/json'})
        with urllib.request.urlopen(request,timeout=30) as response:
            result=json.load(response)
        affected=[entry['name'] for entry,value in zip(external,result['results'],strict=True) if value.get('vulns')]
        if affected:
            raise SystemExit('Review OSV advisories for: '+', '.join(affected))
        print('PASS: OSV Pub audit:',len(external),'locked external packages, no matched advisories')

if __name__=='__main__':main()
