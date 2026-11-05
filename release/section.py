"""Bind selected main source and separately checked-out rehearsal driver."""
import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess


def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args], text=True).strip()


def identity(root, expected):
    if not re.fullmatch(r'[0-9a-f]{40}', expected):
        raise ValueError('requested identity must be a full 40-character commit')
    revision = git(root, 'rev-parse', 'HEAD')
    if revision != expected:
        raise ValueError('checkout revision differs from requested identity')
    if git(root, 'status', '--porcelain', '--untracked-files=no'):
        raise ValueError('tracked checkout bytes differ from committed source')
    paths = subprocess.check_output(['git', '-C', str(root), 'ls-files', '-z']).split(b'\0')
    source = hashlib.sha256()
    for name in sorted(path for path in paths if path):
        path = root / os.fsdecode(name)
        data = os.fsencode(os.readlink(path)) if path.is_symlink() else path.read_bytes()
        source.update(name + b'\0' + hashlib.sha256(data).digest())
    return {'revision': revision, 'tree': git(root, 'rev-parse', 'HEAD^{tree}'),
            'tracked_bytes_sha256': source.hexdigest()}


def selected_product(product):
    result = identity(product, os.environ['PRODUCT_REVISION'])
    tree = os.environ['PRODUCT_TREE']
    if not re.fullmatch(r'[0-9a-f]{40}', tree) or result['tree'] != tree:
        raise ValueError('checkout tree differs from requested main tree')
    return result


def bind(product, tooling, output):
    record = {'product': selected_product(product),
              'tooling': identity(tooling, os.environ['GITHUB_SHA']),
              'publishing': False}
    for name in ('Cargo.lock', 'rust-toolchain.toml', 'release/toolchain.json'):
        if (product / name).read_bytes() != (tooling / name).read_bytes():
            raise ValueError('recovery must preserve product tool pins: ' + name)
    output.mkdir(parents=True, exist_ok=True)
    (output / 'sources.json').write_text(json.dumps(record, indent=2) + '\n')


def retain(product, tooling, output):
    output.mkdir(parents=True, exist_ok=True)
    hashes = {str(path.relative_to(product)): hashlib.sha256(path.read_bytes()).hexdigest()
              for path in sorted((product / 'dist').glob('*')) if path.is_file()}
    record = {'run': os.environ.get('GITHUB_RUN_ID'), 'section': os.environ.get('SECTION'),
              'product': selected_product(product),
              'tooling': identity(tooling, os.environ['GITHUB_SHA']), 'sha256': hashes}
    (output / 'artifacts.json').write_text(json.dumps(record, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=('bind', 'retain'))
    for name in ('product', 'tooling', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    command = bind if args.command == 'bind' else retain
    command(args.product, args.tooling, args.output)


if __name__ == '__main__':
    main()
