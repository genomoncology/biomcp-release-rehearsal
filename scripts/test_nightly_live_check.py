"""Exercise the scheduled command offline, with fake Cargo and a local SSH store."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
if __package__:
    from .nightly_recommendation_cases import classification_case, recommendation_case, reranked_tail_case
else:
    from nightly_recommendation_cases import classification_case, recommendation_case, reranked_tail_case
CARGO = '''#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
assert sys.argv[1:] == ['test','--locked','-p','biomcp','--test','tools','--','--live-check','--details']
print(json.dumps(json.loads(os.environ['FAKE_REPORT'])))
print('offline build log', file=sys.stderr)
sys.exit(int(os.environ.get('FAKE_EXIT', '0')))
'''
SSH = '''#!/usr/bin/env python3
import io, os, subprocess, sys, tarfile
assert sys.argv[-2] == os.environ.get('FAKE_HOST', 'offline-store')
if '# nightly-history' in sys.argv[-1]:
    print(os.environ.get('FAKE_HISTORY', '[]'))
    sys.exit(0)
if os.environ.get('FAKE_COPY') == 'fail':
    sys.exit(23)
payload = sys.stdin.buffer.read()
if os.environ.get('FAKE_COPY') == 'manifest':
    changed = io.BytesIO()
    with tarfile.open(fileobj=io.BytesIO(payload)) as original, tarfile.open(fileobj=changed, mode='w') as replacement:
        for entry in original:
            body = original.extractfile(entry).read()
            if entry.name == 'report.json':
                body += b'corrupt'
                entry.size = len(body)
            replacement.addfile(entry, io.BytesIO(body))
    payload = changed.getvalue()
result = subprocess.run(sys.argv[-1], shell=True, input=payload)
sys.exit(result.returncode)
'''


def publication_growth_case(change='addition', count=1590, insertion=650):
    # Retained LitVar diagnostics: 1590 PMIDs, 200-ID iCite batches,
    # unchanged PubMed selection, then one insertion and rebatching.
    old = [str(10000000 + i) for i in range(count)]
    new = old[:insertion] + ['20000000'] + old[insertion:]
    if change == 'removal':
        old, new = new, old
    def requests(ids):
        rows = [{'identity': 'publications', 'source': 'litvar', 'role': 'Publications',
                 'status': 200, 'provenance': {'inputs': None, 'input_context': None,
                 'discovery': {'namespace': 'pmid', 'ids': ids}}}]
        for start in range(0, len(ids), 200):
            batch = ids[start:start + 200]
            rows.append({'identity': ','.join(batch), 'source': 'icite', 'role': 'Raw',
                         'status': 200, 'provenance': {'input_context': 'fixed-request',
                         'inputs': {'namespace': 'pmid', 'ids': batch}}})
        rows.append({'identity': 'same-pubmed', 'source': 'pubmed', 'role': 'Summary',
                     'status': 200})
        return rows
    before, after = requests(old), requests(new)
    if change == 'missing-batch':
        after.pop(-2)
    if change == 'parameter':
        for row in after[1:-1]:
            row['provenance']['input_context'] = 'changed-request'
    if change == 'unproven':
        after[0]['provenance']['discovery']['ids'] = old
    return {'case': 'article/litvar/akt1-rsid-citations.yaml',
            'result': {'drift': ['icite: request-plan-changed', 'icite: request-plan-added'],
                       'commands': [{'ordinal': 1, 'exchanges': {'omitted': 0, 'drift_omitted': 0}}]},
            'witness': [{'ordinal': 1, 'baseline': {'requests': before},
                         'live': {'requests': after}}]}


class NightlyCommand(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.checkout = self.root / 'checkout'
        (self.checkout / 'scripts').mkdir(parents=True)
        shutil.copy(ROOT / 'Makefile', self.checkout / 'Makefile')
        script = ROOT / 'scripts/nightly_live_check.py'
        if script.exists():
            shutil.copy(script, self.checkout / 'scripts')
        self.home = self.root / 'home'
        self.scratch_root = self.root / 'scratch'
        self.store = self.root / 'store'
        self.tools = self.root / 'tools'
        self.tools.mkdir()
        for name, content in [('cargo', CARGO), ('ssh', SSH), ('git', '#!/bin/sh\nprintf \"%040d\\n\" 1\n')]:
            target = self.tools / name
            target.write_text(content)
            target.chmod(0o755)
        self.env = dict(os.environ, HOME=str(self.home),
                        XDG_CONFIG_HOME=str(self.home / '.config'),
                        TMPDIR=str(self.root / 'tmp'),
                        BIOMCP_EVIDENCE_HOST='offline-store',
                        BIOMCP_EVIDENCE_DIR=str(self.store),
                        BIOMCP_SCRATCH_DIR=str(self.scratch_root),
                        PATH=str(self.tools) + os.pathsep + os.environ['PATH'],
                        FAKE_REPORT='{"drift_cases":0,"error_cases":0,"eligible":0,"checked":0}')
        Path(self.env['TMPDIR']).mkdir()

    def run_command(self, args=(), **values):
        env = dict(self.env, **values)
        command = ([sys.executable, 'scripts/nightly_live_check.py', *args] if args
                   else ['make', 'nightly-live-check'])
        return subprocess.run(command, cwd=self.checkout,
                              env=env, text=True, capture_output=True, check=False)

    def copied(self):
        runs = list(self.store.glob('*'))
        self.assertEqual(len(runs), 1)
        run = runs[0]
        manifest = json.loads((run / 'manifest.json').read_text())
        for name, digest in manifest.items():
            self.assertEqual(hashlib.sha256((run / name).read_bytes()).hexdigest(), digest)
        self.assertEqual((run / 'build.log').read_text(), 'offline build log\n')
        return run

    def scratch(self):
        return list(self.scratch_root.glob('*'))

    def test_missing_destination_fails_before_running_or_creating_scratch(self):
        for name in ('BIOMCP_EVIDENCE_HOST', 'BIOMCP_EVIDENCE_DIR', 'BIOMCP_SCRATCH_DIR'):
            self.env.pop(name, None)
        result = self.run_command()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('supply --host', result.stderr)
        self.assertFalse(self.scratch_root.exists())
        self.assertFalse(self.store.exists())

    def local_settings(self, root):
        settings = root / 'biomcp/nightly.json'
        settings.parent.mkdir(parents=True)
        settings.write_text(json.dumps({
            'host': 'offline-store', 'evidence_dir': str(self.store),
            'scratch_dir': str(self.scratch_root)}))
        for name in ('BIOMCP_EVIDENCE_HOST', 'BIOMCP_EVIDENCE_DIR', 'BIOMCP_SCRATCH_DIR'):
            self.env.pop(name, None)

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

    def classified(self, row, expected, exit_code):
        shutil.rmtree(self.store, ignore_errors=True)
        report = {"drift_cases": int(bool(row["result"]["drift"])), "error_cases": 0,
                  "eligible": 1, "checked": 1, "omitted": 0, "case_sample": [row]}
        result = self.run_command(FAKE_REPORT=json.dumps(report))
        self.assertEqual(result.returncode == 0, exit_code == 0, result.stdout + result.stderr)
        saved = json.loads((self.copied() / 'report.json').read_text())
        self.assertEqual(saved['case_sample'][0]['classification'], expected)
        self.assertEqual(saved[expected + '_cases'], 1)

    def test_discovery_changes_dependents_but_not_invariants(self):
        for change, expected, code in [
                ('discovery', 'churn', 0), ('unchanged', 'failure', 1),
                ('order', 'healthy', 0), ('schema', 'failure', 1),
                ('500', 'outage', 0), ('429', 'throttled', 0),
                ('cut', 'failure', 1), ('clean-cut', 'failure', 1), ('missing', 'uncertain', 1),
                ('unrelated', 'failure', 1), ('invariant', 'failure', 1)]:
            with self.subTest(change=change):
                self.classified(classification_case(change), expected, code)

    def test_throttle_explains_downstream_drift_only_in_its_command(self):
        row = classification_case('429-output')
        row['witness'][0]['baseline']['requests'].append({
            'identity': 'downstream', 'source': 'icite', 'status': 200})
        row['result']['drift'].extend(['downstream: request-plan-removed',
                                       'page continuation changed'])
        self.classified(row, 'throttled', 0)
        second = classification_case('unchanged')['witness'][0]
        second['ordinal'] = 2
        row['witness'].append(second)
        row['result']['commands'].append({'ordinal': 2})
        self.classified(row, 'failure', 1)

    def test_recovery_keeps_schema_and_invariant_failures(self):
        for change, expected in [('recovery', 'recovered'), ('shape', 'failure')]:
            row = classification_case('429-output')
            command = row['witness'][0]
            command['baseline']['requests'][-1]['status'] = None
            command['live']['requests'][-1]['status'] = 200
            row['result']['commands'][0].update(before_status='partial', after_status='complete')
            row['result']['drift'] = [
                'old-id: HTTP status None -> Some(200)',
                'command 1: output status changed: partial -> complete']
            if change == 'shape':
                row['result']['drift'].append('old-id: field removed: /id')
            self.classified(row, expected, int(expected == 'failure'))

    def test_recovery_does_not_explain_unrelated_plan_changes(self):
        row = classification_case('unchanged')
        row['result']['commands'][0].update(before_status='partial', after_status='complete')
        self.classified(row, 'failure', 1)

    def test_provider_failure_does_not_hide_same_source_bad_reply(self):
        for change in ('429-output', '500-output'):
            for reason in ('bad_reply', 'bad-reply'):
                with self.subTest(change=change, reason=reason):
                    row = classification_case(change)
                    row['witness'][0]['live']['output']['sources'][0]['reason'] = reason
                    self.classified(row, 'failure', 1)
                    row['result']['drift'] = row['result']['drift'][:1]
                    row['witness'][0]['baseline']['output']['status'] = 'failed'
                    self.classified(row, 'failure', 1)

    def test_recovery_retains_case_and_output_source_without_failed_request(self):
        row = classification_case('order')
        row['result']['drift'] = ['command 1: output status changed: partial -> complete']
        row['result']['commands'][0].update(before_status='partial', after_status='complete')
        row['witness'][0]['baseline']['output'] = {'status': 'partial', 'sources': [
            {'source': 'pubmed', 'state': 'failed', 'reason': 'unavailable'}]}
        row['witness'][0]['live']['output'] = {'status': 'complete'}
        self.classified(row, 'recovered', 0)
        saved = json.loads((self.copied() / 'report.json').read_text())
        self.assertEqual(saved['cause_classes']['recovered'], {
            'cases': ['order'], 'sources': ['pubmed']})

    def test_outage_fails_on_third_calendar_night_and_resets_after_gap(self):
        import datetime
        row = classification_case('500')
        report = {'eligible': 1, 'checked': 1, 'drift_cases': 1, 'error_cases': 0,
                  'case_sample': [row]}
        today = datetime.datetime.now(datetime.timezone.utc).date()
        history = [[(today - datetime.timedelta(days=offset)).strftime('%Y%m%d'),
                    json.dumps(report)] for offset in (2, 1)]
        result = self.run_command(FAKE_REPORT=json.dumps(report), FAKE_HISTORY=json.dumps(history))
        self.assertNotEqual(result.returncode, 0, result.stderr)
        saved = json.loads((self.copied() / 'report.json').read_text())
        self.assertEqual(saved['persistent_outage_sources'], ['pubmed'])
        shutil.rmtree(self.store)
        result = self.run_command(FAKE_REPORT=json.dumps(report), FAKE_HISTORY=json.dumps(history[:1]))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads((self.copied() / 'report.json').read_text())[
            'persistent_outage_sources'], [])

    def test_complete_recommendation_replacements_follow_exact_discovery(self):
        for story in ('multi', 'single'):
            for day in ('1101', '1102'):
                with self.subTest(story=story, day=day):
                    self.classified(recommendation_case(story, day), 'churn', 0)
            for change in ('unrelated', 'context', 'failed-discovery', 'subset', 'partial'):
                with self.subTest(story=story, change=change):
                    self.classified(recommendation_case(story, change=change),
                                    'outage' if change == 'failed-discovery' else 'failure',
                                    0 if change == 'failed-discovery' else 1)

    def test_reranked_source_tail_requires_complete_provider_owned_selection(self):
        for change in ('boundary', 'prefix', 'filter', 'offset', 'limit', 'missing-batch',
                       'duplicate-id', 'omitted-id', 'unchanged', 'source', 'role', 'url',
                       'status', 'over-cap', 'missing-metadata', 'duplicate-namespace',
                       'stale', 'schema', 'absence', 'refusal'):
            with self.subTest(change=change):
                positive = change in ('boundary', 'prefix')
                self.classified(reranked_tail_case(change), 'churn' if positive else 'failure',
                                0 if positive else 1)

    def test_doi_group_change_tracks_graph_discovery_input(self):
        row = classification_case('discovery')
        for side in ('baseline', 'live'):
            for request in row['witness'][0][side]['requests']:
                for kind in ('discovery', 'inputs'):
                    ids = request['provenance'].get(kind)
                    if ids:
                        ids['namespace'] = 'doi'
                        ids['ids'] = ['10.1000/' + value for value in ids['ids']]
        self.classified(row, 'churn', 0)
        row['witness'][0]['live']['requests'][1]['provenance']['input_context'] = 'changed-query-mode'
        self.classified(row, 'failure', 1)

    def test_causality_requires_ordered_selection_complete_diagnostics_and_fixed_parameters(self):
        for change, expected, code in [
                ('rank', 'churn', 0), ('unused-tail', 'failure', 1),
                ('parameter', 'failure', 1), ('omitted', 'failure', 1),
                ('omitted-count', 'failure', 1),
                ('429-output', 'throttled', 0), ('429-schema', 'failure', 1),
                ('429-other-error', 'failure', 1),
                ('500-output', 'outage', 0)]:
            with self.subTest(change=change):
                self.classified(classification_case(change), expected, code)

    def test_incomplete_command_coverage_cannot_clear_changed_cases(self):
        for change in ('plan-drift', 'output-drift', 'unmatched-output', 'duplicate', 'extra', 'reordered',
                       'missing-side', 'missing-requests', 'missing-commands', 'wrong-ordinal'):
            with self.subTest(change=change):
                row = classification_case('order')
                row['result']['commands'].append({'ordinal': 2, 'exchanges': {'omitted': 0}})
                row['result']['drift'] = ['second: request-plan-changed']
                if change == 'output-drift':
                    row['result']['drift'] = ['command 2: output status changed: complete -> partial']
                if change in ('duplicate', 'extra', 'reordered', 'missing-side', 'missing-requests',
                              'wrong-ordinal', 'unmatched-output'):
                    second = json.loads(json.dumps(row['witness'][0]))
                    second['ordinal'] = 2
                    row['witness'].append(second)
                    if change == 'unmatched-output':
                        row['result']['drift'] = ['command 3: output status changed: complete -> partial']
                    elif change == 'duplicate':
                        second['ordinal'] = 1
                    elif change == 'extra':
                        row['witness'].append(dict(second, ordinal=3))
                    elif change == 'reordered':
                        row['witness'].reverse()
                    elif change == 'missing-side':
                        second.pop('live')
                    elif change == 'missing-requests':
                        second['live'].pop('requests')
                    elif change == 'wrong-ordinal':
                        row['result']['commands'][1]['ordinal'] = 3
                if change == 'missing-commands':
                    row['result'].pop('commands')
                self.classified(row, 'failure', 1)

    def test_any_exchange_omission_fails_before_source_classification(self):
        for clean in (False, True):
            for location in ('drift', 'exchanges', 'before-statuses', 'after-statuses'):
                with self.subTest(clean=clean, location=location):
                    row = classification_case('rank')
                    exchanges = row['result']['commands'][0]['exchanges']
                    if clean:
                        row['result']['drift'] = []
                    if location == 'drift':
                        exchanges['drift_omitted'] = 1
                    elif location == 'exchanges':
                        exchanges['omitted'] = 1
                    else:
                        side = 'before_statuses' if location == 'before-statuses' else 'after_statuses'
                        exchanges['items'] = [{side: {'values': [200], 'omitted': 1}}]
                    self.classified(row, 'failure', 1)

    def test_publication_count_change_keeps_request_diagnostics(self):
        for change, expected in [('addition', 'churn'), ('removal', 'churn'),
                                 ('missing-batch', 'failure'), ('parameter', 'failure'),
                                 ('unproven', 'failure')]:
            with self.subTest(change=change):
                row = publication_growth_case(change)
                self.classified(row, expected, 0 if expected == 'churn' else 1)
                saved = json.loads((self.copied() / 'report.json').read_text())
                self.assertEqual(saved['case_sample'][0]['witness'], row['witness'])
                self.assertEqual(saved['case_sample'][0]['result'], row['result'])

    def test_publication_change_adds_or_removes_a_whole_batch(self):
        for change in ('addition', 'removal', 'missing-batch', 'parameter'):
            with self.subTest(change=change):
                row = publication_growth_case(
                    'addition' if change == 'missing-batch' else change,
                    count=1600, insertion=1600)
                if change == 'missing-batch':
                    row['witness'][0]['live']['requests'].pop(1)
                self.classified(row, 'churn' if change in ('addition', 'removal') else 'failure',
                                0 if change in ('addition', 'removal') else 1)
        for change in ('wrong-id', 'parameter', '400'):
            with self.subTest(extra=change):
                row = publication_growth_case(count=1600, insertion=1600)
                added = row['witness'][0]['live']['requests'][-2]
                if change == 'wrong-id':
                    added['provenance']['inputs']['ids'] = ['unexplained']
                elif change == 'parameter':
                    added['provenance']['input_context'] = 'changed-request'
                else:
                    added['status'] = 400
                self.classified(row, 'failure', 1)

    def test_changed_registry_token_follows_checked_first_page(self):
        row = classification_case('discovery')
        first = row['witness'][0]
        before, after = first['baseline']['requests'], first['live']['requests']
        for requests in (before, after):
            for request in requests:
                request['source'] = 'ctgov'
                for kind in ('discovery', 'inputs'):
                    if kind in request['provenance']:
                        request['provenance'][kind]['namespace'] = 'ctgov-page'
        # The first command discovers its own token. Only its follow changes
        # request identity, with unchanged non-token parameters and HTTP 200.
        second = {'ordinal': 2, 'baseline': {'requests': before[1:]},
                  'live': {'requests': after[1:]}}
        first['baseline']['requests'] = before[:1]
        first['live']['requests'] = after[:1]
        row['witness'].append(second)
        row['result']['commands'].append({'ordinal': 2, 'exchanges': {'omitted': 0, 'drift_omitted': 0}})
        self.classified(row, 'churn', 0)
        second['live']['requests'][0]['status'] = 400
        row['result']['drift'].append('continuation: HTTP status Some(200) -> Some(400)')
        self.classified(row, 'failure', 1)

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


if __name__ == '__main__':
    unittest.main()
