"""Run a few real commands against live sources and print one line per failure."""
import argparse
import json
import os
import shlex
import subprocess
import sys
import tempfile
import time

# One or two public, keyless commands per source family. Keep KEGG and LitSense out.
CHECKS = (
    ('get', 'gene', 'BRAF'),
    ('search', 'gene', 'KRAS'),
    ('search', 'variant', '--gene', 'EGFR', '--protein', 'L858R', '--build', 'GRCh37'),
    ('get', 'variant', 'CA123643', 'civic', '--disable-sources', 'myvariant'),
    ('search', 'article', 'AKT1 E17K'),
    ('article', 'citations', '17611497'),
    ('article', 'recommendations', '34096690', '--limit', '5'),
    ('search', 'trial', '--condition', 'melanoma', '--limit', '1'),
    ('get', 'trial', 'NCT04303780'),
    ('search', 'drug', 'imatinib', '--source', 'openfda-label', '--limit', '3'),
    ('get', 'drug', 'CHEMBL941', 'indications', '--source', 'chembl', '--section-limit', '1'),
    ('search', 'disease', 'breast cancer', '--source', 'ols', '--ontology', 'mondo', '--limit', '1'),
    ('get', 'disease', 'MONDO:0007254', 'synonyms', '--source', 'mydisease', '--section-limit', '2'),
    ('search', 'pathway', 'Signaling by EGFR'),
    ('get', 'pathway', 'R-HSA-5673001', 'summary'),
    ('search', 'protein', '--gene', 'EGFR', '--reviewed'),
    ('get', 'protein', 'P00533', '--section', 'function', '--section-limit', '2'),
    ('search', 'adverse-event', 'pembrolizumab', '--source', 'faers', '--limit', '1'),
    ('search', 'diagnostic', '--source', 'fda-device-recall', '--initiated-from', '2024-01-01',
     '--initiated-to', '2024-12-31', '--limit', '1'),
    ('search', 'cell-line', 'MOLM13'),
    ('get', 'cell-line', 'CVCL_0004', 'identity', '--section-limit', '2'),
    ('search', 'phenotype', 'headache', '--source', 'monarch'),
    ('search', 'gwas', '--variant', 'rs2981582'),
    ('search', 'pgx', '--gene', 'DPYD', '--source', 'cpic'),
)
BUILD = ('cargo', 'build', '--locked', '-p', 'biomcp', '--message-format=json')
# Only these reach the product, so no source key can enter a check or its output.
PASSED_ENVIRONMENT = ('PATH', 'LANG', 'http_proxy', 'HTTP_PROXY', 'https_proxy', 'HTTPS_PROXY',
                      'no_proxy', 'NO_PROXY')
ATTEMPT_LIMIT = 8192
TLS_MESSAGES = {
    "TLS verification failed: " + cause for cause in
    ("unknown issuer", "CA used as end entity", "name mismatch", "expired",
     "not yet valid", "revoked", "other")
}
REASON_LIMIT = 200
RETRY_WAIT = 30  # Pauses to wait before the second try.


def one_line(text):
    return ' '.join(str(text).split())[:REASON_LIMIT] or 'no output'


def build():
    """Return the built executable, or raise with the compiler's first error."""
    result = subprocess.run(BUILD, capture_output=True, text=True, check=False)
    executable, error = None, None
    for line in result.stdout.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        target = message.get('target') or {}
        if message.get('executable') and target.get('name') == 'biomcp':
            executable = message['executable']
        details = message.get('message') or {}
        if error is None and details.get('level') == 'error':
            error = details.get('message')
    if result.returncode or not executable:
        lines = result.stderr.strip().splitlines()
        raise ValueError(error or (lines[-1] if lines else 'exit ' + str(result.returncode)))
    return executable


def reason(result, searching):
    """Name why a command failed from its reply, or return None when it answered."""
    try:
        reply = json.loads(result.stdout)
    except ValueError:
        reply = None
    if not isinstance(reply, dict):
        return 'exit {}: malformed reply'.format(result.returncode)
    failed = [row['source'] + ': ' + row.get('reason', 'failed') for row in reply.get('sources', [])
              if row.get('state') == 'failed']
    if failed:
        return ', '.join(failed)
    error = reply.get('error')
    if isinstance(error, dict):
        return error.get('message', error.get('code', 'error reply'))
    if result.returncode or reply.get('status') != 'complete':
        return 'exit {}, status {}'.format(result.returncode, reply.get('status'))
    page = reply.get('page')
    if isinstance(page, dict) and page.get('returned') == 0:
        return 'no results'
    # Some searches have no top-level page, and an empty list may be left out of the reply.
    body = reply.get('result')
    if searching and not (isinstance(body, dict) and body.get('items')):
        return 'no results'
    return None


def source_diagnostic(source):
    """Keep fixed status fields and recognized guidance, never source text."""
    row = {key: source[key][:128] for key in ('source', 'state', 'reason')
           if isinstance(source.get(key), str)}
    notices = source.get('notices', [])
    for notice in notices if isinstance(notices, list) else []:
        if not isinstance(notice, dict) or notice.get('kind') != 'request-guidance':
            continue
        if isinstance(notice.get('message'), str) and notice['message'] in TLS_MESSAGES:
            row['tls_guidance'] = notice['message']
        delay = notice.get('retry_after_seconds')
        if type(delay) is int and 0 <= delay <= 2**32:
            row['retry_after_seconds'] = delay
    return row


def attempt_record(args, attempt, started, result=None, failure=None):
    """Flush a bounded JSON record before any retry can replace the evidence."""
    row = {'kind': 'live-check-attempt', 'command': list(args), 'attempt': attempt,
           'elapsed_ms': round((time.monotonic() - started) * 1000),
           'exit_code': result.returncode if result is not None else None,
           'timeout': failure == 'timeout', 'malformed_reply': False, 'truncated': False,
           'sources': []}
    if result is not None:
        try:
            reply = json.loads(result.stdout)
        except ValueError:
            reply = None
        row['malformed_reply'] = not isinstance(reply, dict)
        if isinstance(reply, dict):
            if isinstance(reply.get('status'), str):
                row['status'] = reply['status'][:128]
                row['truncated'] |= len(reply['status']) > 128
            error = reply.get('error')
            if isinstance(error, dict) and isinstance(error.get('code'), str):
                row['error_code'] = error['code'][:128]
                row['truncated'] |= len(error['code']) > 128
            sources = reply.get('sources', [])
            if isinstance(sources, list):
                row['truncated'] |= len(sources) > 64
                row['truncated'] |= any(
                    isinstance(source, dict) and any(
                        isinstance(source.get(key), str) and len(source[key]) > 128
                        for key in ('source', 'state', 'reason')) for source in sources[:64])
                row['sources'] = [source_diagnostic(source) for source in sources[:64]
                                  if isinstance(source, dict)]
    if failure:
        row['process_error'] = failure
    encoded = json.dumps(row, separators=(',', ':'))
    while len(encoded.encode('utf-8')) > ATTEMPT_LIMIT:
        row['truncated'] = True
        row['sources'].pop()
        encoded = json.dumps(row, separators=(',', ':'))
    print(encoded, flush=True)


def check(executable, args, environment, timeout, attempt):
    started = time.monotonic()
    try:
        result = subprocess.run([executable, '--format', 'json', *args], env=environment,
                                capture_output=True, text=True, timeout=timeout, check=False)
    except subprocess.TimeoutExpired:
        attempt_record(args, attempt, started, failure='timeout')
        return 'no reply in {} seconds'.format(timeout)
    except OSError as error:
        attempt_record(args, attempt, started, failure=type(error).__name__)
        return str(error)
    attempt_record(args, attempt, started, result=result)
    return reason(result, args[0] == 'search')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pause', type=float, default=1.0, help='Seconds between commands')
    parser.add_argument('--timeout', type=float, default=120.0, help='Seconds for one command')
    args = parser.parse_args()
    try:
        executable = build()
    except (OSError, ValueError) as error:
        print('FAIL {}: {}'.format(shlex.join(BUILD), one_line(error)))
        print('live check: not run, the build failed')
        return 1
    failures, retries, started = [], [], time.monotonic()
    with tempfile.TemporaryDirectory(prefix='biomcp-live-check-') as home:
        environment = {name: os.environ[name] for name in PASSED_ENVIRONMENT if name in os.environ}
        environment.update(HOME=home, XDG_CACHE_HOME=home + '/cache', XDG_CONFIG_HOME=home + '/config')
        for index, row in enumerate(CHECKS):
            if index:
                time.sleep(args.pause)
            problem = check(executable, row, environment, args.timeout, 1)
            if problem is None:
                print('ok ' + shlex.join(('biomcp', *row)), flush=True)
            else:
                retries.append((row, problem))
        # One slow reply is not an outage: only a command that fails twice is reported.
        if retries:
            time.sleep(args.pause * RETRY_WAIT)
        for row, first in retries:
            command = shlex.join(('biomcp', *row))
            problem = check(executable, row, environment, args.timeout, 2)
            if problem is None:
                print('ok {} on its second try, first: {}'.format(command, one_line(first)), flush=True)
            else:
                failures.append('FAIL {}: {}'.format(command, one_line(problem)))
    # Failures come last because the failure mail carries only the end of the log.
    for line in failures:
        print(line)
    print('live check: {} of {} commands failed in {:.0f} seconds'.format(
        len(failures), len(CHECKS), time.monotonic() - started))
    return int(bool(failures))


if __name__ == '__main__':
    sys.exit(main())
