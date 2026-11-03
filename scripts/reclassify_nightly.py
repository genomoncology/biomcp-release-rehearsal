"""Replay every stored nightly report offline; print one summary per retained night."""
import argparse
import datetime
import json
from pathlib import Path
import shlex
import subprocess

from nightly_live_check import classify_case, classify_report


def replay(rows, since):
    previous, results = [], []
    for name, raw in sorted(rows):
        if name[:8] < since:
            continue
        night = datetime.datetime.strptime(name[:8], '%Y%m%d').date()
        entry = {'run': name}
        try:
            report = json.loads(raw)
        except ValueError as error:
            # Inspect the complete trailing object, but never clear malformed stdout.
            entry['raw_error'] = str(error)
            try:
                report = json.loads(raw[raw.index('{'):])
            except (ValueError, KeyError):
                entry['failed'] = True
                results.append(entry)
                continue
        try:
            entry['failed'] = classify_report(report, previous, night) or 'raw_error' in entry
            entry['report'] = report
            previous.append((night.isoformat(), report))
        except (ValueError, KeyError, TypeError, AttributeError) as error:
            entry.update(failed=True, classification_error=str(error), report=report)
            # Old reports predate eligible counts. Inspect their retained cases
            # without inventing the missing selection proof or clearing the night.
            for row in report.get('case_sample', []):
                row['classification'] = classify_case(row)
        results.append(entry)
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host', help='SSH evidence host; omit for a local directory')
    parser.add_argument('--evidence-dir', required=True)
    parser.add_argument('--output', required=True, help='New derived JSON file; raw reports stay unchanged')
    parser.add_argument('--since', default='20261027')
    args = parser.parse_args()
    if args.host:
        reader = """import json,pathlib,sys
root=pathlib.Path(sys.argv[1])
print(json.dumps([(p.parent.name,p.read_text()) for p in sorted(root.glob('*/report.json'))]))
"""
        command = shlex.join(['python3', '-c', reader, args.evidence_dir])
        rows = json.loads(subprocess.check_output(['ssh', args.host, command], text=True))
    else:
        rows = [(p.parent.name, p.read_text()) for p in Path(args.evidence_dir).glob('*/report.json')]
    results = replay(rows, args.since)
    if not results:
        parser.error('no stored reports in requested interval')
    payload = json.dumps(results) + '\n'
    if args.host:
        writer = """import pathlib,sys
p=pathlib.Path(sys.argv[1]);p.parent.mkdir(parents=True,exist_ok=True)
with p.open('x') as target:target.write(sys.stdin.read())
"""
        subprocess.run(['ssh', args.host, shlex.join(['python3', '-c', writer, args.output])],
                       input=payload, text=True, check=True)
    else:
        with Path(args.output).open('x') as target:
            target.write(payload)
    for entry in results:
        report = entry.get('report', {})
        counts = {key: value for key, value in report.items() if key.endswith('_cases')}
        failures = [row['case'] for row in report.get('case_sample', [])
                    if row.get('classification') in ('failure', 'uncertain')]
        print(json.dumps({'run': entry['run'], 'failed': entry['failed'], 'counts': counts,
                          'retained_failures': failures,
                          'raw_error': entry.get('raw_error'),
                          'classification_error': entry.get('classification_error'),
                          'persistent_outage_sources': report.get('persistent_outage_sources', [])}))
    print('Derived evidence: ' + (args.host + ':' if args.host else '') + args.output)


if __name__ == '__main__':
    main()
