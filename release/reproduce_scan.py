#!/usr/bin/env python3
"""Reproduce staged screening from retained native packages, without rebuilding."""
import argparse
import ast
import io
import json
import re
import shutil
import tarfile
import tempfile
import urllib.request
from pathlib import Path
import subprocess
import sys
import zipfile

if __package__:
    from .secrets import MAX_FILE
else:
    from secrets import MAX_FILE


# Diagnostic comparisons stay in memory. No matched material or its digest is
# printed, retained or added to a scanner allowlist.
SELFTEST_SOURCE = 'https://raw.githubusercontent.com/gnutls/gnutls/3.7.9/lib/crypto-selftests-pk.c'
KEY_PATTERN = rb'(?i)-----BEGIN[ A-Z0-9_-]{0,100}PRIVATE KEY(?: BLOCK)?-----[\s\S-]{64,}?KEY(?: BLOCK)?-----'
PEM_PATTERN = rb'-----BEGIN ([A-Z0-9 _-]*PRIVATE KEY(?: BLOCK)?)-----\r?\n[A-Za-z0-9+/=\r\n]+-----END \1-----'


def retained_libraries(scratch):
    for arch, directory in [('amd64', 'x86_64-linux-gnu'), ('arm64', 'aarch64-linux-gnu')]:
        label = 'biomcp-container-' + arch + '.tar'
        library = 'usr/lib/' + directory + '/libgnutls.so.30.34.3'
        libraries, package = [], False
        with tarfile.open(scratch / 'staged' / label) as container:
            for layer in container:
                if not layer.isfile() or not layer.name.startswith('blobs/sha256/'):
                    continue
                data = container.extractfile(layer).read(MAX_FILE + 1)
                if len(data) > MAX_FILE:
                    raise ValueError('retained layer exceeds screening limit')
                if len(data) <= 262 or data[257:262] != b'ustar':
                    continue
                with tarfile.open(fileobj=io.BytesIO(data)) as archive:
                    for member in archive:
                        name = member.name.removeprefix('./')
                        if not member.isfile():
                            continue
                        if name == 'var/lib/dpkg/status':
                            status = archive.extractfile(member).read(MAX_FILE + 1)
                            if len(status) > MAX_FILE:
                                raise ValueError('retained package metadata exceeds screening limit')
                            package |= any(b'Package: libgnutls30\n' in row and b'Version: 3.7.9-' in row
                                           for row in status.split(b'\n\n'))
                        if name != library:
                            continue
                        payload = archive.extractfile(member).read(MAX_FILE + 1)
                        if len(payload) > MAX_FILE:
                            raise ValueError('retained library exceeds screening limit')
                        libraries.append(payload)
        yield label, libraries, package


def classify_private_keys(scratch):
    with urllib.request.urlopen(SELFTEST_SOURCE, timeout=120) as response:
        source = response.read(1024 * 1024).decode('utf-8')
    fixtures = []
    for declaration in re.finditer(r'static const char [a-zA-Z0-9_]+\[\]\s*=\s*((?:"(?:\\.|[^"\\])*"\s*)+);', source):
        value = ''.join(ast.literal_eval(token) for token in re.findall(r'"(?:\\.|[^"\\])*"', declaration[1])).encode()
        fixtures.extend(match.group(0) for match in re.finditer(KEY_PATTERN, value))
    if not fixtures:
        raise ValueError('public self-test source unavailable')
    for label, libraries, package in retained_libraries(scratch):
        matches, bodies, public, constants = 0, 0, 0, 0
        for payload in libraries:
            for match in re.finditer(KEY_PATTERN, payload):
                value = match.group(0)
                matches += 1
                body = bool(re.fullmatch(PEM_PATTERN, value))
                bodies += body
                public += any(value == fixture for fixture in fixtures)
                constants += not body and b'\x00' in value
        print(json.dumps({'artifact': label, 'package': 'libgnutls30',
                          'upstream_version_confirmed': package, 'library_copies': len(libraries),
                          'private_key_rule_matches': matches, 'complete_pem_bodies': bodies,
                          'public_upstream_selftest_matches': public, 'nul_separated_marker_matches': constants,
                          'unclassified_matches': matches - public, 'source': SELFTEST_SOURCE}))
    return 0


def challenge_retained_libraries(scratch):
    fresh = subprocess.check_output([shutil.which('openssl'), 'genpkey', '-algorithm', 'RSA',
                                     '-pkeyopt', 'rsa_keygen_bits:1024'], stderr=subprocess.DEVNULL)
    with tempfile.TemporaryDirectory(prefix='biomcp-retained-key-challenge-') as temporary:
        target = Path(temporary)
        library = target / 'libgnutls.so.30.34.3'
        for label, libraries, package in retained_libraries(scratch):
            if not package or len(libraries) != 1:
                raise ValueError('retained library identity unavailable')
            library.write_bytes(libraries[0] + b'\n' + fresh)
            result = subprocess.run([sys.executable, str(Path(__file__).with_name('secrets.py')),
                                     'artifacts', str(target)], capture_output=True, text=True)
            rows = [json.loads(line) for line in result.stdout.splitlines()] if result.returncode == 1 else []
            if (result.returncode != 1 or result.stderr or not rows
                    or any(row != {'file': library.name, 'rule': 'private-key'} for row in rows)):
                raise ValueError('retained library challenge did not fail on the fresh key')
            print(json.dumps({'artifact': label, 'scanner_exit': result.returncode,
                              'file': library.name, 'rule': 'private-key'}))
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archives', type=Path, required=True)
    parser.add_argument('--scratch', type=Path, required=True)
    parser.add_argument('--classify-private-key-markers', action='store_true')
    parser.add_argument('--challenge-retained-libraries', action='store_true')
    args = parser.parse_args()
    if args.classify_private_key_markers or args.challenge_retained_libraries:
        try:
            return (challenge_retained_libraries(args.scratch) if args.challenge_retained_libraries
                    else classify_private_keys(args.scratch))
        except (OSError, ValueError, TypeError, SyntaxError, tarfile.TarError, EOFError, subprocess.SubprocessError):
            print(json.dumps({'category': 'classification-unavailable'}))
            return 2
    args.scratch.mkdir(parents=True, exist_ok=False)
    artifacts = args.scratch / 'artifacts'
    for path in sorted(args.archives.glob('native-*.zip')):
        with zipfile.ZipFile(path) as archive:
            # Retained workflow output is still checked before extraction.
            for member in archive.infolist():
                name = Path(member.filename)
                if name.is_absolute() or '..' in name.parts or '\\' in member.filename:
                    raise ValueError('unsafe retained member')
            archive.extractall(artifacts / path.stem)
    record = json.loads(next(artifacts.glob('*/release.json')).read_text())
    source = args.scratch / 'source'
    source.mkdir()
    archive = args.scratch / 'source.tar'
    subprocess.run(['git', 'archive', '--output', str(archive), record['revision']], check=True)
    subprocess.run(['tar', '-xf', str(archive), '-C', str(source)], check=True)
    staged = args.scratch / 'staged'
    subprocess.run([sys.executable, str(source / 'release/workflow.py'), 'stage',
                    '--artifacts', str(artifacts), '--plugin', str(source),
                    '--version', record['version'], '--revision', record['revision'],
                    '--output', str(staged)], check=True)
    return subprocess.run([sys.executable, str(Path(__file__).with_name('secrets.py')),
                           'artifacts', str(staged)]).returncode


if __name__ == '__main__':
    raise SystemExit(main())
