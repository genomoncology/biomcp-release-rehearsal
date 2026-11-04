"""Exercise installed boolean request mapping and unchanged successful replies."""
import hashlib
import io
import json
import os
import shutil
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
from urllib.parse import urlencode, quote
from zipfile import ZipFile

BASELINE = 'd349f24892deac67c3b9bb836b708a487b5ea442'
BASELINE_HASH = 'b50998e586ccc38a2a4f6d851cce229651e380560814e5d4212d4694f89c7660'


def main():
    binary, stage = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    archive = stage / 'baseline.zip'
    if not archive.is_file():
        raise SystemExit('Stage the retained rehearsal archive at ' + str(archive))
    with ZipFile(archive) as outer:
        with tarfile.open(fileobj=io.BytesIO(outer.read('biomcp-darwin-arm64.tar.gz'))) as native:
            members = [member for member in native.getmembers() if Path(member.name).name == 'biomcp' and member.isfile()]
            assert len(members) == 1
            data = native.extractfile(members[0]).read()
    assert hashlib.sha256(data).hexdigest() == BASELINE_HASH
    baseline = stage / 'baseline-biomcp'
    baseline.write_bytes(data)
    baseline.chmod(0o755)
    report = {'baseline_revision': BASELINE, 'baseline_sha256': BASELINE_HASH, 'requests': []}
    with tempfile.TemporaryDirectory(prefix='biomcp-usage-') as scratch:
        home = Path(scratch)
        replay = home / 'replay' / 'ctgov'
        replay.mkdir(parents=True)
        version = Path('tests/replies/grammar/boolean-literals/ctgov/e6d9a4526e6cc39dfac0cef1b72580f7020a33c464d145ab8e6a77f411ff0fff.json')
        shutil.copy2(version, replay / version.name)
        env = {key: value for key, value in os.environ.items() if key in {'PATH', 'TMPDIR', 'SystemRoot', 'WINDIR'}}
        env.update(HOME=str(home), XDG_CONFIG_HOME=str(home / 'config'), XDG_CACHE_HOME=str(home / 'cache'))
        for results, total in [(False, 0), (True, 1)]:
            params = {'countTotal': 'true', 'fields': 'NCTId,BriefTitle,OverallStatus,Phase,Condition,InterventionName,LastUpdatePostDate,HasResults', 'pageSize': '10'}
            if results:
                params['filter.advanced'] = 'AREA[ResultsFirstPostDate]RANGE[MIN,MAX]'
            url = 'https://clinicaltrials.gov/api/v2/studies?' + urlencode(sorted(params.items()), quote_via=quote)
            recording = {'method': 'GET', 'source': 'ctgov', 'url': url, 'exchanges': [{'fetched': 1793203200, 'status': 200, 'content_type': 'application/json', 'delay_ms': 1, 'retry_after_ms': None, 'body': {'kind': 'json', 'value': {'totalCount': total, 'studies': []}}}]}
            key = hashlib.sha256(b'GET\0' + url.encode()).hexdigest()
            (replay / (key + '.json')).write_text(json.dumps(recording))
            report['requests'].append({'has_results': results, 'url': url})
        for entity, query, source, flag, field in [
            ('adverse-event', ['pump'], 'maude', '--reaction', 'Headache'),
            ('drug', ['artemether'], 'who-pq', '--initiated-from', '2026-01-01'),
            ('diagnostic', [], 'fda-cdx', '--gene', 'BRAF'),
        ]:
            args = ['search', entity, *query, '--source', source, flag, field]
            for label, executable, option in [('baseline', baseline, '--source'), ('candidate', binary, flag)]:
                result = subprocess.run([str(executable), *args], env=env, capture_output=True)
                assert result.returncode == 2, (label, entity, result.stdout)
                assert json.loads(result.stdout)['error']['option'] == option
                (stage / (label + '-' + entity + '.json')).write_bytes(result.stdout)
        outputs = {}
        for label, executable, flags, total in [
            ('literal-true', binary, ['--has-results', 'true'], 1),
            ('literal-false', binary, ['--has-results', 'false'], 0),
            ('bare', binary, ['--has-results'], 1),
            ('omitted', binary, [], 0),
            ('baseline-bare', baseline, ['--has-results'], 1),
            ('baseline-omitted', baseline, [], 0),
        ]:
            args = ['search', 'trial', *flags, '--replay', str(replay.parent)]
            result = subprocess.run([str(executable), *args], env=env, capture_output=True, check=True)
            response = json.loads(result.stdout)
            assert response['page']['total'] == total, (label, response)
            assert 'query' not in response['query'], (label, response)
            outputs[label] = result.stdout
            (stage / (label + '.json')).write_bytes(result.stdout)
        for names in [('literal-true', 'bare', 'baseline-bare'), ('literal-false', 'omitted', 'baseline-omitted')]:
            assert len({outputs[name] for name in names}) == 1, names
        plain_version = subprocess.check_output([str(baseline), 'version'], env=env)
        literal_version = subprocess.check_output(
            [str(binary), '--offline', 'false', 'version', '--verbose', 'false'], env=env)
        assert literal_version == plain_version
        (stage / 'global-local-false-version.txt').write_bytes(literal_version)
        for path in [['search', 'trial'], ['search', 'drug'], ['search', 'diagnostic'],
                     ['version'], ['skill', 'install'], ['serve'], ['key', 'set']]:
            args = ['help', *path]
            before = subprocess.check_output([str(baseline), *args], env=env)
            after = subprocess.check_output([str(binary), *args], env=env)
            assert before == after, path
    report['successful_replies_byte_identical'] = True
    (stage / 'request-parity.json').write_text(json.dumps(report, indent=2) + '\n')
    print('installed boolean requests and successful reply parity passed')


if __name__ == '__main__':
    main()
