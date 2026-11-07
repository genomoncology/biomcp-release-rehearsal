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
        return 'exit {}: {}'.format(result.returncode, result.stderr or result.stdout)
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


def check(executable, args, environment, timeout):
    try:
        result = subprocess.run([executable, '--format', 'json', *args], env=environment,
                                capture_output=True, text=True, timeout=timeout, check=False)
    except subprocess.TimeoutExpired:
        return 'no reply in {} seconds'.format(timeout)
    except OSError as error:
        return str(error)
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
            problem = check(executable, row, environment, args.timeout)
            if problem is None:
                print('ok ' + shlex.join(('biomcp', *row)), flush=True)
            else:
                retries.append((row, problem))
        # One slow reply is not an outage: only a command that fails twice is reported.
        if retries:
            time.sleep(args.pause * RETRY_WAIT)
        for row, first in retries:
            command = shlex.join(('biomcp', *row))
            problem = check(executable, row, environment, args.timeout)
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
