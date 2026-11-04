"""Exercise Linux release container arguments through the Docker boundary."""
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile
import unittest

from release.smoke import SCHEMA_ID, environment

ROOT = Path(__file__).resolve().parents[1]
REPORT = [
    {"artifact": "wheel", "result": "pass", "checks": ["candidate-bytes", "schemas"]},
    {"artifact": "homebrew-formula", "result": "deferred", "reason": "Homebrew is unavailable"},
]
GATE = r'''import argparse, json, pathlib
parser = argparse.ArgumentParser()
parser.add_argument('--scratch', type=pathlib.Path)
parser.add_argument('--report', type=pathlib.Path)
args, _ = parser.parse_known_args()
report = json.dumps(ROWS, indent=2) + '\n'
(args.report or args.scratch / 'report.json').write_text(report)
print('installed checks completed')
print(report, end='')
print('gate complete')
'''
TOOL = r'''import json, os, sys
excluded = 'live-source-checks/retained-drift-witnesses'
if sys.argv[1:] == ['--proof-metadata']:
    print(json.dumps([{'name': 'runtime/one', 'requires_repository_python': False},
                      {'name': 'runtime/two', 'requires_repository_python': False},
                      {'name': excluded, 'requires_repository_python': True}]))
else:
    assert sys.argv[1:5] == ['--proofs', '--exact', '--skip', excluded], sys.argv
    assert sys.argv[5:] == ['--test-threads', '2'], sys.argv
    defect = os.environ.get('PROOF_DEFECT')
    passed = 0 if defect == 'zero' else 1 if defect == 'missing' else 2
    filtered = 2 if defect == 'extra-filter' else 1
    print(f'test result: ok. {passed} passed; 0 failed; 0 ignored; 0 measured; {filtered} filtered out')
'''
DOCKER = r'''import hashlib, io, json, os, pathlib, shutil, subprocess, sys, tarfile, tempfile
args = sys.argv[1:]
with open(os.environ['DOCKER_RECORD'], 'a') as stream:
    stream.write(json.dumps(args) + '\n')
if args[0] == 'save':
    pathlib.Path(args[args.index('-o') + 1]).write_bytes(b'container archive')
if args[0] != 'run':
    sys.exit(0)
if '/bin/bash' in args:
    user = home = None
    for index, value in enumerate(args):
        if value in ('--user', '-u'):
            user = args[index + 1]
        elif value.startswith('--user='):
            user = value.split('=', 1)[1]
        elif value in ('-e', '--env') and args[index + 1].startswith('HOME='):
            home = args[index + 1].split('=', 1)[1]
        elif value.startswith('--env=HOME='):
            home = value.split('=', 2)[2]
    assert user == f'{os.getuid()}:{os.getgid()}', user
    assert home == '/tmp', home
    if '/artifacts:ro' not in ' '.join(args):
        # Docker must receive a caller-owned output directory before it creates files.
        output = pathlib.Path('dist')
        assert output.is_dir(), 'caller must create dist before the first bind mount'
        assert output.stat().st_uid == os.getuid()
        pathlib.Path('dist/release.json').write_text('{"version":"1.0.0"}')
        with tarfile.open('dist/biomcp-linux-x86_64.tar.gz', 'w:gz') as archive:
            info = tarfile.TarInfo('biomcp')
            info.size = 9
            archive.addfile(info, io.BytesIO(b'candidate'))
            for name in ('LICENSE', 'THIRD-PARTY-NOTICES.txt'):
                entry = tarfile.TarInfo(name)
                entry.size = len(b'fixture notice')
                archive.addfile(entry, io.BytesIO(b'fixture notice'))
        pathlib.Path('target/debug').mkdir(parents=True)
        pathlib.Path('target/debug/container-output').touch()
        pathlib.Path('proof/fixtures').mkdir(parents=True)
        for name in ('case-runner', 'tool-runner'):
            runner = pathlib.Path('proof', name)
            runner.write_text('#!' + sys.executable + '\n' + os.environ['TOOL_FIXTURE'] if name == 'tool-runner' else
                              '#!/bin/sh\necho "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"\n')
            runner.chmod(0o755)
        gate = pathlib.Path('proof/tools/release/gate.py')
        gate.parent.mkdir(parents=True)
        gate.write_text(os.environ['GATE_FIXTURE'])
        shutil.copyfile('release/toolchain.py', gate.parent / 'toolchain.py')
        shutil.copyfile('release/toolchain.json', gate.parent / 'toolchain.json')
    else:
        # Execute the installation shell with mounted paths and disposable storage.
        with tempfile.TemporaryDirectory() as container:
            script = args[-1].replace('/tmp/check', container + '/check')
            mounts = [args[index + 1] for index, value in enumerate(args) if value == '-v']
            for mount in mounts:
                host, target, *options = mount.split(':')
                script = script.replace(target, host)
            subprocess.run(['/bin/bash', '-c', script], check=True)
        assert not pathlib.Path(container).exists()
elif '/usr/bin/sha256sum' in args:
    print(hashlib.sha256(b'candidate').hexdigest(), '/usr/local/bin/biomcp')
elif '/bin/cat' in args:
    sys.stdout.buffer.write(pathlib.Path('container', pathlib.Path(args[-1]).name).read_bytes())
elif args[-2] == 'schema':
    print(json.dumps({'$id': args[-1]}))
elif args[-1] == 'help':
    print('{"result":{"commands":["get"]}}')
elif '--entrypoint' in args and args[args.index('--entrypoint') + 1] == '/bin/sh':
    assert 'BIOMCP_TEST_FIXTURES=/tmp/check/fixtures' in args, args
    with tempfile.TemporaryDirectory() as root:
        start = args.index('-ec') + 1
        command = [arg.replace('/tmp/check', root + '/check').replace('/proof/', str(pathlib.Path('container').resolve()) + '/')
                   for arg in args[start:]]
        subprocess.run(['/bin/sh', '-ec', *command], check=True)
elif args[-1] == 'serve':
    for frame in sys.stdin:
        value = json.loads(frame)
        if 'id' not in value:
            continue
        result = ({'serverInfo': {'version': '1.0.0'}} if value['id'] == 1 else
                  {'tools': [{'name': 'biomcp'}]} if value['id'] == 2 else {})
        print(json.dumps({'jsonrpc': '2.0', 'id': value['id'], 'result': result}))
'''


@unittest.skipIf(sys.platform == 'win32', 'the release script requires a POSIX shell')
class LinuxContainers(unittest.TestCase):
    def run_release(self, extra, defect=''):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'release').mkdir()
            for name in ('linux.sh', 'container_check.py', 'smoke.py', 'Dockerfile', 'toolchain.py', 'toolchain.json', 'source_revision.py'):
                shutil.copyfile(ROOT / 'release' / name, root / 'release' / name)
            shutil.copyfile(ROOT / 'Makefile', root / 'Makefile')
            bindir = root / 'bin'
            bindir.mkdir()
            docker = bindir / 'docker'
            docker.write_text('#!' + sys.executable + '\n' + DOCKER)
            docker.chmod(0o755)
            (bindir / 'python').symlink_to(sys.executable)
            subprocess.run(['git', 'init', '-q', str(root)], check=True)
            subprocess.run(['git', '-C', str(root), '-c', 'user.name=Fixture',
                            '-c', 'user.email=fixture@example.org', 'commit',
                            '--allow-empty', '-qm', 'Initialize fixture'], check=True)
            record = root / 'docker.jsonl'
            env = environment(root)
            env.update(PATH=str(bindir) + os.pathsep + os.environ['PATH'],
                       BUILD_IMAGE='build-image', DOCKER_RECORD=str(record), BIOMCP_BUILD_REVISION='a' * 40,
                       TOOL_FIXTURE=TOOL, PROOF_DEFECT=defect,
                       GATE_FIXTURE='ROWS = ' + repr(REPORT) + '\n' + GATE)
            result = subprocess.run(['make', 'release-linux', 'CONTAINER_ARGS=' + shlex.join(extra).replace('$', '$$')],
                                    cwd=root, env=env, capture_output=True, text=True)
            if defect:
                self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                diagnostic = {'zero': 'container runner did not execute any proof',
                              'extra-filter': 'installed runner did not report the complete declared selection',
                              'missing': 'container proof count differs from declared selection'}[defect]
                self.assertIn(diagnostic, result.stdout + result.stderr)
                return None
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            containers = [json.loads(line) for line in result.stdout.splitlines()
                          if line.startswith('{') and json.loads(line).get('artifact') == 'docker']
            self.assertEqual(len(containers), 1, result.stdout)
            self.assertEqual(containers[0]['result'], 'pass')
            self.assertIn('runner-proofs', containers[0]['checks'])
            self.assertIn('all-cases-and-trials', containers[0]['checks'])
            self.assertEqual(containers[0]['trials'], 2)
            self.assertEqual(containers[0]['proofs'], {
                'ran': 2, 'excluded': ['live-source-checks/retained-drift-witnesses'], 'excluded_count': 1})
            self.assertFalse((root / 'injected').exists())
            self.assertEqual((root / 'dist/biomcp-container-amd64.tar').read_bytes(),
                             b'container archive')
            for output in ('dist', 'target/debug'):
                (root / output / 'host-output').write_text('caller can write')
            self.assertEqual([str(path.relative_to(root)) for path in root.rglob('*')
                              if (path.lstat().st_uid, path.lstat().st_gid) != (os.getuid(), os.getgid())], [])
            # The Docker process removed its temporary installation tree before returning.
            report = root / 'dist/install-report.json'
            self.assertEqual(json.loads(report.read_text()), REPORT)
            self.assertGreater(len(report.read_text().splitlines()), 2)
            return [json.loads(line) for line in record.read_text().splitlines()]

    def test_release_preserves_caller_ownership_and_forwards_container_arguments(self):
        extra = ['--label', 'example.owner=release check', '--label',
                 'example.literal=$(touch injected);*', '--network', 'none',
                 '--user=0:0', '--env=HOME=/root']
        baseline = self.run_release([])
        supplied = self.run_release(extra)
        self.assertEqual(len(baseline), 14)
        self.assertEqual(len(supplied), len(baseline))
        self.assertEqual(sum(row[0] == 'run' for row in baseline), 11)
        pins = json.loads((ROOT / 'release/toolchain.json').read_text())
        build, = [row for row in baseline if row[0] == 'build']
        self.assertIn('runtime_image=' + pins['images']['runtime'], build)
        self.assertIn('runtime_packages=' + ' '.join(pins['runtime-packages']), build)
        for plain, labeled in zip(baseline, supplied):
            with self.subTest(command=plain):
                self.assertNotIn('--label', plain)
                if plain[0] == 'run':
                    start = labeled.index(extra[0])
                    self.assertEqual(labeled[start:start + len(extra)], extra)
                    labeled = labeled[:start] + labeled[start + len(extra):]
                # Temporary source paths and the checkout revision vary per run.
                normalize = lambda row: [arg.replace(arg.split(':/')[0], '<mount>')
                                        if arg.startswith('/') and ':/' in arg else
                                        '<revision>' if arg.startswith('BIOMCP_BUILD_REVISION=') else arg
                                        for arg in row]
                self.assertEqual(normalize(labeled), normalize(plain))
        self.assertEqual([row[-1] for row in baseline[3:12]],
                         ['/usr/local/bin/biomcp', '/usr/share/doc/biomcp/LICENSE',
                          '/usr/share/doc/biomcp/THIRD-PARTY-NOTICES.txt', SCHEMA_ID, 'help', 'serve',
                          '--proof-metadata', '2', '2'])

    def test_container_rejects_zero_missing_or_extra_filtered_proofs(self):
        for defect in ('zero', 'missing', 'extra-filter'):
            with self.subTest(defect=defect):
                self.run_release([], defect)


if __name__ == '__main__':
    unittest.main()
