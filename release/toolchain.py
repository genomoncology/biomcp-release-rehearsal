"""Read the release tool pins and bootstrap the pinned Rust installer."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import subprocess
import sys
import tempfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]


def manifest():
    return json.loads(Path(__file__).with_name('toolchain.json').read_text())


def check():
    pins = manifest()
    for name, image in pins['images'].items():
        if not re.fullmatch(r'[^\s@]+@sha256:[0-9a-f]{64}', image):
            raise ValueError('toolchain.json image requires a sha256 digest: ' + name)
    for package in pins['runtime-packages']:
        if not re.fullmatch(r'[a-z0-9+.-]+=[a-zA-Z0-9+:.~_-]+', package):
            raise ValueError('toolchain.json runtime package requires a version')
    for digest in pins['rustup-sha256'].values():
        if not re.fullmatch(r'[0-9a-f]{64}', digest):
            raise ValueError('toolchain.json rustup requires a sha256 digest')
    source = (ROOT / 'rust-toolchain.toml').read_text()
    if not re.search(r'^channel\s*=\s*"' + re.escape(pins['rust']) + r'"\s*$', source, re.M):
        raise ValueError('rust-toolchain.toml differs from release/toolchain.json')
    # Literal CLI pins belong only in the manifest. Accept shell quoting and =.
    literal = re.compile(r'''--(?:default-toolchain|version)(?:\s+|=)\s*["']?\d''')
    # Recipes consume these pins rather than maintaining independent versions.
    for name in ('linux.sh', 'matrix.sh'):
        text = (ROOT / 'release' / name).read_text()
        if ('toolchain.py' not in text or literal.search(text)
                or re.search(r'(?:/cp\d|rustup/archive/\d)', text)):
            raise ValueError(name + ' must consume the release tool manifest')
    # Reject recipe copies so the reviewed manifest remains the only pin source.
    for name in ('workflow.py', 'rehearse.py', 'matrix.sh', 'linux.sh'):
        text = (ROOT / 'release' / name).read_text()
        if re.search(r'(?:quay\.io/pypa/manylinux|debian:bookworm|@sha256:)', text):
            raise ValueError(name + ' duplicates an image pin')
        if name.endswith('.sh') and ('toolchain.py image runtime' not in text
                or 'toolchain.py get runtime-packages' not in text):
            raise ValueError(name + ' must consume runtime pins')
    dockerfile = (ROOT / 'release/Dockerfile').read_text()
    if ('FROM ${runtime_image}' not in dockerfile
            or '--no-install-recommends ${runtime_packages}' not in dockerfile
            or re.search(r'^ARG runtime_(?:image|packages)=', dockerfile, re.M)):
        raise ValueError('Dockerfile must consume runtime pins without defaults')
    scanner = (ROOT / 'release/secrets.py').read_text()
    if re.search(r'^VERSION\s*=\s*[\"\']', scanner, re.M):
        raise ValueError('secrets.py duplicates the scanner pin')
    workflow = ROOT / '.github/workflows/release.yml'
    if workflow.exists():
        text = workflow.read_text()
        for field in ('python-version', 'toolchain'):
            if re.search(r'^\s+' + field + r':\s*[\'\"]?\d', text, re.M):
                raise ValueError('release.yml duplicates a ' + field + ' pin')
        if literal.search(text):
            raise ValueError('release.yml duplicates a release tool pin')
        for job in ('candidate', 'qualify'):
            body = re.search(r'(?ms)^  ' + job + r':\n(.*?)(?=^  [a-z-]+:\n|\Z)', text)
            if not body or 'release/toolchain.py outputs' not in body[1] or 'id: tools' not in body[1]:
                raise ValueError('release.yml ' + job + ' must read the release tool manifest')
    print('Release tool pins agree')


def install_rust():
    pins = manifest()
    system, machine = platform.system(), platform.machine().lower()
    arch = {'arm64': 'aarch64', 'amd64': 'x86_64'}.get(machine, machine)
    target = arch + {'Linux': '-unknown-linux-gnu', 'Darwin': '-apple-darwin',
                     'Windows': '-pc-windows-msvc'}[system]
    suffix = '.exe' if system == 'Windows' else ''
    base = 'https://static.rust-lang.org/rustup/archive/' + pins['rustup'] + '/' + target + '/rustup-init' + suffix
    with tempfile.TemporaryDirectory() as directory:
        installer = Path(directory) / ('rustup-init' + suffix)
        with urllib.request.urlopen(base, timeout=120) as response:
            payload = response.read()
        expected = pins['rustup-sha256'][target]
        if hashlib.sha256(payload).hexdigest() != expected:
            raise ValueError('rustup installer checksum differs')
        installer.write_bytes(payload)
        installer.chmod(0o700)
        subprocess.run([str(installer), '-y', '--no-modify-path', '--profile', 'minimal',
                        '--default-toolchain', pins['rust']], check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['get', 'image', 'outputs', 'check', 'install-rust', 'install-machete', 'python-check'])
    parser.add_argument('key', nargs='?')
    args = parser.parse_args()
    pins = manifest()
    if args.command == 'get':
        value = pins[args.key]
        print(' '.join(value) if isinstance(value, list) else value)
    elif args.command == 'image':
        print(pins['images'][args.key])
    elif args.command == 'outputs':
        for key in ('rust', 'rustup', 'python', 'cargo-deny'):
            print(key.replace('-', '_') + '=' + pins[key])
    elif args.command == 'check':
        check()
    elif args.command == 'install-rust':
        install_rust()
    elif args.command == 'install-machete':
        expected = pins['cargo-machete']
        try:
            installed = subprocess.check_output(['cargo', 'machete', '--version'], text=True).strip()
        except (OSError, subprocess.CalledProcessError):
            installed = ''
        if installed != expected:
            subprocess.run(['cargo', 'install', 'cargo-machete', '--locked',
                            '--version', pins['cargo-machete']], check=True)
    else:
        expected = tuple(map(int, pins['python'].split('.')))
        if sys.version_info[:len(expected)] != expected:
            raise ValueError('Python differs from release tool pin: ' + pins['python'])
        print('Release Python: ' + platform.python_version())


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
