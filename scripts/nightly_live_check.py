"""Run the bounded public diagnostic once and retain verified evidence before cleanup."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tarfile
import tempfile

COMMAND = ['cargo', 'test', '--locked', '-p', 'biomcp', '--test', 'tools',
           '--', '--live-check', '--details']
FILES = ('report.json', 'build.log', 'exit-status', 'run.json')

# The receiver creates a unique destination, accepts only our named plain files,
# and verifies the sender's manifest before acknowledging durable delivery.
RECEIVE = r'''
import hashlib, json, os, pathlib, sys, tarfile
root = pathlib.Path(sys.argv[1])
root.mkdir(parents=True, exist_ok=False)
allowed = {'report.json', 'build.log', 'exit-status', 'run.json', 'manifest.json'}
seen = set()
with tarfile.open(fileobj=sys.stdin.buffer, mode='r|') as archive:
    for member in archive:
        if member.name not in allowed or member.name in seen or not member.isfile():
            raise ValueError('unexpected evidence entry')
        seen.add(member.name)
        source = archive.extractfile(member)
        with (root / member.name).open('xb') as target:
            while True:
                chunk = source.read(1024 * 1024)
                if not chunk:
                    break
                target.write(chunk)
            target.flush()
            os.fsync(target.fileno())
if seen != allowed:
    raise ValueError('incomplete evidence')
manifest_bytes = (root / 'manifest.json').read_bytes()
if hashlib.sha256(manifest_bytes).hexdigest() != sys.argv[2]:
    raise ValueError('manifest transfer mismatch')
manifest = json.loads(manifest_bytes)
if set(manifest) != allowed - {'manifest.json'}:
    raise ValueError('unexpected manifest entries')
for name, expected in manifest.items():
    digest = hashlib.sha256()
    with (root / name).open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            digest.update(chunk)
    if digest.hexdigest() != expected:
        raise ValueError('evidence checksum mismatch: ' + name)
fd = os.open(str(root), os.O_RDONLY)
try:
    os.fsync(fd)
finally:
    os.close(fd)
'''


def digest(path):
    checksum = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            checksum.update(chunk)
    return checksum.hexdigest()


def terminal(requests):
    """Ignore concurrency order and successful retries, preserving transport attempts."""
    result = {}
    for item in requests:
        identity = item['identity']
        if identity not in result or not item.get('cached') or result[identity].get('cached'):
            result[identity] = item
    return result


def cut(value):
    if isinstance(value, dict):
        return value.get('cut') is True or any(cut(item) for item in value.values())
    return isinstance(value, list) and any(cut(item) for item in value)


def discovery_sequences(request):
    provenance = request.get('provenance') or {}
    sequences = provenance.get('discoveries')
    if sequences is None:
        sequence = provenance.get('discovery')
        sequences = [sequence] if sequence else []
    if not isinstance(sequences, list):
        return {}
    result, total = {}, 0
    for sequence in sequences:
        if not isinstance(sequence, dict):
            return {}
        namespace, ids = sequence.get('namespace'), sequence.get('ids')
        if (not isinstance(namespace, str) or not namespace or len(namespace) > 128
                or namespace in result or not valid_ids(ids)):
            return {}
        total += len(ids)
        if total > 10_000:
            return {}
        result[namespace] = ids
    return result


def valid_ids(ids):
    return (isinstance(ids, list) and len(ids) <= 10_000
            and all(isinstance(value, str) and 0 < len(value) <= 256
                    and not any(ord(char) < 32 or ord(char) == 127 for char in value)
                    for value in ids) and len(ids) == len(set(ids)))


def discovery_changes(before, after):
    changes = {}
    for key in before.keys() & after.keys():
        old, new = before[key], after[key]
        left, right = discovery_sequences(old), discovery_sequences(new)
        same = (old.get('source') == new.get('source')
                and old.get('role') == new.get('role') and old.get('url') == new.get('url')
                and old.get('status') == new.get('status') == 200)
        for namespace in left.keys() | right.keys():
            changes.setdefault(namespace, []).append({
                'request': key, 'left': left.get(namespace), 'right': right.get(namespace),
                'valid': same and namespace in left and namespace in right})
    return changes


def consumed_selection(group, before, after, left, right, complete=False):
    selections, batch_sizes = [], []
    for requests in (before, after):
        rows = []
        sizes = []
        for request in requests.values():
            provenance = request.get('provenance') or {}
            inputs = provenance.get('inputs') or {}
            key = (request['source'], request['role'], inputs.get('namespace'),
                   provenance.get('input_context'))
            if key == group:
                ids = inputs.get('ids')
                if request.get('status') != 200 or not valid_ids(ids):
                    return False
                rows.extend(ids)
                sizes.append(len(ids))
        if not valid_ids(rows) or (not sizes and not complete):
            return False
        selections.append(set(rows))
        batch_sizes.append(max(sizes, default=0))
    return (selections == [set(left), set(right)]
            and (complete or (batch_sizes[0] == batch_sizes[1]
                 and (set(left) < set(right) or set(right) < set(left)))))


def dependent_change(removed, added, discoveries, before, after, peers, complete=False):
    groups = {}
    for side, requests in enumerate((removed, added)):
        for request in requests:
            inputs = request.get('provenance', {}).get('inputs')
            if not inputs:
                return 'uncertain'
            if not valid_ids(inputs.get('ids')):
                return 'failure'
            context = request.get('provenance', {}).get('input_context')
            if not context:
                return 'uncertain'
            group = (request['source'], request['role'], inputs['namespace'], context)
            groups.setdefault(group, [[], []])[side].append(inputs['ids'])
    # At a batch boundary only one side has a changed request. Include its
    # unchanged peers so the existing context and the entire selection prove
    # that the added/removed batch follows discovery rather than new parameters.
    for group, rows in groups.items():
        if all(rows):
            continue
        rows[:] = [[], []]
        for side, requests in enumerate((before.values(), after.values())):
            for request in requests:
                provenance = request.get('provenance') or {}
                inputs = provenance.get('inputs') or {}
                key = (request['source'], request['role'], inputs.get('namespace'),
                       provenance.get('input_context'))
                if key == group:
                    rows[side].append(inputs['ids'])
    for group, (old_rows, new_rows) in groups.items():
        namespace = group[2]
        old = {value for row in old_rows for value in row}
        new = {value for row in new_rows for value in row}
        candidates = discoveries.get(namespace, [])
        # A complete command may consume a wholly replaced discovery selection,
        # including no IDs. Require every input in both directions, with the same
        # context, rather than accepting arbitrary subsets or a failed provider.
        if complete and old != new and any(candidate['valid']
                and consumed_selection(group, *peers, candidate['left'], candidate['right'],
                                       complete=True) for candidate in candidates):
            continue
        if (old == new or not old_rows or not new_rows
                or sum(map(len, old_rows)) != len(old)
                or sum(map(len, new_rows)) != len(new)):
            return 'failure'
        if not candidates:
            return 'failure'
        # Transport sorts ID parameters. Require the same contiguous discovery
        # window, not merely arbitrary subsets of two changed reply bodies.
        if not any(candidate['valid']
                   and candidate['left'] != candidate['right']
                   and (same_window(old, new, candidate['left'], candidate['right'])
                        or consumed_selection(group, *peers, candidate['left'], candidate['right']))
                   for candidate in candidates):
            return 'failure'
    return 'churn'


def same_window(old, new, left, right):
    if not old or not new:
        return False
    if len(old) != len(new):
        # A source insertion/removal can rebatch the remaining discovery tail.
        # Require the entire tail on both sides and an unchanged prefix; an
        # arbitrary missing request must not become evidence of source churn.
        if not (set(left) < set(right) or set(right) < set(left)):
            return False
        start = len(left) - len(old)
        return (start >= 0 and len(right) - len(new) == start
                and set(left[:start]) == set(right[:start])
                and set(left[start:]) == old and set(right[start:]) == new)
    for start in range(len(left) - len(old) + 1):
        stop = start + len(old)
        if set(left[start:stop]) == old and set(right[start:stop]) == new:
            return True
    return False


def command_classification(capture, discoveries, peers, complete=False):
    before = terminal(capture['baseline']['requests'])
    after = terminal(capture['live']['requests'])
    statuses = [item.get('status') for item in after.values()]
    if any(type(status) is not int or not (200 <= status < 300 or status == 429)
           for status in statuses):
        return 'failure'
    removed = [before[key] for key in before.keys() - after.keys()]
    added = [after[key] for key in after.keys() - before.keys()]
    # Proven 429-only outcomes are nonfatal; unrelated plan changes still need proof.
    plan = dependent_change(removed, added, discoveries, before, after, peers, complete) if removed or added else 'healthy'
    if plan in ('failure', 'uncertain'):
        return plan
    return 'throttled' if 429 in statuses else plan


def throttled_output(command):
    """A status transition needs the checked product's source failure reason too."""
    before = command['baseline'].get('output', {})
    after = command['live'].get('output', {})
    sources = after.get('sources', [])
    failed = [source for source in sources if source.get('state') in ('failed', 'stale')]
    throttled = {request['source'] for request in terminal(command['live']['requests']).values()
                 if request.get('status') == 429}
    return (before.get('status') in ('complete', 'partial')
            and after.get('status') in ('partial', 'failed') and bool(failed)
            and all(source.get('reason') == 'rate-limited'
                    and source.get('source') in throttled for source in failed))


def omitted_diagnostics(value):
    if isinstance(value, dict):
        return any((key in ('omitted', 'drift_omitted')
                    and (type(item) is not int or item != 0))
                   or omitted_diagnostics(item) for key, item in value.items())
    return isinstance(value, list) and any(omitted_diagnostics(item) for item in value)


def complete_diagnostics(result, witness, changed):
    commands = result.get('commands', [])
    if not isinstance(commands, list) or omitted_diagnostics(commands):
        return False
    if not changed:
        return True
    if not commands or not isinstance(witness, list):
        return False
    expected = [command.get('ordinal') if isinstance(command, dict) else None
                for command in commands]
    actual = [command.get('ordinal') if isinstance(command, dict) else None
              for command in witness]
    if (any(type(ordinal) is not int for ordinal in expected + actual)
            or expected != list(range(1, len(commands) + 1)) or actual != expected):
        return False
    return all(isinstance(command.get(side), dict)
               and isinstance(command[side].get('requests'), list)
               for command in witness for side in ('baseline', 'live'))


def classify_case(row):
    if not isinstance(row, dict) or not isinstance(row.get('case'), str):
        raise ValueError('invalid case row')
    result = row.get('result', {})
    if row.get('error') is None and 'drift' not in result:
        raise ValueError('missing case result')
    drift = result.get('drift', [])
    if not isinstance(drift, list) or any(not isinstance(item, str) for item in drift):
        raise ValueError('invalid case drift')
    if row.get('error') is not None:
        return 'failure'
    witness = row.get('witness')
    if cut(witness) or not complete_diagnostics(result, witness, bool(drift)):
        return 'failure'
    if not drift:
        return 'healthy'
    # Omitted diagnostics may hide an invariant or schema failure.
    if any(item.startswith('more drift details omitted;') for item in drift):
        return 'failure'
    # Schema and equivalent-request invariants always retain their failure status.
    invariant = any(not (': request-plan-' in item or ': HTTP status ' in item
                         or ': output status changed:' in item) for item in drift)
    if invariant:
        return 'failure'
    discoveries = {}
    peers = tuple(terminal([request for command in witness
                            for request in command[side]['requests']])
                  for side in ('baseline', 'live'))
    outcomes = []
    output_changes = {item for item in drift if ': output status changed:' in item}
    for command, comparison in zip(witness, result['commands']):
        before = terminal(command['baseline']['requests'])
        after = terminal(command['live']['requests'])
        seen = before.keys() & after.keys()
        for current in discoveries.values():
            current[:] = [change for change in current if change['request'] not in seen]
        for namespace, changes in discovery_changes(before, after).items():
            current = discoveries.setdefault(namespace, [])
            replaced = {change['request'] for change in changes}
            current[:] = [change for change in current if change['request'] not in replaced]
            current.extend(changes)
        complete = comparison.get('before_status') == comparison.get('after_status') == 'complete'
        outcomes.append(command_classification(command, discoveries, peers, complete))
        matched = {item for item in output_changes
                   if item.startswith('command ' + str(command['ordinal'])
                                      + ': output status changed:')}
        if matched:
            if outcomes[-1] != 'throttled' or not throttled_output(command):
                return 'failure'
            output_changes -= matched
    if output_changes:
        return 'failure'
    for outcome in ('failure', 'uncertain', 'throttled', 'churn'):
        if outcome in outcomes:
            return outcome
    return 'healthy'


def classify_report(report):
    eligible, checked = report.get('eligible'), report.get('checked')
    if (type(eligible) is not int or type(checked) is not int
            or eligible < 0 or checked != eligible):
        raise ValueError('checked count does not match eligible cases')
    counts = [report['drift_cases'], report['error_cases']]
    if any(type(count) is not int or count < 0 for count in counts):
        raise ValueError('invalid drift/error counts')
    rows = report.get('case_sample', [])
    if not isinstance(rows, list):
        raise ValueError('invalid case sample')
    totals = {name + '_cases': 0 for name in
              ('healthy', 'churn', 'throttled', 'uncertain', 'failure')}
    for row in rows:
        outcome = classify_case(row)
        row['classification'] = outcome
        totals[outcome + '_cases'] += 1
    # A partial report cannot clear diagnostics hidden outside its sample.
    drift_rows = sum(bool(row.get('result', {}).get('drift')) for row in rows)
    error_rows = sum(row.get('error') is not None for row in rows)
    if drift_rows != counts[0] or error_rows != counts[1]:
        raise ValueError('diagnostic counts do not match retained cases')
    report.update(totals)
    report['classification_version'] = 1
    return bool(totals['failure_cases'] or totals['uncertain_cases'])


def run_check(scratch):
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    (scratch / 'run.json').write_text(json.dumps({
        'revision': revision, 'command': COMMAND,
        'started': datetime.datetime.now(datetime.timezone.utc).isoformat()}) + '\n')
    with (scratch / 'report.json').open('wb') as report, (scratch / 'build.log').open('wb') as log:
        try:
            result = subprocess.run(COMMAND, stdout=report, stderr=log, check=False).returncode
        except OSError as error:
            log.write(str(error).encode())
            result = 127
    (scratch / 'exit-status').write_text(str(result) + '\n')
    try:
        report = json.loads((scratch / 'report.json').read_text())
        failed = classify_report(report)
        (scratch / 'report.json').write_text(json.dumps(report) + '\n')
        return result != 0 or failed
    except (ValueError, KeyError, TypeError, AttributeError) as error:
        print('Invalid live-check report: ' + str(error), file=sys.stderr)
        return True


def deliver(scratch, host, evidence_dir):
    manifest = {name: digest(scratch / name) for name in FILES}
    (scratch / 'manifest.json').write_text(json.dumps(manifest, sort_keys=True) + '\n')
    archive = scratch / 'transfer.tar'
    with tarfile.open(archive, 'w') as target:
        for name in (*FILES, 'manifest.json'):
            target.add(scratch / name, arcname=name, recursive=False)
    destination = evidence_dir / scratch.name
    command = shlex.join(['python3', '-c', RECEIVE, str(destination),
                          digest(scratch / 'manifest.json')])
    with archive.open('rb') as source:
        subprocess.run(['ssh', '-o', 'BatchMode=yes', '-o', 'ConnectTimeout=30',
                        host, command], stdin=source, check=True)
    print('Verified evidence: ' + host + ':' + str(destination))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    config_home = Path(os.environ.get('XDG_CONFIG_HOME') or Path.home() / '.config')
    settings_path = config_home / 'biomcp/nightly.json'
    try:
        settings = json.loads(settings_path.read_text()) if settings_path.exists() else {}
        if not isinstance(settings, dict) or any(not isinstance(value, str)
                                                for value in settings.values()):
            raise ValueError('expected an object of string settings')
    except (OSError, ValueError) as error:
        parser.error('invalid nightly settings at ' + str(settings_path) + ': ' + str(error))
    parser.add_argument('--host', default=os.environ.get(
        'BIOMCP_EVIDENCE_HOST', settings.get('host')))
    parser.add_argument('--evidence-dir',
                        default=os.environ.get('BIOMCP_EVIDENCE_DIR', settings.get('evidence_dir')))
    parser.add_argument('--scratch-dir',
                        default=os.environ.get('BIOMCP_SCRATCH_DIR', settings.get('scratch_dir')))
    args = parser.parse_args()
    if not args.host or not args.evidence_dir or not args.scratch_dir:
        parser.error('supply --host, --evidence-dir and --scratch-dir or their BIOMCP environment settings')
    scratch_root = Path(args.scratch_dir)
    scratch_root.mkdir(parents=True, exist_ok=True)
    scratch = Path(tempfile.mkdtemp(prefix=datetime.datetime.now(
        datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ-'), dir=scratch_root))
    print('Owned scratch: ' + str(scratch), flush=True)
    try:
        failed = run_check(scratch)
        deliver(scratch, args.host, Path(args.evidence_dir))
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print('Nightly check failed; retained scratch: ' + str(scratch) + ': ' + str(error),
              file=sys.stderr)
        return 1
    shutil.rmtree(scratch)
    return int(bool(failed))


if __name__ == '__main__':
    sys.exit(main())
