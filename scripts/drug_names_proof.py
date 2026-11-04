"""Reuse the source-bound public CLI for exact native-ID reply comparison."""
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import zipfile
import urllib.error
import urllib.parse
import urllib.request

PROOFS = Path('target/drug-names-proof')
ARCHIVE_SHA = 'c322e70ca7b9c3e5039740debf8575257d605d01f354261b28e121ac59eaf6f3'
BASELINE_SOURCE = 'd349f24892deac67c3b9bb836b708a487b5ea442'
BASELINE_BINARY_SHA = 'b50998e586ccc38a2a4f6d851cce229651e380560814e5d4212d4694f89c7660'


def extract():
    archive = PROOFS / 'baseline.zip'
    archive.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(['scp', 'yellow:/home/ian/workspace/data/biomcp-redux-evidence/0294/public-rehearsal-41806259796/native-macosx_11_0_arm64.zip', archive], check=True)
    if hashlib.sha256(archive.read_bytes()).hexdigest() != ARCHIVE_SHA:
        raise ValueError('public rehearsal archive differs from its receipt')
    with zipfile.ZipFile(archive) as bundle:
        with tarfile.open(fileobj=io.BytesIO(bundle.read('biomcp-darwin-arm64.tar.gz'))) as native:
            members = [m for m in native.getmembers() if m.isfile() and Path(m.name).name == 'biomcp']
            if len(members) != 1:
                raise ValueError('public native archive must contain one CLI')
            data = native.extractfile(members[0]).read()
    if hashlib.sha256(data).hexdigest() != BASELINE_BINARY_SHA:
        raise ValueError('public CLI differs from its source receipt')
    binary = PROOFS / 'baseline-bin/biomcp'
    binary.parent.mkdir(exist_ok=True)
    binary.write_bytes(data)
    binary.chmod(0o755)
    archive.unlink()
    print(f'baseline CLI {BASELINE_SOURCE} {BASELINE_BINARY_SHA}')


def native_replies(directory):
    replies = {}
    for path in directory.glob('*.json'):
        record = json.loads(path.read_text())
        if 'drug/name-get' in record.get('key', ''):
            continue
        for execution in record.get('executions', []):
            if not isinstance(execution, dict):
                continue
            args = execution.get('args', [])
            if execution.get('surface') != 'cli' or not any(args[i:i + 2] == ['get', 'drug'] and i + 2 < len(args) and not args[i + 2].startswith('--') for i in range(len(args) - 1)):
                continue
            key = record['key']
            if key.startswith('trial/'):
                # The recorded-next-step trial owns replay folders outside the child home.
                normalized = [arg.replace(execution['root'], '<root>').replace(execution['checkout'], '<checkout>') for arg in args]
                for index, arg in enumerate(normalized[:-1]):
                    if arg in ['--replay', '--record', '--case-cache']:
                        normalized[index + 1] = '<runtime-path>'
                key = key.rsplit('/', 2)[0] + '/' + json.dumps(normalized)
            replies.setdefault(key, []).append((execution.get('sequence_start', 0), execution['exit'], bytes(execution['stdout'])))
    return {key: [(exit_code, stdout) for _, exit_code, stdout in sorted(rows)] for key, rows in replies.items()}


def compare():
    baseline = native_replies(PROOFS / 'baseline-transcripts')
    current = native_replies(PROOFS / 'transcripts')
    if not baseline or baseline.keys() != current.keys():
        raise ValueError('native-ID case inventory differs')
    changed = [name for name in baseline if baseline[name] != current[name]]
    if changed:
        raise ValueError(f'native-ID replies changed: {changed}')
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    binary = Path('target/drug-names-installed/bin/biomcp')
    report = {'source_commit': revision, 'binary_source_commit': (PROOFS / 'source.txt').read_text().strip(),
              'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
              'baseline_source': BASELINE_SOURCE, 'baseline_binary_sha256': BASELINE_BINARY_SHA,
              'native_get_reply_count': sum(map(len, baseline.values())), 'native_get_byte_parity': True}
    (PROOFS / 'source-bindings.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


def witness():
    destination = PROOFS / 'source-witness'
    destination.mkdir(parents=True, exist_ok=True)
    for suffix in ['.exact', '']:
        selector = f'products.brand_name{suffix}:"imatinib" OR products.active_ingredients.name{suffix}:"imatinib"'
        url = 'https://api.fda.gov/drug/drugsfda.json?' + urllib.parse.urlencode({'search': selector, 'limit': 50})
        try:
            with urllib.request.urlopen(url, timeout=30) as reply:
                body, status = reply.read(), reply.status
        except urllib.error.HTTPError as error:
            body, status = error.read(), error.code
        name = 'exact' if suffix else 'words'
        (destination / f'{name}.json').write_bytes(body)
        data = json.loads(body)
        facts = [{'application': row.get('application_number'),
                  'brands': [p.get('brand_name') for p in row.get('products', [])],
                  'ingredients': [i.get('name') for p in row.get('products', []) for i in p.get('active_ingredients', [])],
                  'openfda_names': {key: row.get('openfda', {}).get(key) for key in ['brand_name', 'generic_name', 'substance_name']}}
                 for row in data.get('results', [])]
        metadata = {'source': 'drugsfda', 'url': url, 'status': status,
                    'source_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
                    'body_sha256': hashlib.sha256(body).hexdigest(), 'name_fields': facts}
        (destination / f'{name}-metadata.json').write_text(json.dumps(metadata, indent=2) + '\n')
        print(json.dumps(metadata, indent=2))


def live():
    destination = (PROOFS / 'live').resolve()
    destination.mkdir(parents=True, exist_ok=True)
    config = destination / 'config.json'
    config.write_text('{}\n')
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    from release.smoke import environment
    child_environment = environment(destination)
    binary = Path('target/drug-names-installed/bin/biomcp').resolve()
    common = [str(binary), '--config', str(config), '--fresh']
    replies = []
    for index, name in enumerate(['imatinib', 'Gleevec', 'imatinib mesylate', 'no such medicine anywhere']):
        args = ['get', 'drug', name, 'approvals', '--section-limit', '1']
        if name == 'Gleevec':
            args += ['--source', 'drugsfda']
        result = subprocess.run([*common, '--record', str(destination / f'recordings-{index}'), *args], env=child_environment, capture_output=True, check=False)
        (destination / f'{index}-stdout.json').write_bytes(result.stdout)
        (destination / f'{index}-stderr.txt').write_bytes(result.stderr)
        value = json.loads(result.stdout)
        if index < 3:
            if result.returncode != 0 or value.get('result', {}).get('identity', {}).get('id') != 'NDA021335':
                raise ValueError(f'live name did not select the original Gleevec application; see {destination}/{index}-stdout.json')
            identity = value['result']['identity']
            replies.append({'input': name, 'id': identity['id'], 'name_resolution': identity['name_resolution']})
        else:
            expected = ['search', 'drug', name, '--source', 'drugsfda']
            if result.returncode != 2 or value.get('next', [{}])[0].get('args') != expected:
                raise ValueError('unresolved live name lost the exact next search command')
    binding = {'source_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
               'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
               'binary_source_commit': (PROOFS / 'source.txt').read_text().strip(), 'replies': replies}
    (destination / 'source-bindings.json').write_text(json.dumps(binding, indent=2) + '\n')
    print(json.dumps(binding, indent=2))


def upgrade():
    destination = (PROOFS / 'ema-upgrade').resolve()
    destination.mkdir(parents=True, exist_ok=True)
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    from release.smoke import environment
    env = environment(destination)
    baseline = (PROOFS / 'baseline-bin/biomcp').resolve()
    current = Path('target/drug-names-installed/bin/biomcp').resolve()
    cache = str(destination / 'case-cache')
    fixture = str(Path('tests/replies/drug/name-get').resolve())
    commands = [(baseline, ['--replay', fixture, '--case-cache', cache, 'sync', 'ema']),
                (baseline, ['--offline', '--case-cache', cache, 'search', 'drug', 'Example ingredient', '--source', 'ema']),
                (current, ['--offline', '--case-cache', cache, 'get', 'drug', 'Example ingredient', 'regulatory'])]
    replies = []
    for index, (binary, args) in enumerate(commands):
        result = subprocess.run([str(binary), *args], env=env, capture_output=True, check=False)
        (destination / f'{index}-stdout.json').write_bytes(result.stdout)
        (destination / f'{index}-stderr.txt').write_bytes(result.stderr)
        if result.returncode != 0:
            raise ValueError(f'EMA upgrade command failed; see {destination}/{index}-stdout.json')
        replies.append(json.loads(result.stdout))
    if replies[1].get('result', {}).get('total') != 0:
        raise ValueError('baseline EMA bundle did not reproduce the missing generic index')
    if replies[2].get('result', {}).get('identity', {}).get('id') != 'EMEA/H/C/000406':
        raise ValueError('current name get did not read the legacy stored generic field')
    binding = {'source_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
               'binary_source_commit': (PROOFS / 'source.txt').read_text().strip(),
               'baseline_source': BASELINE_SOURCE, 'legacy_generic_search_total': 0,
               'current_get_id': replies[2]['result']['identity']['id']}
    (destination / 'source-bindings.json').write_text(json.dumps(binding, indent=2) + '\n')
    print(json.dumps(binding, indent=2))


def scan():
    packages = (Path.home() / '.npm/_npx').glob('*/node_modules/jscpd/package.json')
    scanner = next((p.parent.parent / '.bin/jscpd' for p in packages
                    if json.loads(p.read_text()).get('version') == '4.0.5'), None)
    if scanner is None:
        raise ValueError('cached jscpd 4.0.5 is required')
    subprocess.run([scanner, '--version'], check=True)
    paths = subprocess.check_output(['git', 'diff', '--name-only', '5dfcab7ed', 'HEAD', '--', 'crates'], text=True).splitlines()
    paths = [p for p in paths if p.endswith('.rs')]
    old_paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', '5dfcab7ed', '--', *paths], text=True).splitlines()
    baseline = PROOFS / 'baseline-source'
    baseline.mkdir(parents=True, exist_ok=True)
    archive = subprocess.run(['git', 'archive', '5dfcab7ed', *old_paths], check=True, stdout=subprocess.PIPE)
    subprocess.run(['tar', '-xf', '-', '-C', baseline], input=archive.stdout, check=True)
    for kind in ['product', 'tests']:
        current = [p for p in paths if ('/tests/' in p) == (kind == 'tests')]
        prior = [str(baseline / p) for p in old_paths if ('/tests/' in p) == (kind == 'tests')]
        for label, selected in [('baseline', prior), ('current', current)]:
            if selected:
                subprocess.run([scanner, *selected, '--format', 'rust', '--max-size', '10mb',
                                '--reporters', 'json', '--output', PROOFS / f'jscpd-{label}-{kind}'], check=True)
    subprocess.run(['ruff', 'check', '--select', 'F401,F841,ERA,BLE,E722,S110', 'scripts/drug_names_proof.py'], check=True)


if __name__ == '__main__':
    {'extract': extract, 'compare': compare, 'witness': witness, 'live': live, 'scan': scan, 'upgrade': upgrade}[sys.argv[1]]()
