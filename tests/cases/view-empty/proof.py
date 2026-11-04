"""Retain raw installed JSON comparisons and scoped duplication findings."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path('target/view-empty')
BASELINE = '0e42cd3d4d04abc078ea00a21e9817c3e9f4edcd'


def prepare():
    expected = '4789c874b0356f2c5ea525a075e5b76615872df32ff77ebeb3453ca21ef0459f'
    binding = json.loads((ROOT / 'baseline-source-bindings.json').read_text())
    assert hashlib.sha256((ROOT / 'baseline-biomcp').read_bytes()).hexdigest() == expected
    assert binding['binary_sha256'] == expected
    assert binding['binary_source_commit'] == BASELINE
    cases = ROOT / 'json-cases'
    cases.mkdir(exist_ok=True)
    for path in Path('tests/cases/view-empty').glob('*.yaml'):
        case = json.loads(path.read_text())
        case['steps'] = case['steps'][:1]
        (cases / path.name).write_text(json.dumps(case))


def outputs(directory):
    result = {}
    for path in directory.glob('*.json'):
        record = json.loads(path.read_text())
        for execution in record.get('executions', []):
            args = execution.get('args', [])
            if execution.get('surface') != 'cli' or '--format' not in args:
                continue
            end = args.index('--format') + 2
            if args[end - 1] == 'json':
                result[tuple(args[:end])] = bytes(execution['stdout'])
    return result


def compare():
    baseline = outputs(ROOT / 'baseline-transcripts')
    current = outputs(ROOT / 'transcripts')
    assert len(baseline) == len(current) == 4
    assert baseline == current, 'Installed JSON output changed'
    print('Four installed JSON replies are byte-identical to the source-bound retained baseline.')


def scans():
    packages = Path.home().glob('.npm/_npx/*/node_modules/jscpd/package.json')
    scanner = next(p.parent.parent / '.bin/jscpd' for p in packages
                   if json.loads(p.read_text()).get('version') == '4.0.5')
    baseline = ROOT / 'baseline-source'
    baseline.mkdir(exist_ok=True)
    archive = subprocess.check_output(['git', 'archive', BASELINE, 'crates/biomcp/src/view'])
    subprocess.run(['tar', '-xf', '-', '-C', str(baseline)], input=archive, check=True)
    for label, prefix in [('baseline', baseline), ('current', Path('.'))]:
        subprocess.run([str(scanner), str(prefix / 'crates/biomcp/src/view'), '--format', 'rust',
                        '--max-size', '10mb', '--reporters', 'json', '--output',
                        str(ROOT / ('jscpd-' + label))], check=True)


{'prepare': prepare, 'compare': compare, 'scans': scans}[sys.argv[1]]()
