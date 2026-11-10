"""Run the live check with fake cargo and product commands."""
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
from nightly_live_check import CHECKS

CARGO = '''#!/usr/bin/env python3
import json, os, sys
if os.environ.get('FAKE_BUILD') == 'fail':
    print(json.dumps({'reason': 'compiler-message',
                      'message': {'level': 'error', 'message': 'mismatched types'}}))
    sys.exit(101)
print(json.dumps({'reason': 'compiler-artifact', 'target': {'name': 'biomcp'},
                  'executable': os.environ['FAKE_BIOMCP']}))
'''
# Replies keep the fields real source, search and usage replies carry.
BIOMCP = '''#!/usr/bin/env python3
import json, os, sys, time
with open(SEEN, 'a') as seen:
    seen.write(json.dumps({'args': sys.argv[1:], 'env': sorted(os.environ)}) + '\\n')
words = ' '.join(sys.argv[1:])
replies = json.loads(REPLIES)
for key, (code, body, *once) in replies.items():
    tries = sum(json.loads(line)['args'] == sys.argv[1:] for line in open(SEEN))
    if key in words and not (once == ['once'] and tries > 1):
        if once == ['sequence']:
            code, body = body[min(tries - 1, len(body) - 1)]
        if body == 'hang':
            time.sleep(30)
        print(body if isinstance(body, str) else json.dumps(body))
        print('crash detail', file=sys.stderr) if code == 101 else None
        sys.exit(code)
print(json.dumps({'schema': 'biomcp.article-page/9', 'status': 'complete',
                  'page': {'offset': 0, 'returned': 1, 'total': 193},
                  'result': {'items': [{'pmid': '1'}]},
                  'sources': [{'source': 'pubmed', 'state': 'ok', 'count': 193}],
                  'problems': [{'source': 'pubmed', 'field': 'electronic_date',
                                'code': 'value.unknown'}]}))
'''
FAILED_SOURCE = {'schema': 'biomcp.article-page/9', 'status': 'failed',
                 'sources': [{'source': 'pubmed', 'state': 'failed', 'reason': 'proxy',
                              'term': 'AKT1 E17K'}]}
NO_RESULTS = {'schema': 'biomcp.trial-page/14', 'status': 'complete',
              'page': {'offset': 0, 'returned': 0, 'total': 0},
              'sources': [{'source': 'ctgov', 'state': 'ok', 'count': 0}]}
# Cell line and pathway searches carry no top-level page, and an empty list may be left out.
NO_ITEMS = {'schema': 'biomcp.cell-line-page/1', 'status': 'complete', 'page': None,
            'result': {'release': '53'}, 'sources': [{'source': 'cellosaurus', 'state': 'ok'}]}
EMPTY_ITEMS = {'schema': 'biomcp.pathway-page/1', 'status': 'complete', 'page': None,
               'result': {'items': [], 'page': {'offset': 0, 'returned': 0, 'total': 0}},
               'sources': [{'source': 'reactome', 'state': 'ok'}]}
USAGE = {'schema': 'biomcp.error/4', 'status': 'usage',
         'error': {'code': 'usage.arguments', 'message': 'expected a supported command'}}
PARTIAL = {'schema': 'biomcp.gene-cards/13', 'status': 'partial',
           'sources': [{'source': 'hgnc', 'state': 'ok'},
                       {'source': 'gnomad', 'state': 'failed', 'reason': 'timeout'}]}


class LiveCheck(unittest.TestCase):
    def run_check(self, replies=None, build=None, extra=()):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            seen = root / 'seen.jsonl'
            # The product sees only the allowed environment, so its settings live in the file.
            settings = 'SEEN = {!r}\nREPLIES = {!r}\n'.format(str(seen), json.dumps(replies or {}))
            for name, body in (('cargo', CARGO), ('biomcp', BIOMCP.replace('\n', '\n' + settings, 1))):
                path = root / name
                path.write_text(body)
                path.chmod(path.stat().st_mode | stat.S_IXUSR)
            env = {**os.environ, 'PATH': str(root) + os.pathsep + os.environ['PATH'],
                   'FAKE_BIOMCP': str(root / 'biomcp'),
                   'NCBI_API_KEY': 'test-only-value', 'ONCOKB_TOKEN': 'test-only-value'}
            if build:
                env['FAKE_BUILD'] = build
            result = subprocess.run(
                [sys.executable, 'scripts/nightly_live_check.py', '--pause', '0', *extra],
                cwd=ROOT, env=env, capture_output=True, text=True, check=False)
            rows = [json.loads(line) for line in seen.read_text().splitlines()] if seen.exists() else []
        return result, rows

    def test_passing_run_checks_every_command_and_exits_zero(self):
        result, rows = self.run_check()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(len(rows), len(CHECKS))
        self.assertIn('live check: 0 of {} commands failed in '.format(len(CHECKS)),
                      result.stdout.splitlines()[-1])

    def test_each_failure_is_one_line_with_command_and_reason_at_the_end(self):
        result, _ = self.run_check({
            'AKT1 E17K': [4, FAILED_SOURCE],
            'NCT04303780': [2, USAGE],
            'melanoma': [0, NO_RESULTS],
            'BRAF': [0, PARTIAL],
            'MOLM13': [101, 'thread panicked'],
        })
        self.assertEqual(result.returncode, 1)
        lines = result.stdout.splitlines()
        self.assertIn('live check: 5 of {} commands failed in '.format(len(CHECKS)), lines[-1])
        self.assertEqual(lines[-6:-1], [
            'FAIL biomcp get gene BRAF: gnomad: timeout',
            "FAIL biomcp search article 'AKT1 E17K': pubmed: proxy",
            'FAIL biomcp search trial --condition melanoma --limit 1: no results',
            'FAIL biomcp get trial NCT04303780: expected a supported command',
            'FAIL biomcp search cell-line MOLM13: exit 101: malformed reply',
        ])

    def test_a_search_with_no_top_level_page_and_no_items_fails(self):
        result, _ = self.run_check({'MOLM13': [0, NO_ITEMS], 'Signaling by EGFR': [0, EMPTY_ITEMS]})
        self.assertEqual(result.returncode, 1)
        self.assertIn("FAIL biomcp search pathway 'Signaling by EGFR': no results", result.stdout)
        self.assertIn('FAIL biomcp search cell-line MOLM13: no results', result.stdout)

    def test_a_hung_command_fails_alone_and_the_run_continues(self):
        result, rows = self.run_check({'MOLM13': [0, 'hang']}, extra=('--timeout', '1'))
        self.assertEqual(result.returncode, 1)
        self.assertIn('FAIL biomcp search cell-line MOLM13: no reply in 1.0 seconds', result.stdout)
        self.assertEqual(len(rows), len(CHECKS) + 1)

    def test_a_command_that_recovers_on_its_second_try_does_not_fail_the_run(self):
        slow = {'schema': 'biomcp.protein-page/1', 'status': 'failed',
                'sources': [{'source': 'uniprot', 'state': 'failed', 'reason': 'deadline-exceeded'}]}
        result, rows = self.run_check({'--reviewed': [4, slow, 'once']})
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('ok biomcp search protein --gene EGFR --reviewed on its second try, '
                      'first: uniprot: deadline-exceeded', result.stdout)
        self.assertNotIn('FAIL', result.stdout)
        self.assertEqual(len(rows), len(CHECKS) + 1)

    def attempts(self, result, args):
        return [row for line in result.stdout.splitlines() if line.startswith('{')
                and (row := json.loads(line)).get('command') == args]

    def test_both_attempts_keep_their_distinct_source_failure(self):
        second = {**FAILED_SOURCE, 'sources': [
            {'source': 'pubmed', 'state': 'failed', 'reason': 'unavailable'}]}
        result, _ = self.run_check({'AKT1 E17K': [0, [[4, FAILED_SOURCE], [4, second]], 'sequence']})
        attempts = self.attempts(result, ['search', 'article', 'AKT1 E17K'])
        self.assertEqual([row['attempt'] for row in attempts], [1, 2])
        self.assertEqual([row['sources'][0]['reason'] for row in attempts], ['proxy', 'unavailable'])
        self.assertEqual([row['exit_code'] for row in attempts], [4, 4])
        self.assertTrue(all(row['elapsed_ms'] >= 0 for row in attempts))
        self.assertEqual(result.returncode, 1)

    def test_recovered_attempt_does_not_replace_failed_diagnostics(self):
        result, _ = self.run_check({'AKT1 E17K': [4, FAILED_SOURCE, 'once']})
        attempts = self.attempts(result, ['search', 'article', 'AKT1 E17K'])
        self.assertEqual([row['status'] for row in attempts], ['failed', 'complete'])
        self.assertEqual([row['exit_code'] for row in attempts], [4, 0])
        self.assertEqual(result.returncode, 0)

    def test_attempt_output_is_bounded_and_omits_payloads(self):
        secret = 'sentinel-must-not-be-logged'
        sources = [{'source': 'pubmed', 'state': 'failed', 'reason': 'tls', 'term': secret,
                    'notices': [{'kind': 'request-guidance', 'message': secret},
                                {'kind': 'request-guidance',
                                 'message': 'TLS verification failed: unknown issuer'}]}] * 200
        body = {**FAILED_SOURCE, 'sources': sources, 'result': {'body': secret},
                'error': {'code': 'request.failed', 'message': secret}, 'headers': {'key': secret}}
        result, _ = self.run_check({'AKT1 E17K': [4, body]})
        attempts = self.attempts(result, ['search', 'article', 'AKT1 E17K'])
        self.assertEqual(len(attempts), 2)
        for row in attempts:
            encoded = json.dumps(row)
            self.assertNotIn(secret, encoded)
            self.assertTrue(row['truncated'])
            self.assertIn('unknown issuer', encoded)
        for line in result.stdout.splitlines():
            if line.startswith('{'):
                self.assertLessEqual(len(line.encode('utf-8')), 8192)

    def test_malformed_and_timeout_attempts_keep_only_fixed_diagnostics(self):
        result, _ = self.run_check({'MOLM13': [0, 'not-json-body-sentinel']})
        self.assertNotIn('not-json-body-sentinel', result.stdout)
        attempts = self.attempts(result, ['search', 'cell-line', 'MOLM13'])
        self.assertEqual(len(attempts), 2)
        for row in attempts:
            self.assertTrue(row['malformed_reply'])
            self.assertNotIn('sentinel', json.dumps(row))
        result, _ = self.run_check({'MOLM13': [0, 'hang']}, extra=('--timeout', '1'))
        attempts = self.attempts(result, ['search', 'cell-line', 'MOLM13'])
        self.assertEqual(len(attempts), 2)
        self.assertTrue(all(row['timeout'] and row['exit_code'] is None for row in attempts))

    def test_a_build_failure_is_one_line_and_runs_nothing(self):
        result, rows = self.run_check(build='fail')
        self.assertEqual(result.returncode, 1)
        self.assertEqual(rows, [])
        self.assertIn(': mismatched types\n', result.stdout)

    def test_no_key_reaches_the_product(self):
        _, rows = self.run_check()
        for row in rows:
            self.assertNotIn('NCBI_API_KEY', row['env'])
            self.assertNotIn('ONCOKB_TOKEN', row['env'])

    def test_kegg_and_litsense_are_never_called(self):
        for row in CHECKS:
            words = ' '.join(row).lower()
            self.assertNotIn('kegg', words)
            self.assertNotIn('litsense', words)


if __name__ == '__main__':
    unittest.main()
