"""Exercise source label pages against one installed CLI and retain source bindings."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.parse
import urllib.request

PROOF = Path('target/label-search-proof')


def focused():
    artifacts = [json.loads(line) for line in (PROOF / 'build.jsonl').read_text().splitlines()]
    executable = next(Path(row['executable']) for row in artifacts
                      if row.get('reason') == 'compiler-artifact' and row.get('executable')
                      and row['target']['name'] == 'biomcp' and row['target']['kind'] == ['bin'])
    runner = next(Path(row['executable']) for row in artifacts
                  if row.get('reason') == 'compiler-artifact' and row.get('executable')
                  and row['target']['name'] == 'cases')
    installed = PROOF / 'installed/biomcp'
    installed.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(executable, installed)
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    (PROOF / 'source-bindings.json').write_text(json.dumps({
        'source_commit': revision, 'binary_sha256': hashlib.sha256(installed.read_bytes()).hexdigest(),
        'executable': str(installed.resolve()), 'runner': str(runner)}, indent=2) + '\n')
    command = [sys.executable, 'scripts/trials.py', '--runner', str(runner),
               '--executable', str(installed), '--installed']
    listed = subprocess.check_output([*command, '--list'], text=True)
    (PROOF / 'actual-list.txt').write_text(listed)
    names = [line.removesuffix(': test') for line in listed.splitlines()
             if line.endswith(': test') and ('drug/label-' in line or 'drug/openfda-label' in line
                                              or 'drug/help' in line)]
    if not names:
        raise ValueError('no label trials discovered')
    subprocess.run([*command, *names], check=True)


def live():
    executable = (PROOF / 'installed/biomcp').resolve()
    for query in ['imatinib', 'chronic myeloid leukemia']:
        for limit in ([3, 10] if query == 'imatinib' else [10]):
            seen = set()
            offset = 0
            total = None
            with tempfile.TemporaryDirectory(prefix='label-search-live-') as isolated:
                env = dict(os.environ, HOME=isolated, XDG_CONFIG_HOME=isolated,
                           XDG_CACHE_HOME=isolated, XDG_DATA_HOME=isolated)
                while True:
                    command = [str(executable), 'search', 'drug', query, '--source', 'openfda-label',
                               '--limit', str(limit), '--offset', str(offset)]
                    stem = f'{query.replace(" ", "-")}-{limit}-{offset}'
                    revision = json.loads((PROOF / 'source-bindings.json').read_text())['source_commit']
                    recording = PROOF / 'source-recordings' / revision / stem
                    recording.parent.mkdir(parents=True, exist_ok=True)
                    command[1:1] = ['--record', str(recording.resolve())]
                    result = subprocess.run(command, env=env, capture_output=True, check=False)
                    (PROOF / f'{stem}.json').write_bytes(result.stdout)
                    (PROOF / f'{stem}-stderr.txt').write_bytes(result.stderr)
                    result.check_returncode()
                    reply = json.loads(result.stdout)
                    page = reply['page']
                    items = reply['result'].get('items', [])
                    current = page['total']
                    if total is not None and total != current:
                        raise ValueError('native total changed across pages')
                    total = current
                    if len(items) != min(limit, total - offset) or any(not row.get('name') for row in items):
                        raise ValueError('page is incomplete or unnamed')
                    ids = [row['id'] for row in items]
                    if len(set(ids)) != len(ids) or seen.intersection(ids):
                        raise ValueError('source page repeats an SPL set')
                    seen.update(ids)
                    if offset == 0:
                        urls = set()
                        for path in recording.rglob('*.json'):
                            recorded = json.loads(path.read_text())
                            for exchange in recorded['exchanges']:
                                value = exchange['body']['value']
                                body = json.loads(value) if isinstance(value, str) else value
                                if exchange['status'] == 200 and body.get('results'):
                                    urls.add(recorded['url'])
                        if len(urls) != 1:
                            raise ValueError('expected one populated source query recording')
                        parsed = urllib.parse.urlsplit(next(iter(urls)))
                        params = dict(urllib.parse.parse_qsl(parsed.query))
                        params['limit'] = '1'
                        params.pop('skip', None)
                        url = urllib.parse.urlunsplit(parsed._replace(query=urllib.parse.urlencode(params)))
                        with urllib.request.urlopen(url) as response:
                            native = response.read()
                        (PROOF / f'{stem}-source.json').write_bytes(native)
                        native_total = json.loads(native)['meta']['results']['total']
                        (PROOF / f'{stem}-source-selector.json').write_text(json.dumps({'url': url, 'total': native_total}))
                        if native_total != total:
                            raise ValueError('source total differs for the same recorded query')
                    offset += len(items)
                    if offset >= total or limit == 3 or query != 'imatinib':
                        break
            print(f'{query} limit={limit} total={total} observed_unique={len(seen)} complete_paging={offset >= total}')


def scan():
    baseline = '26d580e5'
    scanner = next((p.parent.parent / '.bin/jscpd'
                    for p in (Path.home() / '.npm/_npx').glob('*/node_modules/jscpd/package.json')
                    if json.loads(p.read_text()).get('version') == '4.0.5'), None)
    if scanner is None:
        raise ValueError('cached jscpd 4.0.5 is required')
    subprocess.run([scanner, '--version'], check=True)
    paths = subprocess.check_output(['git', 'diff', '--name-only', baseline, 'HEAD', '--',
                                    'crates/biomcp-adapters/src/openfda_label',
                                    'crates/biomcp-types/src/drug/query.rs'], text=True).splitlines()
    prior = []
    for name in paths:
        target = PROOF / 'baseline-source' / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(subprocess.check_output(['git', 'show', baseline + ':' + name]))
        prior.append(str(target))
    for label, selected in [('baseline', prior), ('current', paths)]:
        subprocess.run([scanner, *selected, '--format', 'rust', '--max-size', '10mb',
                        '--reporters', 'json', '--output', PROOF / f'jscpd-{label}'], check=True)
    subprocess.run(['ruff', 'check', '--select', 'F401,F841,ERA,BLE,E722,S110',
                    'scripts/label_search_proof.py'], check=True)


def diagnose():
    executable = (PROOF / 'installed/biomcp').resolve()
    with tempfile.TemporaryDirectory(prefix='label-search-diagnostic-') as isolated:
        env = dict(os.environ, HOME=isolated, XDG_CONFIG_HOME=isolated,
                   XDG_CACHE_HOME=isolated, XDG_DATA_HOME=isolated)
        result = subprocess.run([str(executable), '--record', str((PROOF / 'diagnostic-recordings').resolve()),
                                 'search', 'drug', 'imatinib', '--source', 'openfda-label',
                                 '--limit', '3', '--offset', '12'], env=env, capture_output=True)
    (PROOF / 'diagnostic-stdout.json').write_bytes(result.stdout)
    (PROOF / 'diagnostic-stderr.txt').write_bytes(result.stderr)
    print(result.stdout.decode())
    result.check_returncode()


def retain():
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    destination = f'/home/ian/workspace/data/biomcp-redux-evidence/0605/{revision}'
    archive = PROOF / 'proof.tar.gz'
    with tarfile.open(archive, 'w:gz') as bundle:
        for item in sorted(PROOF.iterdir()):
            if item.name not in ['installed', 'proof.tar.gz']:
                bundle.add(item, arcname=item.name)
    subprocess.run(['ssh', 'yellow', 'mkdir', '-p', destination], check=True)
    subprocess.run(['scp', str(archive), 'yellow:' + destination + '/proof.tar.gz'], check=True)
    print('Evidence:', 'yellow:' + destination + '/proof.tar.gz')


if __name__ == '__main__':
    {'focused': focused, 'live': live, 'scan': scan, 'retain': retain, 'diagnose': diagnose}[sys.argv[1]]()
