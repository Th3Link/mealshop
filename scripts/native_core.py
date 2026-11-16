#!/usr/bin/env python3
"""Build the shared Rust domain core as installed Android executables, never writable assets."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
APP = ROOT / 'clients/flutter'
CONFIG = json.loads((APP / 'toolchain.json').read_text())
TARGETS = {
    'x86_64': ('x86_64-linux-android', 'x86_64-linux-android24-clang'),
    'arm64-v8a': ('aarch64-linux-android', 'aarch64-linux-android24-clang'),
    'armeabi-v7a': ('armv7-linux-androideabi', 'armv7a-linux-androideabi24-clang'),
}

def fingerprint():
    files = [ROOT/'Cargo.toml', ROOT/'Cargo.lock', APP/'toolchain.json', Path(__file__).resolve()]
    for name in ['src','data','appinfo']:
        files.extend(path for path in (ROOT/name).rglob('*') if path.is_file() and path != ROOT/'src/docs.rs')
    digest = hashlib.sha256()
    for file in sorted(files):
        digest.update(str(file.relative_to(ROOT)).encode()+b'\0')
        digest.update(file.read_bytes()+b'\0')
    return digest.hexdigest()


def verify(abis):
    path = APP/'android/app/src/main/assets/mealshop-core-build.json'
    if not path.is_file(): raise SystemExit('Build the Android Rust core with scripts/native_core.py first')
    metadata = json.loads(path.read_text())
    source = fingerprint()
    for abi in abis:
        binary = APP/'android/app/src/main/jniLibs'/abi/'libmealshop_core.so'
        record = metadata.get(abi,{})
        if not binary.is_file() or record.get('source_sha256') != source or record.get('rust') != CONFIG['rust_version']:
            raise SystemExit('Missing or stale Android core for '+abi+'; run python3 scripts/native_core.py')
        with binary.open('rb') as stream:
            if hashlib.file_digest(stream,'sha256').hexdigest() != record.get('binary_sha256'):
                raise SystemExit('Native Android core checksum differs for '+abi)
    print('PASS: native binaries match the current public core sources:',', '.join(abis))


def build(abis):
    from native_notices import generate
    generate()
    source = fingerprint()
    manifest = APP/'android/app/src/main/assets/mealshop-core-build.json'
    metadata = json.loads(manifest.read_text()) if manifest.exists() else {}
    tools = ROOT / 'var/mobile/android-sdk/ndk' / CONFIG['android_ndk'] / 'toolchains/llvm/prebuilt/linux-x86_64/bin'
    output = ROOT / 'var/mobile/rust-target'
    for abi in abis:
        target, clang = TARGETS[abi]
        env = os.environ | {
            'CARGO_TARGET_DIR': str(output),
            'CARGO_TARGET_' + target.upper().replace('-', '_') + '_LINKER': str(tools / clang),
            'CC_' + target.replace('-', '_'): str(tools / clang),
            'AR_' + target.replace('-', '_'): str(tools / 'llvm-ar'),
            'RUSTFLAGS': '-C link-arg=-Wl,-z,max-page-size=16384',
            'CARGO_BUILD_JOBS': '2',
            'MEALSHOP_CORE_SOURCE': source,
        }
        subprocess.run(['cargo', '+' + CONFIG['rust_version'], 'build', '--target', target,
                        '--features', 'device', '--release', '--locked'], cwd=ROOT, env=env, check=True)
        if fingerprint() != source: raise SystemExit('Core sources changed during compilation; rebuild before packaging')
        binary = output / target / 'release/mealshop-server'
        subprocess.run([str(tools / 'llvm-strip'), str(binary)], check=True)
        headers = subprocess.check_output([str(tools / 'llvm-readelf'), '-lW', str(binary)], text=True)
        loads = [line.split() for line in headers.splitlines() if line.strip().startswith('LOAD ')]
        if not loads or any(int(row[-1], 16) < 16384 for row in loads):
            raise SystemExit('Native core does not support 16 KiB memory pages')
        destination = APP / 'android/app/src/main/jniLibs' / abi / 'libmealshop_core.so'
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(binary, destination)
        destination.chmod(0o755)
        with destination.open('rb') as stream:
            digest = hashlib.file_digest(stream,'sha256').hexdigest()
        metadata[abi] = {'source_sha256':source,'binary_sha256':digest,'rust':CONFIG['rust_version'],'target':target}
        manifest.write_text(json.dumps(metadata,indent=2)+'\n')
        print('Installed native core for', abi, flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--abi', choices=TARGETS, action='append')
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    if args.check: verify(args.abi or list(TARGETS))
    else: build(args.abi or list(TARGETS))
