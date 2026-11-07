"""Release contract: callers reject artifacts that lose required notices."""
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest
import zipfile

LICENSE = Path('LICENSE').read_bytes()
NOTICES = (b'Generated third-party notices\nSynthetic fixture dependency\nPermission notice\n'
           b'AlphaGenome protocol messages, Copyright 2024 Google LLC (Apache-2.0)\n')


def payloads():
    return {'LICENSE': LICENSE, 'THIRD-PARTY-NOTICES.txt': NOTICES}


def write_payloads(directory):
    for name, data in payloads().items():
        (directory / name).write_bytes(data)


def archive(path, files):
    if path.name.endswith(('.tar.gz', '.tar')):
        with tarfile.open(path, 'w:gz' if path.name.endswith('.gz') else 'w') as target:
            for name, value in files.items():
                entry = tarfile.TarInfo(name)
                entry.size = len(value)
                target.addfile(entry, io.BytesIO(value))
    else:
        with zipfile.ZipFile(path, 'w') as target:
            for name, value in files.items():
                target.writestr(name, value)


def wheel_fixture(directory, tag):
    binary = tag.encode()
    record = {'version': '1.0.0', 'revision': 'a' * 40, 'platform': tag,
              'distribution': 'biomcp-cli', 'profile': 'release',
              'cargo_profile': {'debug_assertions': False, 'opt_level': '3'},
              'binary_sha256': hashlib.sha256(binary).hexdigest()}
    prefix = 'biomcp_cli-1.0.0.dist-info/'
    suffix = '.exe' if tag.startswith('win_') else ''
    files = {prefix + name: value for name, value in payloads().items()}
    files.update({prefix + 'release.json': json.dumps(record).encode(),
                  prefix + 'METADATA': b'Name: biomcp-cli\nVersion: 1.0.0\nLicense: MIT\nProject-URL: Homepage, https://biomcp.org\n\nmcp-name: io.github.genomoncology/biomcp\n',
                  prefix + 'WHEEL': ('Tag: py3-none-' + tag + '\n').encode(),
                  'biomcp_cli-1.0.0.data/scripts/biomcp' + suffix: binary,
                  'biomcp_cli-1.0.0.data/scripts/biomcp-cli' + suffix: binary})
    path = directory / ('biomcp_cli-1.0.0-py3-none-' + tag + '.whl')
    archive(path, files)
    return path, record


class Licensing(unittest.TestCase):
    def test_generation_excludes_dependencies_that_do_not_ship(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            cargo = root / 'cargo'
            cargo.write_text(
                f"#!{sys.executable}\n"
                "import json, pathlib, sys, tomllib\n"
                "if sys.argv[1:3] == ['vendor', '--locked']:\n"
                "    assert len(sys.argv) == 4\n"
                "    with pathlib.Path(__file__).with_name('vendors').open('a') as log: log.write('locked\\n')\n"
                "elif sys.argv[1:] == ['about', '--version']:\n"
                "    print('cargo-about ' + json.loads(pathlib.Path('release/toolchain.json').read_text())['cargo-about'])\n"
                "else:\n"
                "    args = sys.argv[1:]\n"
                "    config = tomllib.loads(pathlib.Path(args[args.index('--config') + 1]).read_text())\n"
                "    assert config.get('ignore-dev-dependencies') is True, 'dev dependencies do not ship'\n"
                "    assert config.get('ignore-build-dependencies') is True, 'build dependencies do not ship'\n"
                "    data = {'licenses': [{'name': 'MIT License', 'id': 'MIT', 'text': 'Permission notice',\n"
                "             'used_by': [{'crate': {'name': 'shipping-crate', 'version': '1.0.0'}}]}],\n"
                "            'crates': [{'name': 'shipping-crate'}]}\n"
                "    pathlib.Path(args[args.index('--output-file') + 1]).write_text(json.dumps(data))\n"
            )
            cargo.chmod(0o755)
            result = subprocess.run(
                [sys.executable, '-c', 'from pathlib import Path; from release.notices import generate; '
                 'generate(Path(__import__("sys").argv[1]))', str(root)],
                env=dict(os.environ, PATH=str(root) + os.pathsep + os.environ['PATH']),
                capture_output=True, text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual((root / 'vendors').read_text().splitlines(), ['locked'])
            notices = (root / 'THIRD-PARTY-NOTICES.txt').read_text()
            self.assertIn('shipping-crate 1.0.0', notices)
            self.assertIn('Permission notice', notices)
            self.assertEqual((root / 'LICENSE').read_bytes(), LICENSE)

    def test_inspection_requires_license_notices_and_registry_metadata(self):
        for defect in ('LICENSE', 'THIRD-PARTY-NOTICES.txt', 'protos', 'marker', 'License', 'Project-URL', None):
            with self.subTest(defect=defect), tempfile.TemporaryDirectory() as temp:
                path, _ = wheel_fixture(Path(temp), 'linux_x86_64')
                with zipfile.ZipFile(path) as source:
                    files = {name: source.read(name) for name in source.namelist()}
                if defect in payloads():
                    del files['biomcp_cli-1.0.0.dist-info/' + defect]
                elif defect == 'protos':
                    name = 'biomcp_cli-1.0.0.dist-info/THIRD-PARTY-NOTICES.txt'
                    files[name] = files[name].replace(b'AlphaGenome protocol messages', b'Other messages')
                elif defect:
                    name = 'biomcp_cli-1.0.0.dist-info/METADATA'
                    token = {'marker': b'mcp-name: io.github.genomoncology/biomcp', 'License': b'License: MIT',
                             'Project-URL': b'Project-URL: Homepage, https://biomcp.org'}[defect]
                    files[name] = files[name].replace(token, b'')
                archive(path, files)
                result = subprocess.run([sys.executable, 'release/artifacts.py', 'inspect', str(path)], capture_output=True, text=True)
                self.assertEqual(result.returncode == 0, defect is None, result.stderr)

    def test_staging_refuses_missing_license_notice_or_registry_marker(self):
        from release.test_workflow import Staging
        for kind in ('wheel', 'archive', 'desktop', 'container'):
            for missing in ('LICENSE', 'THIRD-PARTY-NOTICES.txt', 'marker'):
                if missing == 'marker' and kind != 'wheel':
                    continue
                with self.subTest(kind=kind, missing=missing), tempfile.TemporaryDirectory() as temp:
                    root = Path(temp)
                    staging = Staging()
                    artifacts, plugin = staging.prepare(root)
                    tag = 'macosx_11_0_arm64' if kind == 'desktop' else 'manylinux_2_28_x86_64'
                    directory = artifacts / tag
                    pattern = {'wheel': '*.whl', 'archive': '*.tar.gz', 'desktop': '*.mcpb',
                               'container': 'biomcp-container-*.tar'}[kind]
                    path, = directory.glob(pattern)
                    self.remove(path, missing, kind)
                    if kind == 'archive':
                        path.with_name(path.name + '.sha256').write_text(hashlib.sha256(path.read_bytes()).hexdigest() + '  ' + path.name)
                    result = staging.stage(root, artifacts, plugin)
                    self.assertNotEqual(result.returncode, 0, result.stdout)
                    self.assertIn('requires', result.stderr)

    def remove(self, path, missing, kind):
        if kind in ('wheel', 'desktop'):
            with zipfile.ZipFile(path) as source:
                files = {name: source.read(name) for name in source.namelist()}
            prefix = 'biomcp_cli-1.0.0.dist-info/' if kind == 'wheel' else ''
            if missing == 'marker':
                files[prefix + 'METADATA'] = files[prefix + 'METADATA'].replace(b'mcp-name:', b'other:')
            else:
                del files[prefix + missing]
        else:
            with tarfile.open(path) as source:
                files = {entry.name: source.extractfile(entry).read() for entry in source if entry.isfile()}
            if kind == 'container':
                with tarfile.open(fileobj=io.BytesIO(files['layer.tar'])) as source:
                    layer = {entry.name: source.extractfile(entry).read() for entry in source if entry.isfile()}
                del layer['usr/share/doc/biomcp/' + missing]
                with tempfile.TemporaryDirectory() as temp:
                    target = Path(temp) / 'layer.tar'
                    archive(target, layer)
                    files['layer.tar'] = target.read_bytes()
            else:
                del files[missing]
        archive(path, files)

    def test_staging_refuses_container_directory_whiteouts(self):
        from release.test_workflow import Staging
        for whiteout in ('.wh..wh..opq', '.wh.usr', 'usr/share/doc/.wh.biomcp', 'usr/share/doc/biomcp/.wh..wh..opq'):
            with self.subTest(whiteout=whiteout), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                staging = Staging()
                artifacts, plugin = staging.prepare(root)
                path, = (artifacts / 'manylinux_2_28_x86_64').glob('biomcp-container-*.tar')
                with tarfile.open(path) as source:
                    files = {entry.name: source.extractfile(entry).read() for entry in source if entry.isfile()}
                layer = root / 'upper.tar'
                archive(layer, {whiteout: b''})
                files['upper.tar'] = layer.read_bytes()
                files['manifest.json'] = b'[{"Layers": ["layer.tar", "upper.tar"]}]'
                archive(path, files)
                result = staging.stage(root, artifacts, plugin)
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertIn('requires LICENSE', result.stderr)
