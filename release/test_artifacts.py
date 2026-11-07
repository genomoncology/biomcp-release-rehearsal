"""Outside-in checks on the local artifact producer and installed launcher."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile

from release.smoke import environment
from release.test_licenses import wheel_fixture

ROOT = Path(__file__).resolve().parents[1]
CARGO = r'''import json, os, pathlib, sys
args = sys.argv[1:]
root = pathlib.Path(__file__).parent
with (root / 'cargo.jsonl').open('a') as stream:
    stream.write(json.dumps(args) + '\n')
if args[0] in ('build', 'test'):
    assert '--locked' in args and args[args.index('--profile') + 1] == 'release', args
    name = args[args.index('--test') + 1] if args[0] == 'test' else 'biomcp'
    home = pathlib.Path(os.environ['CARGO_HOME'])
    with (root / 'configs.jsonl').open('a') as stream:
        stream.write(json.dumps([name, (home / 'config.toml').read_text()]) + '\n')
    embed = (root / 'embed').read_text().replace('{stage}', str(home.parent)) if (root / 'embed').exists() else ''
    if embed:
        with (root / name).open('a') as stream:
            stream.write('# ' + embed + '\n')
    assert ('--no-run' in args) == (args[0] == 'test'), args
    assert name in ('biomcp', 'cases', 'tools'), args
    print(json.dumps({'reason': 'compiler-artifact', 'target': {'name': name},
                      'executable': str(root / name),
                      'profile': {'debug_assertions': False, 'opt_level': '3'}}))
elif args[:2] == ['vendor', '--locked'] and len(args) == 3:
    pass
elif args == ['metadata', '--locked', '--format-version', '1', '--no-deps']:
    print(json.dumps({'packages': [{'name': 'biomcp', 'version': '1.0.0'}]}))
elif args == ['about', '--version']:
    print('cargo-about ' + ABOUT_VERSION)
elif args[:2] == ['about', 'generate']:
    pathlib.Path(args[args.index('--output-file') + 1]).write_text(json.dumps({
        'crates': [{}], 'licenses': [{'name': 'Fixture', 'id': 'MIT', 'text': 'Fixture license',
                                     'used_by': [{'crate': {'name': 'fixture', 'version': '1.0'}}]}]}))
else:
    sys.exit('unexpected Cargo command: ' + repr(args))
'''


@unittest.skipIf(os.name == 'nt', 'synthetic Cargo uses a POSIX shebang')
class ReleaseCommands(unittest.TestCase):
    def fixtures(self, root):
        tools = root / 'bin'
        tools.mkdir()
        version = json.loads((ROOT / 'release/toolchain.json').read_text())['cargo-about']
        cargo = tools / 'cargo'
        cargo.write_text(f'#!{sys.executable}\nABOUT_VERSION = {version!r}\n' + CARGO)
        cargo.chmod(0o755)
        candidate = tools / 'biomcp'
        descriptions = json.loads((ROOT / 'manifest.json').read_text())['tools']
        candidate.write_text(f'#!{sys.executable}\nimport json, sys\nTOOLS = {descriptions!r}\n' +
                             "import pathlib\n" +
                             "if sys.argv[1:] == ['skill', 'install', 'guide']:\n" +
                             "    path = pathlib.Path('guide/skills/biomcp/SKILL.md')\n" +
                             "    path.parent.mkdir(parents=True)\n" +
                             "    path.write_text('Fixture guide\\n')\n" +
                             "    sys.exit()\n" +
                             "assert sys.argv[1:] == ['serve']\n" +
                             "for frame in sys.stdin:\n" +
                             "    value = json.loads(frame)\n" +
                             "    if 'id' in value:\n" +
                             "        print(json.dumps({'jsonrpc': '2.0', 'id': value['id'], 'result': " +
                             "{'tools': TOOLS} if value['id'] == 2 else {}}))\n")
        candidate.chmod(0o755)
        for name in ('cases', 'tools'):
            (tools / name).write_bytes(('fixture runner ' + name).encode())
        env = environment(root)
        env.update(PATH=str(tools) + os.pathsep + os.defpath, BIOMCP_BUILD_REVISION='a' * 40)
        return tools, env

    def test_build_command_packages_candidate_bytes_and_metadata(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            tools, env = self.fixtures(root)
            output = root / 'dist'
            result = subprocess.run([sys.executable, 'release/artifacts.py', 'build', '--output', str(output)],
                                    cwd=ROOT, env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            record = json.loads((output / 'release.json').read_text())
            calls = [json.loads(line) for line in (tools / 'cargo.jsonl').read_text().splitlines()]
            self.assertEqual(len([call for call in calls if call[:2] == ['vendor', '--locked']]), 1)
            (name, config), = [json.loads(line) for line in (tools / 'configs.jsonl').read_text().splitlines()]
            self.assertEqual(name, 'biomcp')
            import tomllib
            flags = tomllib.loads(config)['target']['cfg(all())']['rustflags']
            self.assertEqual([flag.rsplit('=', 1)[1] for flag in flags], ['cargo', 'biomcp', 'vendor'])
            self.assertTrue(all(flag.startswith('--remap-path-prefix=') for flag in flags))
            self.assertEqual(record['revision'], 'a' * 40)
            self.assertEqual(record['version'], '1.0.0')
            self.assertEqual(record['binary_sha256'], hashlib.sha256((tools / 'biomcp').read_bytes()).hexdigest())
            inspected = subprocess.run([sys.executable, 'release/artifacts.py', 'inspect', str(next(output.glob('*.whl')))],
                                       cwd=ROOT, env=env, capture_output=True, text=True)
            self.assertEqual(inspected.returncode, 0, inspected.stderr)
            self.assertEqual(json.loads(inspected.stdout), record)
            summary = json.loads(result.stdout)
            archive = Path(summary['archive'])
            self.assertTrue(archive.is_file())
            self.assertEqual(archive.with_name(archive.name + '.sha256').read_text().split()[0],
                             hashlib.sha256(archive.read_bytes()).hexdigest())
            with zipfile.ZipFile(next(output.glob('*.mcpb'))) as bundle:
                manifest = json.loads(bundle.read('manifest.json'))
                self.assertEqual(bundle.read(manifest['server']['entry_point']), (tools / 'biomcp').read_bytes())
                self.assertEqual(manifest['tools'], json.loads((ROOT / 'manifest.json').read_text())['tools'])
            self.assertIn(archive.as_uri(), (output / 'biomcp.rb').read_text())
            self.assertEqual(json.loads((output / 'server.json').read_text())['version'], '1.0.0')

    def test_build_refuses_a_binary_that_names_a_build_folder(self):
        for embed in ('{root}/crates/biomcp/src/main.rs', '{root_windows}\\crates\\biomcp\\src\\main.rs',
                      '{{stage}}/sources/ring-0.17/src/lib.rs', '{tools}/deps/biomcp',
                      '{home}/registry/src/index/ring-0.17/src/lib.rs'):
            with self.subTest(embed=embed), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                tools, env = self.fixtures(root)
                env['CARGO_HOME'] = str(root / 'cargo')
                (tools / 'embed').write_text(embed.format(
                    root=ROOT, root_windows=str(ROOT).replace('/', '\\'), tools=tools, home=root / 'cargo'))
                output = root / 'dist'
                result = subprocess.run([sys.executable, 'release/artifacts.py', 'build', '--output', str(output)],
                                        cwd=ROOT, env=env, capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertIn('Release binary holds build paths', result.stderr)
                self.assertFalse((output / 'release.json').exists())

    def test_unprepared_gate_copies_public_fixtures_and_both_runners(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            tools, env = self.fixtures(root)
            output = root / 'dist'
            output.mkdir()
            wheel_fixture(output, 'linux_x86_64')
            scratch = root / 'install'
            result = subprocess.run([sys.executable, 'release/gate.py', '--prepare-only',
                                     '--artifacts', str(output), '--scratch', str(scratch)],
                                    cwd=ROOT, env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            for name, target in (('cases', 'case-runner'), ('tools', 'tool-runner')):
                self.assertEqual((scratch / target).read_bytes(), (tools / name).read_bytes())
            for source in (ROOT / 'tests').rglob('*'):
                if source.is_file():
                    self.assertEqual((scratch / 'fixtures' / source.relative_to(ROOT / 'tests')).read_bytes(), source.read_bytes())
            provenance = Path('sdlc/planning/slices/gene.md')
            self.assertEqual((scratch / 'fixtures/support' / provenance).read_bytes(), (ROOT / provenance).read_bytes())
            for source in (ROOT / 'release').glob('*.py'):
                self.assertEqual((scratch / 'tools/release' / source.name).read_bytes(), source.read_bytes())
            self.assertEqual((scratch / 'tools/LICENSE').read_bytes(), (ROOT / 'LICENSE').read_bytes())
            calls = [json.loads(line) for line in (tools / 'cargo.jsonl').read_text().splitlines()]
            vendors = [call for call in calls if call[:2] == ['vendor', '--locked']]
            self.assertEqual(len(vendors), 1)
            self.assertEqual([call[call.index('--test') + 1] for call in calls if call not in vendors],
                             ['cases', 'tools'])
            self.assertFalse((scratch / 'venv').exists())
            configs = [json.loads(line) for line in (tools / 'configs.jsonl').read_text().splitlines()]
            self.assertEqual([name for name, _ in configs], ['cases', 'tools'])
            self.assertFalse(any('remap-path-prefix' in config for _, config in configs))

    def test_build_keeps_notice_bytes_with_windows_text_newlines(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            _, env = self.fixtures(root)
            output = root / 'dist'
            # Emulate Windows' default text writes while running the real command.
            command = '''import io, runpy, sys
from unittest.mock import patch
native_open = io.open
def windows_open(file, mode='r', buffering=-1, encoding=None, errors=None,
                 newline=None, closefd=True, opener=None):
    if 'w' in mode and 'b' not in mode and newline is None:
        newline = '\\r\\n'
    return native_open(file, mode, buffering, encoding, errors, newline, closefd, opener)
sys.path.insert(0, 'release')
sys.argv = ['release/artifacts.py', 'build', '--output', sys.argv[1]]
with patch('io.open', windows_open):
    runpy.run_path('release/artifacts.py', run_name='__main__')
'''
            result = subprocess.run([sys.executable, '-c', command, str(output)],
                                    cwd=ROOT, env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual((output / 'SKILL.md').read_text(), 'Fixture guide\n')
            notices = (output / 'THIRD-PARTY-NOTICES.txt').read_bytes()
            self.assertTrue(notices.startswith(b'Generated third-party notices\n'))
            with zipfile.ZipFile(next(output.glob('*.whl'))) as wheel:
                self.assertEqual(wheel.read('biomcp_cli-1.0.0.dist-info/THIRD-PARTY-NOTICES.txt'), notices)


class Artifacts(unittest.TestCase):
    def test_debug_wheel_is_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["python3", "release/artifacts.py", "build", "--profile", "dev", "--output", str(root)], check=True)
            result = subprocess.run(["python3", "release/artifacts.py", "inspect", str(next(root.glob("*.whl")))], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("release profile required", result.stderr)

    def test_selected_missing_executable_has_no_build_fallback(self):
        runner = os.environ["BIOMCP_CASE_RUNNER"]
        for args in [
            ["--cases", "tests/cases/gene", "gene-identity/kras.yaml", "--exact"],
            ["mcp-research/research-search-fetch-parity", "--exact"],
        ]:
            result = subprocess.run([runner] + args, env={**os.environ, "BIOMCP_TEST_EXECUTABLE": "/does-not-exist/biomcp"}, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("No such file", result.stdout)
            self.assertIn("1 failed", result.stdout)
            self.assertNotIn("selected no tests", result.stderr)


if __name__ == "__main__":
    unittest.main()
