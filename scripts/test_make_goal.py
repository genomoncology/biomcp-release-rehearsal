"""Run targeted goals through the public shell launchers in an isolated tree."""
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class MakeGoalTests(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory(prefix="make goal's ")
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        for folder in ('scripts', 'sdlc', 'old-python', 'fake-ssh'):
            (self.root / folder).mkdir()
        for name in ('make_goal.sh', 'check_usage_tests.sh'):
            shutil.copy2(ROOT / 'scripts' / name, self.root / 'scripts' / name)
        self.config = json.loads((ROOT / 'sdlc/pm.json').read_text())
        self.write_config()
        self.env = {key: value for key, value in os.environ.items() if key not in {
            'MAKEFLAGS', 'MFLAGS', 'MAKELEVEL', 'MAKEOVERRIDES',
        }}
        self.env['PATH'] = str(self.root / 'old-python') + os.pathsep + self.env['PATH']
        # Model the old Python: it can bootstrap JSON, but cannot run tomllib checks.
        self.executable('old-python/python3', '#!/bin/sh\n'
                        'if [ "$1" != "-" ]; then echo "old Python lacks tomllib" >&2; exit 91; fi\n'
                        'exec ' + shlex.quote(sys.executable) + ' "$@"\n')
        self.executable('fake-ssh/ssh', '#!/bin/sh\necho isolated-ssh\n')
        (self.root / 'Makefile').write_text(
            'targeted-check docs-check:\n\tpython3 probe.py\nssh-check:\n\tssh\n')
        (self.root / 'probe.py').write_text('''import json, os, pathlib, shutil, subprocess, sys, tomllib
assert tomllib.loads('answer = 42')['answer'] == 42
print(json.dumps({'python': str(pathlib.Path(sys.executable).resolve()),
                  'sccache': shutil.which('sccache'),
                  'rust': subprocess.check_output(['rustc', '--version'], text=True).strip(),
                  'toolchain': os.environ['RUSTUP_TOOLCHAIN']}), flush=True)
os.environ['PATH'] = str(pathlib.Path('fake-ssh').resolve()) + os.pathsep + os.environ['PATH']
subprocess.run(['make', '--no-print-directory', 'ssh-check'], check=True)
''')

    def executable(self, name, content):
        path = self.root / name
        path.write_text(content)
        path.chmod(0o755)

    def write_config(self):
        (self.root / 'sdlc/pm.json').write_text(json.dumps(self.config))

    def run_launcher(self, script, goal):
        return subprocess.run(['sh', str(self.root / 'scripts' / script), goal],
                              cwd=self.root.parent, env=self.env, capture_output=True, text=True)

    def test_targeted_and_docs_goals_reuse_pins_and_preserve_child_path(self):
        old = subprocess.run(['python3', '-c', 'import tomllib'], env=self.env,
                             capture_output=True, text=True)
        self.assertEqual(old.returncode, 91)
        pinned_python = str(Path(shutil.which('python3.13')).resolve())
        inherited_cache = shutil.which('sccache')
        self.assertIsNotNone(inherited_cache, 'fixture needs the inherited compiler cache')
        for script, goal in [('make_goal.sh', 'targeted-check'), ('check_usage_tests.sh', 'docs')]:
            with self.subTest(script=script):
                result = self.run_launcher(script, goal)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                record = next(json.loads(line) for line in result.stdout.splitlines()
                              if line.startswith('{'))
                self.assertEqual(record['python'], pinned_python)
                self.assertEqual(record['sccache'], inherited_cache)
                self.assertTrue(record['rust'].startswith('rustc ' + record['toolchain'] + ' '), record)
                self.assertIn('isolated-ssh', result.stdout)

    def test_unknown_goal_is_literal_and_make_reports_it(self):
        goal = 'missing; touch escaped'
        result = self.run_launcher('make_goal.sh', goal)
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn('No rule to make target', result.stderr)
        self.assertIn(goal, result.stderr)
        self.assertFalse((self.root / 'escaped').exists())

    def test_unsupported_gate_fails_before_running_commands(self):
        for gate in [None, 'touch escaped', 'make landing-gate && make landing-gate',
                     'make landing-gate; touch escaped']:
            with self.subTest(gate=gate):
                self.config['ticketsGate'] = gate
                self.write_config()
                result = self.run_launcher('make_goal.sh', 'targeted-check')
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('unsupported ticketsGate', result.stderr)
                self.assertFalse((self.root / 'escaped').exists())

    def test_options_and_assignments_are_not_goals(self):
        for goal in ['', '--eval=bad', 'PATH=bad']:
            with self.subTest(goal=goal):
                result = self.run_launcher('make_goal.sh', goal)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('expected one Make goal', result.stderr)


if __name__ == '__main__':
    unittest.main()
