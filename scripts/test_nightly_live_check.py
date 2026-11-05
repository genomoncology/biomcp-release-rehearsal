"""Exercise scheduled command settings, delivery and scratch ownership offline."""
import fnmatch
import json
from pathlib import Path
import shutil
import unittest

if __package__:
    from . import nightly_test_support as support
else:
    import nightly_test_support as support

ROOT = support.ROOT


class NightlyCommand(support.NightlyHarness, unittest.TestCase):
    def test_missing_destination_fails_before_running_or_creating_scratch(self):
        for name in ('BIOMCP_EVIDENCE_HOST', 'BIOMCP_EVIDENCE_DIR', 'BIOMCP_SCRATCH_DIR'):
            self.env.pop(name, None)
        result = self.run_command()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('supply --host', result.stderr)
        self.assertFalse(self.scratch_root.exists())
        self.assertFalse(self.store.exists())

    def test_make_uses_only_local_settings_from_config_home_or_home(self):
        for xdg in (True, False):
            with self.subTest(xdg=xdg):
                config = self.root / 'config' if xdg else self.home / '.config'
                self.local_settings(config)
                if xdg:
                    self.env['XDG_CONFIG_HOME'] = str(config)
                else:
                    self.env.pop('XDG_CONFIG_HOME')
                result = self.run_command()
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.copied()
                self.assertEqual(self.scratch(), [])
                shutil.rmtree(self.store)

    def test_environment_overrides_local_settings(self):
        overrides = {name: self.env[name] for name in (
            'BIOMCP_EVIDENCE_HOST', 'BIOMCP_EVIDENCE_DIR', 'BIOMCP_SCRATCH_DIR')}
        self.local_settings(Path(self.env['XDG_CONFIG_HOME']))
        overrides.update(BIOMCP_EVIDENCE_HOST='environment-store',
                         BIOMCP_EVIDENCE_DIR=str(self.root / 'environment-evidence'),
                         BIOMCP_SCRATCH_DIR=str(self.root / 'environment-scratch'))
        result = self.run_command(FAKE_HOST='environment-store', **overrides)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertFalse(self.store.exists())
        self.assertFalse(self.scratch_root.exists())
        self.store = Path(overrides['BIOMCP_EVIDENCE_DIR'])
        self.copied()
        self.assertEqual(list(Path(overrides['BIOMCP_SCRATCH_DIR']).iterdir()), [])

    def test_arguments_override_environment_and_local_settings(self):
        overrides = {name: self.env[name] for name in (
            'BIOMCP_EVIDENCE_HOST', 'BIOMCP_EVIDENCE_DIR', 'BIOMCP_SCRATCH_DIR')}
        self.local_settings(Path(self.env['XDG_CONFIG_HOME']))
        evidence, scratch = self.root / 'argument-evidence', self.root / 'argument-scratch'
        result = self.run_command(
            args=('--host', 'argument-store', '--evidence-dir', str(evidence),
                  '--scratch-dir', str(scratch)), FAKE_HOST='argument-store', **overrides)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertFalse(self.store.exists())
        self.assertFalse(self.scratch_root.exists())
        self.store = evidence
        self.copied()
        self.assertEqual(list(scratch.iterdir()), [])

    def test_malformed_local_settings_fail_before_starting(self):
        self.local_settings(Path(self.env['XDG_CONFIG_HOME']))
        settings = Path(self.env['XDG_CONFIG_HOME']) / 'biomcp/nightly.json'
        for body in ('{', '[]', '{"host": null}', '{"scratch_dir": 3}'):
            with self.subTest(body=body):
                settings.write_text(body)
                result = self.run_command()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('invalid nightly settings at ' + str(settings), result.stderr)
                self.assertNotIn('Owned scratch:', result.stdout)
                self.assertFalse(self.scratch_root.exists())
                self.assertFalse(self.store.exists())

    def test_empty_settings_fail_before_starting_from_each_origin(self):
        environment = dict(self.env)
        self.local_settings(Path(self.env['XDG_CONFIG_HOME']))
        settings = Path(self.env['XDG_CONFIG_HOME']) / 'biomcp/nightly.json'
        valid = json.loads(settings.read_text())
        for origin in ('file', 'environment', 'argument'):
            for key, variable, option in (
                    ('host', 'BIOMCP_EVIDENCE_HOST', '--host'),
                    ('evidence_dir', 'BIOMCP_EVIDENCE_DIR', '--evidence-dir'),
                    ('scratch_dir', 'BIOMCP_SCRATCH_DIR', '--scratch-dir')):
                with self.subTest(origin=origin, key=key):
                    self.env = dict(environment)
                    settings.write_text(json.dumps(valid))
                    args = ()
                    if origin == 'file':
                        for name in ('BIOMCP_EVIDENCE_HOST', 'BIOMCP_EVIDENCE_DIR',
                                     'BIOMCP_SCRATCH_DIR'):
                            self.env.pop(name)
                        settings.write_text(json.dumps(dict(valid, **{key: ''})))
                    elif origin == 'environment':
                        self.env[variable] = ''
                    else:
                        args = (option, '')
                    result = self.run_command(args=args)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn('supply --host', result.stderr)
                    self.assertNotIn('Owned scratch:', result.stdout)
                    self.assertFalse(self.scratch_root.exists())
                    self.assertFalse(self.store.exists())

    def test_clean_copies_and_cleans_only_owned_scratch(self):
        unowned = self.scratch_root / 'another-run'
        unowned.mkdir(parents=True)
        (unowned / 'keep').write_text('owned by another run')
        result = self.run_command()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual((self.copied() / 'exit-status').read_text(), '0\n')
        self.assertEqual(self.scratch(), [unowned])
        self.assertEqual((unowned / 'keep').read_text(), 'owned by another run')
        second = self.run_command()
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertEqual(len(list(self.store.glob('*'))), 2)

    def test_drift_errors_and_failed_runner_still_copy(self):
        for overrides in [dict(FAKE_REPORT='{"drift_cases":1,"error_cases":0}'),
                          dict(FAKE_REPORT='{"drift_cases":0,"error_cases":1}'),
                          dict(FAKE_REPORT='{"drift_cases":0,"error_cases":0,"eligible":1,"checked":0}'),
                          dict(FAKE_REPORT='{"drift_cases":0,"error_cases":0,"checked":0}'),
                          dict(FAKE_EXIT='7'), dict(FAKE_REPORT='{}')]:
            with self.subTest(overrides=overrides):
                result = self.run_command(**overrides)
                self.assertNotEqual(result.returncode, 0)
                self.copied()
                self.assertEqual(self.scratch(), [])
                shutil.rmtree(self.store)

    def test_diagnostic_witness_survives_verified_delivery(self):
        witness = {"baseline": {"output": {"id": "old"}}, "live": {
            "requests": [{"url": "https://example.org/search?q=cancer", "status": 200,
                          "shape": {"fields": {"/id": True}}}],
            "result_ids": [{"field": "id", "value": "new"}], "output": {"id": "new"}}}
        report = {"drift_cases": 1, "error_cases": 0, "eligible": 2, "checked": 2,
                  "case_sample": [{"case": "changed", "witness": witness,
                                   "result": {"drift": ["x: request-plan-changed"]}},
                                  {"case": "clean", "result": {"drift": []}}]}
        result = self.run_command(FAKE_REPORT=json.dumps(report))
        self.assertNotEqual(result.returncode, 0)
        copied = json.loads((self.copied() / 'report.json').read_text())
        self.assertEqual(copied['case_sample'][0]['witness'], witness)
        self.assertEqual(copied['case_sample'][0]['classification'], 'failure')
        self.assertEqual(self.scratch(), [])

    def test_trial_continuation_uses_live_follow_and_excludes_recorded_mcp_token(self):
        case = json.loads((ROOT / 'tests/native/case-inputs/trial/walk.yaml').read_text())
        live = [step for step in case['steps'] if not step.get('offline')]
        self.assertEqual(live[0]['run'], ['search', 'trial', '--condition', 'melanoma'])
        self.assertEqual(live[1]['follow'], 'page')
        self.assertEqual(len(live), 2)
        recorded = [step for step in case['steps'] if 'mcp' in step][0]
        self.assertIn('--next-page', recorded['mcp'])
        self.assertTrue(recorded['offline'])

    def test_failed_copy_or_manifest_keeps_scratch(self):
        for failure in ['fail', 'manifest']:
            with self.subTest(failure=failure):
                result = self.run_command(FAKE_COPY=failure)
                self.assertNotEqual(result.returncode, 0)
                scratch = self.scratch()
                self.assertEqual(len(scratch), 1, result.stdout + result.stderr)
                self.assertTrue((scratch[0] / 'report.json').is_file())
                self.assertTrue((scratch[0] / 'manifest.json').is_file())
                shutil.rmtree(scratch[0])


def load_tests(loader, tests, pattern):
    # Keep the original entry point complete when discovery targets only it.
    # Broader discovery loads the classification module itself exactly once.
    if pattern is None or not fnmatch.fnmatchcase("test_nightly_classification.py", pattern):
        if __package__:
            from . import test_nightly_classification as classification
        else:
            import test_nightly_classification as classification
        tests.addTests(loader.loadTestsFromModule(classification))
    return tests


if __name__ == '__main__':
    unittest.main()
