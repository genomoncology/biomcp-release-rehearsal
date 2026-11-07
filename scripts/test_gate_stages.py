"""Check stage diagnostics and incremental secret history against real Git."""
import json
import os
import shutil
import re
import secrets
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from release.secrets import inputs
from release.smoke import environment

ROOT = Path(__file__).resolve().parents[1]


class GateStages(unittest.TestCase):
    def test_ticket_launchers_submit_configured_gate_and_preserve_exit(self):
        launchers = (('tls_cause_job.sh', 'gate'), ('hgnc_sync_proof_job.sh', 'routine'),
                     ('check_nightly_tests.sh', 'routine'))
        configured = json.loads((ROOT / 'sdlc/pm.json').read_text())['ticketsGate']
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for directory in ('scripts', 'sdlc', 'bin', 'elsewhere'):
                (root / directory).mkdir()
            for name, _ in launchers:
                shutil.copy2(ROOT / 'scripts' / name, root / 'scripts' / name)
            for name in ('python3', 'python3.13'):
                (root / 'bin' / name).symlink_to(sys.executable)
            capture = 'import json, sys\nprint(json.dumps(sys.argv[1:]))\nsys.exit(23)\n'
            for name, body in {'sh': capture, 'make': capture, 'git': '',
                               'uv': 'print(' + repr(str(root / 'bin/python3')) + ')',
                               'cargo': "print('0.9.2')"}.items():
                tool = root / 'bin' / name
                tool.write_text('#!' + sys.executable + '\n' + body)
                tool.chmod(0o755)
            env = {**environment(root), 'PATH': str(root / 'bin') + os.pathsep + os.defpath}
            for command in (configured, "printf 'quoted \"value\"';\nexit 23\n"):
                (root / 'sdlc/pm.json').write_text(json.dumps({'ticketsGate': command}))
                for name, mode in launchers:
                    with self.subTest(launcher=name, command=command):
                        result = subprocess.run(['/bin/sh', str(root / 'scripts' / name), mode],
                                                cwd=root / 'elsewhere', env=env,
                                                capture_output=True, text=True)
                        self.assertEqual(result.returncode, 23, result.stdout + result.stderr)
                        self.assertEqual(json.loads(result.stdout.splitlines()[-1]), ['-c', command])

    def test_stage_reports_failure_and_stops(self):
        result = subprocess.run([sys.executable, str(ROOT / 'scripts/timed_stage.py'),
                                 'fixture', sys.executable, '-c', 'raise SystemExit(7)'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 7)
        self.assertRegex(result.stdout, r'gate stage=fixture elapsed_seconds=\d+\.\d+ exit=7')

    def test_pre_job_reports_all_cheap_failures(self):
        with tempfile.TemporaryDirectory() as temporary:
            tools = Path(temporary)
            cargo = tools / 'cargo'
            cargo.write_text('#!/bin/sh\nprintf "cargo %s\\n" "$*"\nexit 7\n')
            cargo.chmod(0o755)
            env = {**os.environ, 'PATH': str(tools) + os.pathsep + os.environ['PATH']}
            for key in ('MAKEFLAGS', 'MFLAGS', 'MAKELEVEL', 'MAKEOVERRIDES'):
                env.pop(key, None)
            result = subprocess.run([sys.executable, str(ROOT / 'scripts/timed_stage.py'), 'pre-job',
                                     'make', '--no-print-directory', 'pre-job'],
                                    cwd=ROOT, env=env, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('cargo fmt --all -- --check', result.stdout)
            self.assertIn('cargo clippy --keep-going --workspace', result.stdout)
            self.assertIn('cargo clippy --keep-going --locked -p biomcp --test tools', result.stdout)
            self.assertIn('plain-cli-build', result.stderr)
            self.assertIn('gate stage=pre-job', result.stdout)

    def test_gate_runs_its_stages_in_order_and_stops_on_a_failing_stage(self):
        landing = ['rights', 'audit', 'secrets', 'pre-job', 'workflow-lint', 'cases']
        full = ['audit', 'secrets-full', 'pre-job', 'workflow-lint', 'docs-check',
                'tensor-proto-check', 'library-tests', 'coverage']
        with tempfile.TemporaryDirectory() as temporary:
            tools = Path(temporary)
            make = tools / 'make'
            make.write_text('#!/bin/sh\nfor goal; do :; done\n'
                            '[ "$goal" != "$FAIL_GOAL" ] || exit 7\n')
            make.chmod(0o755)
            env = {**os.environ, 'PATH': str(tools) + os.pathsep + os.environ['PATH']}
            for arguments, stages, failing in (([], landing, 'none'), ([], landing, 'cases'),
                                               ([], landing, 'secrets'), ([], landing, 'pre-job'),
                                               (['--full'], full, 'none')):
                with self.subTest(arguments=arguments, failing=failing):
                    result = subprocess.run([sys.executable, 'scripts/landing_gate.py', *arguments],
                                            cwd=ROOT, env={**env, 'FAIL_GOAL': failing},
                                            capture_output=True, text=True)
                    expected = stages[:stages.index(failing) + 1] if failing in stages else stages
                    self.assertEqual(re.findall(r'gate stage=(\S+)', result.stdout), expected)
                    self.assertEqual(result.returncode, 7 if failing in stages else 0)

    def test_cheap_check_parses_tracked_shell_scripts_by_shebang(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'scripts').mkdir()
            for name in ('Makefile', 'scripts/check_shell.py'):
                (root / name).write_bytes((ROOT / name).read_bytes())
            # This fixture exercises shell checks, not the separate docs validator.
            (root / 'scripts/check_docs.py').write_text('')
            subprocess.run(['git', 'init', '-q', str(root)], check=True)
            # Neither the extension nor executable permission selects the file.
            script = root / 'release fixture.txt'
            script.write_text('#!/bin/sh\nif true; then\n  :\nfi\n')
            (root / 'untracked.sh').write_text('#!/bin/sh\nif\n')
            (root / 'other.sh').write_text('#!/bin/bash\nif\n')
            subprocess.run(['git', '-C', str(root), 'add', 'release fixture.txt', 'other.sh'], check=True)
            focused = root / 'scripts/check_focused.sh'
            focused.write_text('#!/bin/sh\nmake check-cheap\n')
            subprocess.run(['git', '-C', str(root), 'add', 'scripts/check_focused.sh'], check=True)
            goals = ('fmt', 'unused-dependencies', 'check-no-default-features',
                     'lint-workspace', 'lint-tools', 'lint-rustdoc', 'ratchets')
            # check-cheap invokes a recursive make; pass omissions to that child.
            command = ['make', '--no-print-directory', 'check-cheap',
                       'MAKE=make ' + ' '.join('-o ' + goal for goal in goals)]
            for broken, partial in ((False, False), (True, False), (False, True)):
                script.write_text('#!/bin/sh\nif true; then\n' + ('' if broken else ':\nfi\n'))
                focused.write_text('#!/bin/sh\nmake ' +
                                   ('fmt lint-tools ratchets' if partial else 'check-cheap') + '\n')
                result = subprocess.run(command, cwd=root, capture_output=True, text=True)
                self.assertEqual(result.returncode == 0, not (broken or partial), result.stdout + result.stderr)
                if broken:
                    self.assertIn('release fixture.txt', result.stderr)
                if partial:
                    self.assertIn('scripts/check_focused.sh:2: use make check-cheap', result.stderr)

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

    def test_release_checkout_scans_complete_candidate_without_unrelated_refs(self):
        workflow = (ROOT / '.github/workflows/release.yml').read_text()
        candidate = workflow.split('\n  candidate:\n', 1)[1].split('\n  qualify:\n', 1)[0]
        checkout = candidate.split('      - uses: actions/checkout@', 1)[1].split('      - name:', 1)[0]
        self.assertIn('ref: ${{ github.sha }}', checkout)
        self.assertIn('persist-credentials: false', checkout)
        self.assertIn('fetch-tags: false', checkout)
        depth = int(re.search(r'fetch-depth: (\d+)', checkout)[1])
        guard = candidate.split('      - name: Check complete candidate history scope\n', 1)[1]
        guard = guard.split('        run: |\n', 1)[1].split('      - ', 1)[0]
        guard = '\n'.join(line[10:] for line in guard.splitlines())
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source, checkout = root / 'source', root / 'checkout'
            source.mkdir(); checkout.mkdir()
            env = {**environment(root), 'PATH': os.defpath, 'GIT_CONFIG_NOSYSTEM': '1'}
            def git(repo, *args):
                return subprocess.check_output(['git', '-C', str(repo), *args], env=env,
                                               stderr=subprocess.DEVNULL).decode().strip()
            def commit(name, content):
                (source / name).write_text(content)
                git(source, 'add', '.')
                git(source, 'commit', '-qm', 'Add fixture')
            git(source, 'init', '-q')
            git(source, 'config', 'user.name', 'Fixture')
            git(source, 'config', 'user.email', 'fixture@example.invalid')
            commit('base', 'clean base')
            base = git(source, 'rev-parse', 'HEAD')
            git(source, 'checkout', '-qb', 'side')
            commit('removed', 'complete merged ancestry')
            removed_blob = git(source, 'rev-parse', 'HEAD:removed')
            git(source, 'rm', 'removed'); git(source, 'commit', '-qm', 'Remove fixture')
            git(source, 'checkout', '-qb', 'candidate', base)
            commit('candidate', 'clean candidate')
            git(source, 'merge', '--no-ff', '-qm', 'Merge fixture', 'side')
            revision = git(source, 'rev-parse', 'HEAD')
            expected_commits = git(source, 'rev-list', '--count', revision)
            expected_objects = set(git(source, 'rev-list', '--objects', '--no-object-names', revision).splitlines())
            git(source, 'checkout', '--orphan', 'old-main')
            git(source, 'rm', '-rf', '.')
            token = 'ghp_' + secrets.token_hex(18)
            commit('credential.txt', 'access_token = ' + token + '\n')
            git(source, 'tag', 'v0.9.2')
            remote_refs = git(source, 'show-ref')
            git(checkout, 'init', '-q')
            git(checkout, 'remote', 'add', 'origin', source.as_uri())
            # Match the pinned checkout action's positive-depth, exact-SHA fetch.
            # Its depth-zero branch fetches every head and tag instead.
            if depth == 0:
                git(checkout, 'fetch', 'origin', '+refs/heads/*:refs/remotes/origin/*', '+refs/tags/*:refs/tags/*')
            else:
                git(checkout, 'fetch', '--no-tags', '--depth=' + str(depth), 'origin', revision)
            git(checkout, 'checkout', '--detach', revision)
            def scope(expected):
                result = subprocess.run(['bash', '-e', '-o', 'pipefail', '-c', guard], cwd=checkout,
                                        env={**env, 'GITHUB_SHA': revision}, capture_output=True, text=True)
                self.assertEqual(result.returncode == 0, expected, result.stdout + result.stderr)
            def screen(expected):
                result = subprocess.run([sys.executable, str(ROOT / 'release/secrets.py'), 'history', str(checkout)],
                                        cwd=ROOT, env=env, capture_output=True, text=True)
                self.assertEqual(result.returncode, expected, 'fixture history scan changed')
                self.assertNotIn(token, result.stdout + result.stderr)
                return result
            scope(True)
            self.assertEqual(git(checkout, 'rev-parse', '--is-shallow-repository'), 'false')
            self.assertEqual(git(checkout, 'rev-list', '--count', '--all', 'HEAD'), expected_commits)
            self.assertEqual(set(git(checkout, 'rev-list', '--objects', '--no-object-names', '--all', 'HEAD').splitlines()), expected_objects)
            self.assertEqual(git(checkout, 'show-ref', '--head'), revision + ' HEAD')
            self.assertEqual(git(checkout, 'cat-file', '-p', removed_blob), 'complete merged ancestry')
            self.assertEqual(git(checkout, 'status', '--porcelain'), '')
            screen(0)
            # Importing an old ref must trip both the scope guard and real scanner.
            git(checkout, 'fetch', '--no-tags', 'origin', '+refs/heads/old-main:refs/remotes/origin/old-main')
            scope(False)
            self.assertIn('credential.txt', screen(1).stdout)
            self.assertEqual(git(source, 'show-ref'), remote_refs)
            self.assertEqual(git(checkout, 'status', '--porcelain'), '')

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
