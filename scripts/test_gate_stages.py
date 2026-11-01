"""Check stage diagnostics and incremental secret history against real Git."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from release.secrets import inputs

ROOT = Path(__file__).resolve().parents[1]


class GateStages(unittest.TestCase):
    def test_stage_reports_failure_and_stops(self):
        result = subprocess.run([sys.executable, str(ROOT / 'scripts/timed_stage.py'),
                                 'fixture', sys.executable, '-c', 'raise SystemExit(7)'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 7)
        self.assertRegex(result.stdout, r'gate stage=fixture elapsed_seconds=\d+\.\d+ exit=7')

    def test_pre_job_reports_all_cheap_failures_and_stops_gate(self):
        with tempfile.TemporaryDirectory() as temporary:
            tools = Path(temporary)
            cargo = tools / 'cargo'
            cargo.write_text('#!/bin/sh\nprintf "cargo %s\\n" "$*"\nexit 7\n')
            cargo.chmod(0o755)
            env = {**os.environ, 'PATH': str(tools) + os.pathsep + os.environ['PATH']}
            for key in ('MAKEFLAGS', 'MFLAGS', 'MAKELEVEL', 'MAKEOVERRIDES'):
                env.pop(key, None)
            result = subprocess.run([sys.executable, str(ROOT / 'scripts/landing_gate.py')],
                                    cwd=ROOT, env=env, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('cargo fmt --all -- --check', result.stdout)
            self.assertIn('cargo clippy --keep-going --workspace', result.stdout)
            self.assertIn('cargo clippy --keep-going --locked -p biomcp --test tools', result.stdout)
            self.assertIn('plain-cli-build', result.stderr)
            self.assertIn('gate stage=pre-job', result.stdout)
            self.assertNotIn('gate stage=workflow-lint', result.stdout)

    def test_cheap_check_parses_tracked_shell_scripts_by_shebang(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'scripts').mkdir()
            for name in ('Makefile', 'scripts/check_shell.py'):
                (root / name).write_bytes((ROOT / name).read_bytes())
            subprocess.run(['git', 'init', '-q', str(root)], check=True)
            # Neither the extension nor executable permission selects the file.
            script = root / 'release fixture.txt'
            script.write_text('#!/bin/sh\nif true; then\n  :\nfi\n')
            (root / 'untracked.sh').write_text('#!/bin/sh\nif\n')
            (root / 'other.sh').write_text('#!/bin/bash\nif\n')
            subprocess.run(['git', '-C', str(root), 'add', 'release fixture.txt', 'other.sh'], check=True)
            goals = ('fmt', 'unused-dependencies', 'check-no-default-features',
                     'lint-workspace', 'lint-tools', 'ratchets')
            # check-cheap invokes a recursive make; pass omissions to that child.
            command = ['make', '--no-print-directory', 'check-cheap',
                       'MAKE=make ' + ' '.join('-o ' + goal for goal in goals)]
            for broken in (False, True):
                if broken:
                    script.write_text('#!/bin/sh\nif true; then\n')
                result = subprocess.run(command, cwd=root, capture_output=True, text=True)
                self.assertEqual(result.returncode == 0, not broken, result.stdout + result.stderr)
                if broken:
                    self.assertIn('release fixture.txt', result.stderr)

    def test_coverage_reuses_completed_gate_prerequisites(self):
        result = subprocess.run(['make', '--no-print-directory', '-n',
                                 '-o', 'docs-check', '-o', 'library-tests', 'coverage'],
                                cwd=ROOT, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn('reference_matches_grammar', result.stdout)
        self.assertNotIn('cargo test --workspace --exclude biomcp', result.stdout)
        self.assertIn('scripts/coverage.sh run', result.stdout)
        self.assertIn('research-search-fetch-parity', result.stdout)

    def test_release_workflow_retains_complete_history_scan(self):
        workflow = (ROOT / '.github/workflows/release.yml').read_text()
        self.assertIn('run: make secrets-full\n', workflow)
        self.assertNotIn('run: make secrets\n', workflow)
        result = subprocess.run(['make', '--no-print-directory', '-n', 'secrets-full'],
                                cwd=ROOT, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('release/secrets.py history .', result.stdout)
        self.assertNotIn('--since', result.stdout)

    def test_incremental_scan_keeps_removed_and_merge_blobs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            env = {'PATH': os.defpath, 'HOME': temporary, 'GIT_CONFIG_NOSYSTEM': '1'}
            def git(*args):
                return subprocess.check_output(['git', '-C', str(root), *args], env=env,
                                               stderr=subprocess.DEVNULL).decode().strip()
            git('init', '-q')
            git('config', 'user.name', 'Fixture')
            git('config', 'user.email', 'fixture@example.invalid')
            (root / 'old').write_text('already screened')
            git('add', '.'); git('commit', '-qm', 'Base')
            base = git('rev-parse', 'HEAD')
            git('checkout', '-qb', 'side')
            (root / 'removed').write_text('new historical content')
            git('add', '.'); git('commit', '-qm', 'Add')
            git('rm', 'removed'); git('commit', '-qm', 'Remove')
            git('checkout', '-qb', 'candidate', base)
            (root / 'main').write_text('new main content')
            git('add', '.'); git('commit', '-qm', 'Main')
            unrelated = git('rev-parse', 'HEAD')
            git('merge', '--no-ff', '-qm', 'Merge', 'side')
            rows = dict(inputs('history', root, env, base))
            self.assertEqual(set(rows), {'removed', 'main'})
            self.assertEqual(rows['removed'], b'new historical content')
            git('checkout', '-q', 'side')
            with self.assertRaises(subprocess.CalledProcessError):
                list(inputs('history', root, env, unrelated))
            with self.assertRaises(ValueError):
                list(inputs('history', root, env, 'HEAD'))


if __name__ == '__main__':
    unittest.main()
