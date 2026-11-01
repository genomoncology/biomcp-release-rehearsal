"""Release contracts at the tool check and cold rehearsal command boundaries."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class ReleaseTools(unittest.TestCase):
    def test_machete_installs_only_when_the_pinned_version_is_missing(self):
        with tempfile.TemporaryDirectory() as directory:
            tools = Path(directory)
            cargo = tools / 'cargo'
            cargo.write_text("""#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
with Path(os.environ['CALLS']).open('a') as stream:
    stream.write(json.dumps(sys.argv[1:]) + '\\n')
if sys.argv[1:] == ['machete', '--version']:
    print(os.environ['INSTALLED_VERSION'])
""")
            cargo.chmod(0o755)
            calls = tools / 'calls'
            pin = json.loads((ROOT / 'release/toolchain.json').read_text())['cargo-machete']
            for installed in (pin, '0.0.0'):
                with self.subTest(installed=installed):
                    calls.unlink(missing_ok=True)
                    env = dict(os.environ, PATH=str(tools) + os.pathsep + os.environ['PATH'],
                               CALLS=str(calls), INSTALLED_VERSION=installed)
                    subprocess.run([sys.executable, 'release/toolchain.py', 'install-machete'],
                                   cwd=ROOT, env=env, check=True)
                    actual = [json.loads(line) for line in calls.read_text().splitlines()]
                    expected = [['machete', '--version']]
                    if installed != pin:
                        expected.append(['install', 'cargo-machete', '--locked', '--version', pin])
                    self.assertEqual(actual, expected)

    def test_toolchain_drift_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shutil.copytree(ROOT / 'release', root / 'release')
            shutil.copytree(ROOT / '.github', root / '.github')
            shutil.copyfile(ROOT / 'rust-toolchain.toml', root / 'rust-toolchain.toml')
            command = [sys.executable, str(root / 'release/toolchain.py'), 'check']
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            mutations = [
                ('rust-toolchain.toml', lambda s: s.replace('1.95.0', '1.94.0')),
                ('release/linux.sh', lambda s: s + '\ncurl https://static.rust-lang.org/rustup/archive/0.0.1/installer\n'),
                ('release/secrets.py', lambda s: s + '\nVERSION = "0.0.1"\n'),
                ('release/Dockerfile', lambda s: s.replace('FROM ${runtime_image}', 'FROM debian:bookworm-slim')),
                ('release/linux.sh', lambda s: s.replace('toolchain.py image runtime', 'toolchain.py get runtime')),
                ('release/matrix.sh', lambda s: s.replace('toolchain.py get runtime-packages', 'toolchain.py get unpinned-packages')),
                ('release/workflow.py', lambda s: s.replace('manifest()["images"]["manylinux-x86_64"]', '"quay.io/pypa/manylinux_2_28_x86_64"')),
                ('release/rehearse.py', lambda s: s.replace("manifest()['images']['manylinux-' + wheel_arch]", "'quay.io/pypa/manylinux_2_28_' + wheel_arch")),
                ('.github/workflows/release.yml', lambda s: s.replace(
                    'python-version: ${{ steps.tools.outputs.python }}', "python-version: '3.11'")),
            ]
            for name in ('release/linux.sh', 'release/matrix.sh', '.github/workflows/release.yml'):
                for flag in ('--default-toolchain', '--version'):
                    for value in (' "0.0.1"', "='0.0.1'", ' 0.0.1'):
                        def inject_pin(text, name=name, flag=flag, value=value):
                            if name.endswith('.yml') and flag == '--version':
                                return text.replace('--version "${{ steps.tools.outputs.cargo_deny }}"', flag + value)
                            interpreter = 'python'
                            original = interpreter + ' release/toolchain.py install-rust'
                            tool = 'sh /tmp/rustup-init' if flag == '--default-toolchain' else 'cargo install cargo-deny'
                            return text.replace(original, tool + ' ' + flag + value)
                        mutations.append((name, inject_pin))
            for occurrence in (1, 2):
                def remove_reader(text, occurrence=occurrence):
                    marker = 'release/toolchain.py outputs'
                    parts = text.split(marker)
                    parts[occurrence - 1] += 'echo omitted-reader'
                    return marker.join(parts[:occurrence]) + marker.join(parts[occurrence:])
                mutations.append(('.github/workflows/release.yml', remove_reader))
            for name, mutate in mutations:
                with self.subTest(recipe=name):
                    path = root / name
                    original = path.read_text()
                    changed = mutate(original)
                    self.assertNotEqual(changed, original)
                    path.write_text(changed)
                    result = subprocess.run(command, capture_output=True, text=True)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn(Path(name).name, result.stderr)
                    path.write_text(original)

    def test_supply_chain_pins_refuse_floating_images_and_installer_mismatch(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shutil.copytree(ROOT / 'release', root / 'release')
            shutil.copytree(ROOT / '.github', root / '.github')
            shutil.copyfile(ROOT / 'rust-toolchain.toml', root / 'rust-toolchain.toml')
            path = root / 'release/toolchain.json'
            pins = json.loads(path.read_text())
            for key in ('manylinux-x86_64', 'manylinux-aarch64', 'runtime'):
                changed = json.loads(json.dumps(pins))
                changed['images'][key] = 'example/image:latest'
                path.write_text(json.dumps(changed))
                result = subprocess.run([sys.executable, 'release/toolchain.py', 'check'],
                                        cwd=root, capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('digest', result.stderr)
            path.write_text(json.dumps(pins))
            # Execute the real installer command with only its download boundary replaced.
            runner = """import hashlib, io, runpy, sys, urllib.request
payload = b'wrong installer'
def download(url, **kwargs):
    # A compromised origin can replace both the installer and its checksum.
    value = hashlib.sha256(payload).hexdigest().encode() if url.endswith('.sha256') else payload
    return io.BytesIO(value)
urllib.request.urlopen = download
sys.argv = ['release/toolchain.py', 'install-rust']
runpy.run_path(sys.argv[0], run_name='__main__')
"""
            result = subprocess.run([sys.executable, '-c', runner], cwd=root,
                                    capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('rustup installer checksum differs', result.stderr)

    def test_cold_stages_report_all_failures_and_run_later_stages(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'release').mkdir()
            shutil.copyfile(ROOT / 'release/rehearse.py', root / 'release/rehearse.py')
            shutil.copyfile(ROOT / 'release/toolchain.json', root / 'release/toolchain.json')
            stub = '''import os, sys\nfrom pathlib import Path\nwith open('seen', 'a') as stream: stream.write(Path(sys.argv[0]).stem + ':' + ' '.join(sys.argv[1:]) + '\\n')\nsys.exit(7 if Path(sys.argv[0]).stem == 'artifacts' or (Path(sys.argv[0]).stem == 'toolchain' and sys.argv[1] == 'install-rust') else 0)\n'''
            for name in ('toolchain', 'artifacts', 'gate', 'workflow', 'secrets'):
                (root / ('release/' + name + '.py')).write_text(stub)
            bindir = root / 'bin'
            bindir.mkdir()
            for name in ('cargo', 'make'):
                executable = bindir / name
                executable.write_text('#!' + sys.executable + '\n' + stub)
                executable.chmod(0o755)
            env = dict(os.environ, PATH=str(bindir) + os.pathsep + os.environ['PATH'],
                       BIOMCP_BUILD_REVISION='a' * 40)
            report = root / 'report.json'
            result = subprocess.run([sys.executable, 'release/rehearse.py', 'inside',
                                     '--report', str(report)], cwd=root, env=env,
                                    capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            rows = json.loads(report.read_text())
            self.assertEqual([row['stage'] for row in rows if row['exit'] != 0],
                             ['toolchain', 'build'])
            self.assertEqual(rows[-1]['stage'], 'schema')
            self.assertEqual(rows[-1]['exit'], 0)
            self.assertIn('gate:--prepare-only', (root / 'seen').read_text())
            self.assertIn('secrets:history', (root / 'seen').read_text())
            print('Seeded failures: toolchain=7, build=7; later prepare and schema ran')

    @unittest.skipIf(os.name == 'nt', 'the Linux recipe requires a POSIX host')
    def test_cold_container_failures_do_not_stop_install_or_container_stages(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shutil.copytree(ROOT / 'release', root / 'release')
            bindir = root / 'bin'
            bindir.mkdir()
            docker = bindir / 'docker'
            docker.write_text('#!' + sys.executable + '\nimport json,sys\n'
                              'with open("docker.jsonl", "a") as stream: stream.write(json.dumps(sys.argv[1:]) + "\\n")\n'
                              'sys.exit(23)\n')
            docker.chmod(0o755)
            report = root / 'report.json'
            env = dict(os.environ, PATH=str(bindir) + os.pathsep + os.environ['PATH'],
                       BUILD_IMAGE='cold-image', BIOMCP_RELEASE_COLD='1',
                       BIOMCP_BUILD_REVISION='a' * 40, BIOMCP_RELEASE_REPORT=str(report))
            result = subprocess.run(['sh', 'release/linux.sh', '--label', 'example.owner=cold'],
                                    cwd=root, env=env, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            rows = json.loads(report.read_text())
            self.assertEqual([row['stage'] for row in rows if row['exit'] == 23],
                             ['build', 'install', 'container-build', 'container-save', 'container-cleanup'])
            calls = [json.loads(line) for line in (root / 'docker.jsonl').read_text().splitlines()]
            for call in calls[:2]:
                self.assertIn('--rm', call)
                self.assertNotIn(':/cargo', ' '.join(call))
                self.assertIn('example.owner=cold', call)
            self.assertIn('CARGO_HOME=/tmp/release-cargo', calls[0][-1])
            self.assertFalse((root / '.release-cache').exists())


if __name__ == '__main__':
    unittest.main()
