"""Local release commands refuse to mislabel modified source as a clean commit."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class SourceRevision(unittest.TestCase):
    def test_python_check_imports_leave_release_source_clean(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'source'
            subprocess.run(['git', 'clone', '--quiet', '--shared', str(ROOT), str(root)], check=True)
            env = dict(os.environ, GIT_CONFIG_COUNT='1', GIT_CONFIG_KEY_0='core.excludesFile',
                       GIT_CONFIG_VALUE_0='')
            for name in ('BIOMCP_BUILD_REVISION', 'PYTHONDONTWRITEBYTECODE', 'PYTHONPYCACHEPREFIX'):
                env.pop(name, None)
            subprocess.run([sys.executable, '-c', 'import release.artifacts; import scripts.trials'],
                           cwd=root, env=env, check=True)
            for name in ('release', 'scripts'):
                self.assertTrue(list((root / name / '__pycache__').glob('*.pyc')))
            status = subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=all'],
                                             cwd=root, env=env, text=True)
            self.assertEqual(status, '')
            result = subprocess.run([sys.executable, 'release/source_revision.py'], cwd=root, env=env,
                                    capture_output=True, text=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.strip(), subprocess.check_output(
                ['git', 'rev-parse', 'HEAD'], cwd=root, env=env, text=True).strip())

    def test_local_recipes_refuse_dirty_source_before_build_or_download(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'source'
            subprocess.run(['git', 'clone', '--quiet', '--shared', str(ROOT), str(root)], check=True)
            (root / 'README.md').write_text('modified source\n')
            bindir = Path(directory) / 'bin'
            bindir.mkdir()
            for name in ('cargo', 'docker'):
                command = bindir / name
                command.write_text('#!/bin/sh\necho unexpected-build >&2\nexit 91\n')
                command.chmod(0o755)
            env = dict(os.environ, PATH=str(bindir) + os.pathsep + os.environ['PATH'], BUILD_IMAGE='fixture')
            env.pop('BIOMCP_BUILD_REVISION', None)
            for command in ([sys.executable, 'release/artifacts.py', 'build', '--output', 'dist'],
                            ['sh', 'release/linux.sh'], ['sh', 'release/matrix.sh'],
                            [sys.executable, 'release/rehearse.py', 'cold', '--report', 'dist/report.json']):
                with self.subTest(command=command):
                    result = subprocess.run(command, cwd=root, env=env, capture_output=True, text=True, timeout=30)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn('dirty source tree', result.stderr)
                    self.assertIn(' M README.md', result.stderr)
                    self.assertNotIn('unexpected-build', result.stderr)
            command = [sys.executable, str(root / 'release/source_revision.py'), '--revision', 'a' * 40]
            result = subprocess.run(command, cwd=root, env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.strip(), 'a' * 40)
            result = subprocess.run(command[:-2], cwd=root, env=dict(env, BIOMCP_BUILD_REVISION='b' * 40),
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.strip(), 'b' * 40)

    def test_dirty_diagnostic_limits_paths_and_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'source'
            subprocess.run(['git', 'clone', '--quiet', '--shared', str(ROOT), str(root)], check=True)
            for index in range(30):
                (root / (f'{index:03d}-' + 'x' * 220)).touch()
            env = dict(os.environ)
            env.pop('BIOMCP_BUILD_REVISION', None)
            result = subprocess.run([sys.executable, 'release/source_revision.py'], cwd=root, env=env,
                                    capture_output=True, text=True, timeout=30)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(result.stdout, '')
            self.assertIn('?? 000-', result.stderr)
            self.assertNotIn('?? 029-', result.stderr)
            self.assertIn('(status truncated)', result.stderr)
            self.assertLess(len(result.stderr), 4300)


if __name__ == '__main__':
    unittest.main()
