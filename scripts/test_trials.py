"""Selection refuses names before preparation and keeps target and directory."""
from pathlib import Path
import json
import os
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from trials import run_selected

ROOT = Path(__file__).resolve().parents[1]
CARGO = '''#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
with open(os.environ['COMMAND_LOG'], 'a') as log:
    log.write(json.dumps(sys.argv[1:]) + '\\n')
if sys.argv[1:2] == ['vendor']:
    sys.exit(0)
target = sys.argv[sys.argv.index('--test') + 1] if '--test' in sys.argv else 'biomcp'
print(json.dumps({'reason':'compiler-artifact', 'target':{'name':target},
                  'executable':os.environ['FAKE_RUNNER']}))
'''
RUNNER = '''#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
if '--list' in sys.argv:
    if os.environ.get('DISCOVERY_EXIT'):
        print('discovery stdout context')
        print('cases: malformed YAML', file=sys.stderr)
        sys.exit(int(os.environ['DISCOVERY_EXIT']))
    print(os.environ.get('TRIAL_CATALOG', 'literal/case.yaml: test'))
else:
    print(json.dumps({'arguments':sys.argv[1:], 'cwd':str(Path.cwd()),
                      'product':os.environ.get('BIOMCP_TEST_EXECUTABLE')}))
    sys.exit(int(os.environ.get('TRIAL_EXIT', '0')))
'''


class FocusedCommand(unittest.TestCase):
    def test_command_validates_then_prepares_and_preserves_runner_exit(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            for name, source in [('cargo', CARGO), ('runner', RUNNER)]:
                path = directory / name
                path.write_text(source)
                path.chmod(0o755)
            log = directory / 'commands'
            environment = dict(os.environ, PATH=str(directory) + os.pathsep + os.environ['PATH'],
                               COMMAND_LOG=str(log), FAKE_RUNNER=str(directory / 'runner'))
            command = [sys.executable, str(ROOT / 'scripts/trials.py')]
            for names, builds in [([], 0), (['unknown'], 1), (['literal/case.yaml'], 2)]:
                for code in ([0, 37] if builds == 2 else [0]):
                    with self.subTest(names=names, code=code):
                        log.unlink(missing_ok=True)
                        result = subprocess.run([*command, *names], cwd=directory,
                                                env=dict(environment, TRIAL_EXIT=str(code)),
                                                capture_output=True, text=True)
                        calls = [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []
                        vendors = [call for call in calls if call[:1] == ['vendor']]
                        self.assertEqual(len(vendors), int(builds > 0), result.stderr)
                        if vendors:
                            self.assertEqual(vendors[0][:2], ['vendor', '--locked'])
                        self.assertEqual(len(calls) - len(vendors), builds, result.stderr)
                        if builds == 2:
                            self.assertEqual(result.returncode, code, result.stderr)
                            output = json.loads(result.stdout)
                            self.assertEqual(output['arguments'], ['literal/case.yaml', '--exact'])
                            self.assertEqual(output['cwd'], str(ROOT / 'crates/biomcp'))
                            self.assertEqual(output['product'], str(directory / 'runner'))
                        else:
                            self.assertNotEqual(result.returncode, 0)
            log.unlink(missing_ok=True)
            result = subprocess.run([*command, 'literal/case.yaml'], cwd=directory,
                                    env=dict(environment, DISCOVERY_EXIT='29'),
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 29)
            self.assertIn('discovery stdout context', result.stdout)
            self.assertIn('cases: malformed YAML', result.stderr)
            calls = [json.loads(line) for line in log.read_text().splitlines()]
            self.assertEqual(len(calls), 2)
            self.assertEqual(calls[0][:2], ['vendor', '--locked'])
            self.assertEqual(calls[1][:1], ['test'])

    def test_case_file_cli_uses_catalog_before_building_or_running(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            runner = directory / 'runner'
            runner.write_text(RUNNER.replace('#!/usr/bin/env python3', '#!' + sys.executable, 1))
            runner.chmod(0o755)
            command = [sys.executable, str(ROOT / 'scripts/trials.py'),
                       '--runner', str(runner), '--executable', str(runner), '--installed']
            environment = dict(os.environ, PATH=str(directory),
                               TRIAL_CATALOG='markdown-view/view-empty/empty-section.yaml: test')
            # Only the runner's absolute Python interpreter is available; no Cargo.
            for path in ['view-empty/empty-section.yaml', 'view-empty/empty-section',
                         'unknown.yaml']:
                with self.subTest(path=path):
                    result = subprocess.run([*command, '--case-file', path], cwd=directory,
                                            env=environment, capture_output=True, text=True)
                    if path.endswith('empty-section.yaml'):
                        self.assertEqual(result.returncode, 0, result.stderr)
                        output = json.loads(result.stdout)
                        self.assertEqual(output['arguments'],
                            ['markdown-view/view-empty/empty-section.yaml', '--exact'])
                        self.assertNotEqual(output['cwd'], str(ROOT / 'crates/biomcp'))
                    else:
                        self.assertNotEqual(result.returncode, 0)
                        self.assertIn('markdown-view/view-empty/empty-section.yaml', result.stderr)
                        self.assertEqual(result.stdout, '')

    def test_discovery_preserves_rust_case_and_custom_trial_modes(self):
        from trials import main
        for options in [['--target', 'tools', '--proofs'], ['--cases', str(ROOT / 'tests/cases')]]:
            with self.subTest(options=options), patch('sys.argv', ['trials.py', *options, '--list']), \
                    patch('trials.cargo', return_value={'executable': '/runner'}) as build, \
                    patch('trials.subprocess.run') as run, patch('trials.os.chdir'):
                main()
                self.assertEqual(build.call_count, 1)
                expected = ['--proofs'] if '--proofs' in options else ['--cases', str(ROOT / 'tests/cases')]
                self.assertEqual(run.call_args.args[0], ['/runner', *expected, '--list'])


class TrialSelection(unittest.TestCase):
    def test_exact_names_share_discovery_target_and_directory(self):
        with tempfile.TemporaryDirectory() as temporary:
            cwd = Path(temporary)
            target = cwd / "cases"
            target.touch()
            listing = subprocess.CompletedProcess([], 0,
                "mcp-parity/bounded-framing: test\nschema-discovery/case.yaml: test\n", "")
            prepared = []
            with patch("trials.subprocess.run", return_value=listing) as run:
                def prepare():
                    prepared.append(run.call_count)
                run_selected([str(target)], cwd, ["mcp-parity/bounded-framing"], prepare)
                self.assertEqual(prepared, [1])
                self.assertEqual(run.call_args_list[0].args[0], [str(target), "--list"])
                self.assertEqual(run.call_args_list[1].args[0],
                    [str(target), "mcp-parity/bounded-framing", "--exact"])
                self.assertTrue(all(call.kwargs["cwd"] == cwd for call in run.call_args_list))


    def test_target_mode_stays_with_the_exact_selection(self):
        with tempfile.TemporaryDirectory() as temporary:
            cwd = Path(temporary)
            target = cwd / "tools"
            target.touch()
            listing = subprocess.CompletedProcess([], 0,
                "schema-discovery/persistent-native-evidence: test\n", "")
            with patch("trials.subprocess.run", return_value=listing) as run:
                run_selected([str(target), "--proofs"], cwd,
                             ["schema-discovery/persistent-native-evidence"], lambda: None)
                self.assertEqual(run.call_args_list[0].args[0],
                                 [str(target), "--proofs", "--list"])
                self.assertEqual(run.call_args_list[1].args[0],
                    [str(target), "--proofs", "schema-discovery/persistent-native-evidence", "--exact"])

    def test_ambiguous_case_file_refuses_before_preparation(self):
        with tempfile.TemporaryDirectory() as temporary:
            cwd = Path(temporary)
            target = cwd / 'cases'
            target.touch()
            catalog = 'first/dir/case.yaml: test\nsecond/dir/case.yaml: test\n'
            with patch('trials.subprocess.run', return_value=
                       subprocess.CompletedProcess([], 0, catalog)) as run:
                def prepare():
                    self.fail('ambiguous file reached product preparation')
                with self.assertRaisesRegex(ValueError,
                        'found 2.*first/dir/case.yaml, second/dir/case.yaml'):
                    run_selected([str(target)], cwd, [], prepare,
                                 case_files=['dir/case.yaml'])
                self.assertEqual(run.call_count, 1)

    def test_unknown_and_empty_names_refuse_before_preparation(self):
        with tempfile.TemporaryDirectory() as temporary:
            cwd = Path(temporary)
            target = cwd / "cases"
            target.touch()
            listing = subprocess.CompletedProcess([], 0, "mcp-parity/bounded-framing: test\n", "")
            for names in [[], ["mcp-parity"], ["schema-discovery/rejects-bad-cases"],
                          ["runner::protocol::bounded-framing"], ["bounded-framing"]]:
                with self.subTest(names=names), patch("trials.subprocess.run", return_value=listing) as run:
                    def prepare():
                        self.fail("invalid selection reached build preparation")
                    with self.assertRaises(ValueError):
                        run_selected([str(target)], cwd, names, prepare)
                    self.assertEqual(run.call_count, 1)


if __name__ == "__main__":
    unittest.main()
