"""Check nightly source classification through the isolated scheduled command."""
import json
import os
import shutil
import subprocess
import unittest

if __package__:
    from . import nightly_test_support as support
    from .nightly_recommendation_cases import (
        classification_case, publication_growth_case, recommendation_case, reranked_tail_case)
else:
    import nightly_test_support as support
    from nightly_recommendation_cases import (
        classification_case, publication_growth_case, recommendation_case, reranked_tail_case)

ROOT = support.ROOT


class NightlyClassification(support.NightlyHarness, unittest.TestCase):
    def test_discovery_changes_dependents_but_not_invariants(self):
        for change, expected, code in [
                ('discovery', 'churn', 0), ('unchanged', 'failure', 1),
                ('order', 'healthy', 0), ('schema', 'failure', 1),
                ('500', 'outage', 0), ('429', 'throttled', 0),
                ('cut', 'failure', 1), ('clean-cut', 'failure', 1), ('missing', 'uncertain', 1),
                ('unrelated', 'failure', 1), ('invariant', 'failure', 1)]:
            with self.subTest(change=change):
                self.classified(classification_case(change), expected, code)


    @unittest.skipUnless(os.environ.get('BIOMCP_NIGHTLY_REPLAY_HOST'), 'stored report replay is opt-in')
    def test_stored_reports_change_only_failed_root_consequences(self):
        import collections
        if __package__:
            from . import nightly_live_check as current
        else:
            import nightly_live_check as current
        baseline = os.environ['BIOMCP_NIGHTLY_REPLAY_BASE']
        source = subprocess.check_output(['git', 'show', baseline + ':scripts/nightly_live_check.py'],
                                         cwd=ROOT, text=True)
        old = {'__name__': 'baseline_classifier'}
        exec(compile(source, 'baseline_classifier', 'exec'), old)
        reader = """import json,pathlib
root=pathlib.Path('/home/ian/workspace/data/biomcp-redux/0068-scheduled')
print(json.dumps([(p.parent.name,p.read_text()) for p in sorted(root.glob('*/report.json'))
                  if p.parent.name[:8]>='20261027']))
"""
        import shlex
        rows = json.loads(subprocess.check_output(['ssh', os.environ['BIOMCP_NIGHTLY_REPLAY_HOST'],
            shlex.join(['python3', '-c', reader])], text=True))
        self.assertTrue(rows)
        for name, raw in rows:
            report = json.loads(raw[raw.index('{'):])
            changes, counts = [], collections.Counter()
            for row in report.get('case_sample', []):
                before, after = old['classify_case'](row), current.classify_case(row)
                counts[after] += 1
                if before != after:
                    changes.append((row['case'], before, after))
                    with self.subTest(run=name, case=row['case']):
                        self.assertTrue({before, after} <= {'failure', 'outage', 'uncertain'})
                        self.assertTrue(any(current.failed_status(request.get('status'))
                            and request.get('status') != 429 for command in row['witness']
                            for request in command['live']['requests']))
            print(json.dumps({'run': name, 'counts': dict(counts), 'changes': changes}), flush=True)

    def test_failed_discovery_explains_only_its_dependents_and_cached_retries(self):
        import copy
        for status in (429, 500, None):
            for change in ('dependent', 'cached-retry', 'unrelated', 'shape', 'plan', 'plan-output', 'nonempty-cache'):
                with self.subTest(status=status, change=change):
                    row = classification_case('order')
                    command = row['witness'][0]
                    command['live']['requests'].reverse()
                    root = command['live']['requests'][0]
                    root['status'] = status
                    root['provenance'] = {}
                    command['live']['requests'] = [root]
                    row['result']['drift'] = ['search: HTTP status changed',
                                              'old-id: request-plan-removed']
                    if change == 'cached-retry':
                        later = copy.deepcopy(command)
                        later['ordinal'] = 2
                        later['baseline']['requests'] = []
                        row['witness'].append(later)
                        row['result']['commands'].append({'ordinal': 2})
                        row['result']['drift'].append('search: request-plan-added')
                    elif change == 'nonempty-cache':
                        later = copy.deepcopy(command)
                        later['ordinal'] = 2
                        unrelated = {'identity': 'unrelated', 'source': 'other',
                                     'role': 'Search', 'status': 200}
                        later['baseline']['requests'] = [unrelated]
                        later['live']['requests'] = [unrelated, dict(root, status=200)]
                        row['witness'].append(later)
                        row['result']['commands'].append({'ordinal': 2})
                        row['result']['drift'].append('search: request-plan-added')
                    elif change == 'unrelated':
                        command['baseline']['requests'][-1]['provenance']['inputs']['ids'] = ['unrelated']
                    elif change == 'shape':
                        row['result']['drift'].append('search: field removed: /id')
                    elif change in ('plan', 'plan-output'):
                        command['live']['requests'].append({'identity': 'unexplained',
                            'source': 'other', 'role': 'Search', 'status': 200})
                        row['result']['drift'].append('unexplained: request-plan-added')
                        if change == 'plan-output':
                            command['baseline']['output'] = {'status': 'complete'}
                            command['live']['output'] = {'status': 'failed', 'sources': [
                                {'source': 'pubmed', 'state': 'failed', 'reason': 'unavailable'}]}
                            row['result']['drift'].extend([
                                'command 1: output status changed: complete -> failed',
                                'page continuation changed'])
                    positive = change in ('dependent', 'cached-retry')
                    self.classified(row, ('throttled' if status == 429 else 'outage')
                                    if positive else 'uncertain' if change in ('plan', 'plan-output', 'nonempty-cache') else 'failure',
                                    int(not positive))

    def test_missing_discovery_proof_keeps_two_command_outcome_uncertain(self):
        import copy
        for change in ('missing-proof', 'unrelated-output'):
            with self.subTest(change=change):
                row = classification_case('order')
                first = row['witness'][0]
                first['live']['requests'].reverse()
                root = first['live']['requests'][0]
                first['baseline']['requests'][0]['provenance'] = {}
                root['status'], root['provenance'] = 429, {}
                first['live']['requests'] = [root]
                first['baseline']['output'] = {'status': 'complete'}
                first['live']['output'] = {'status': 'failed', 'sources': [
                    {'source': 'pubmed', 'state': 'failed', 'reason': 'rate-limited'}]}
                dependent = copy.deepcopy(first['baseline']['requests'][1])
                dependent['identity'] = 'next-page'
                later = {'ordinal': 2, 'baseline': {'requests': [dependent]},
                         'live': {'requests': [dict(root, status=200), dependent]}}
                row['witness'].append(later)
                row['result']['commands'].append({'ordinal': 2})
                row['result']['drift'] = ['search: HTTP status 200 -> 429',
                    'old-id: request-plan-removed', 'search: request-plan-added',
                    'command 1: output status changed: complete -> failed',
                    'page continuation changed']
                if change == 'unrelated-output':
                    first['live']['output']['sources'].append({
                        'source': 'unrelated', 'state': 'failed', 'reason': 'unavailable'})
                self.classified(row, 'uncertain' if change == 'missing-proof' else 'failure', 1)

    def test_cached_failed_output_requires_the_same_output_and_transition(self):
        import copy
        for change in ('cached', 'output', 'transition', 'missing-output', 'shape'):
            with self.subTest(change=change):
                row = classification_case('500-output')
                first = row['witness'][0]
                row['result']['commands'][0].update(before_status='complete', after_status='failed')
                later = copy.deepcopy(first)
                later['ordinal'] = 2
                for side in ('baseline', 'live'):
                    later[side]['requests'] = []
                row['witness'].append(later)
                row['result']['commands'].append({'ordinal': 2,
                    'before_status': 'complete', 'after_status': 'failed'})
                row['result']['drift'].append('command 2: output status changed: complete -> failed')
                if change == 'output':
                    later['live']['output']['unrelated'] = True
                elif change == 'transition':
                    row['result']['commands'][1]['after_status'] = 'partial'
                elif change == 'missing-output':
                    for command in row['witness']:
                        for side in ('baseline', 'live'):
                            command[side].pop('output')
                elif change == 'shape':
                    row['result']['drift'].append('command 2: field removed: /id')
                self.classified(row, 'outage' if change == 'cached' else 'failure',
                                int(change != 'cached'))

    def test_throttle_explains_downstream_drift_only_in_its_command(self):
        row = classification_case('429-output')
        row['witness'][0]['baseline']['requests'].append({
            'identity': 'downstream', 'source': 'icite', 'role': 'Summary', 'status': 200,
            'provenance': {'input_context': 'fixed', 'inputs': {
                'namespace': 'pmid', 'ids': ['child']}}})
        row['witness'][0]['baseline']['requests'][1]['provenance']['discovery'] = {
            'namespace': 'pmid', 'ids': ['child']}
        row['result']['drift'].extend(['downstream: request-plan-removed',
                                       'page continuation changed'])
        self.classified(row, 'throttled', 0)
        second = classification_case('unchanged')['witness'][0]
        second['ordinal'] = 2
        row['witness'].append(second)
        row['result']['commands'].append({'ordinal': 2})
        self.classified(row, 'failure', 1)

    def test_throttle_explains_later_cache_miss_only_with_root_dependencies(self):
        import copy
        for change in ('cache-miss', 'failed', 'source', 'baseline', 'unrelated',
                       'missing-inputs', 'no-earlier-throttle', 'later-throttle', 'schema'):
            with self.subTest(change=change):
                row = classification_case('429-output')
                later = copy.deepcopy(classification_case('discovery')['witness'][0])
                later['ordinal'] = 2
                later['baseline']['requests'] = []
                root, dependent = later['live']['requests']
                dependent['source'] = 'icite'
                if change == 'failed':
                    dependent['status'] = 503
                elif change == 'source':
                    root['source'] = 'unrelated'
                elif change == 'baseline':
                    later['baseline']['requests'] = [
                        {'identity': 'baseline-only', 'source': 'pubmed', 'status': 200}]
                elif change == 'unrelated':
                    dependent['provenance']['inputs']['ids'] = ['unrelated']
                elif change == 'missing-inputs':
                    dependent.pop('provenance')
                elif change == 'no-earlier-throttle':
                    row = classification_case('order')
                row['witness'].append(later)
                row['result']['commands'].append({'ordinal': 2})
                row['result']['drift'].extend(['search: request-plan-added',
                                               'new-id: request-plan-added'])
                if change == 'later-throttle':
                    row['witness'].reverse()
                    for ordinal, command in enumerate(row['witness'], 1):
                        command['ordinal'] = ordinal
                    row['result']['drift'] = ['search: request-plan-added']
                elif change == 'schema':
                    row['result']['drift'].append('output schema failure')
                expected = ('throttled' if change == 'cache-miss' else
                            'failure' if change in ('failed', 'schema') else 'uncertain')
                self.classified(row, expected, int(expected != 'throttled'))

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
                                    'failure', 1)

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


if __name__ == '__main__':
    unittest.main()
