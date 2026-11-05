"""Run the bounded public diagnostic once and retain verified evidence before cleanup."""
import argparse
import datetime
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile

if __package__:
    from .nightly_evidence import COMMAND, FILES as FILES, RECEIVE as RECEIVE, digest as digest, deliver
else:
    from nightly_evidence import COMMAND, FILES as FILES, RECEIVE as RECEIVE, digest as digest, deliver


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


def failed_status(status):
    return status is None or type(status) is int and (status == 429 or 500 <= status < 600)


def replacement_failure(item, before, after):
    # Discovery may change a failed dependent's identity. Check the same full
    # input selection rule with transport status removed from that comparison.
    if not failed_status(item.get('status')):
        return False
    healthy = {key: dict(value, status=200) if value is item else value
               for key, value in after.items()}
    changes = discovery_changes(before, healthy)
    return any(old.get('status') == 200 and dependent_change(
        [old], [healthy[item['identity']]], changes, before, healthy,
        (before, healthy), complete=True) == 'churn'
        for key, old in before.items() if key not in after)


def failure_plan(before, after, discoveries, peers):
    roots = {key: old for key, old in before.items() if key in after
             and failed_status(after[key].get('status'))}
    removed = [value for key, value in before.items() if key not in after]
    added = [value for key, value in after.items() if key not in before]
    def downstream(request):
        provenance = request.get('provenance') or {}
        inputs = provenance.get('inputs') or {}
        ids = inputs.get('ids')
        return (provenance.get('input_context') and valid_ids(ids) and bool(ids)
                and any(set(ids) <= set(discovery_sequences(root).get(
                    inputs.get('namespace'), [])) for root in roots.values()))
    removed = [request for request in removed if not downstream(request)]
    # A successful provider may also have changed the full selection while a
    # sibling request failed. Preserve the ordinary discovery/context checks.
    healthy = {key: dict(value, status=200) if failed_status(value.get('status')) else value
               for key, value in after.items()}
    added = [healthy[request['identity']] for request in added]
    # Old diagnostics can retain dependent inputs without the failed root's
    # discovery selection. That is missing proof, not contradictory proof.
    if (removed and not added and roots
            and all(not discovery_sequences(root) for root in roots.values())
            and all((request.get('provenance') or {}).get('input_context')
                    and valid_ids(((request.get('provenance') or {}).get('inputs') or {}).get('ids'))
                    and ((request.get('provenance') or {}).get('inputs') or {}).get('namespace')
                    not in discoveries for request in removed)):
        return 'uncertain'
    return dependent_change(removed, added, discoveries, before, healthy,
                            (peers[0], {key: dict(value, status=200)
                             if key in healthy and failed_status(value.get('status')) else value
                             for key, value in peers[1].items()}), complete=True) if removed or added else 'healthy'


def command_causes(command):
    if not isinstance(command, dict) or not all(
            isinstance(command.get(side), dict) and isinstance(command[side].get('requests'), list)
            for side in ('baseline', 'live')):
        return {cause: set() for cause in ('throttled', 'outage', 'recovered')}
    before = terminal(command['baseline']['requests'])
    after = terminal(command['live']['requests'])
    throttled = {item['source'] for item in after.values() if item.get('status') == 429}
    outage = {item['source'] for key, item in after.items()
              if failed_status(item.get('status')) and item.get('status') != 429
              and (key in before and before[key].get('status') == 200
                   or replacement_failure(item, before, after))}
    recovered = {item['source'] for key, item in after.items()
                 if key in before and before[key].get('status') != 200
                 and item.get('status') == 200}
    recovered.update(source['source']
                     for source in command['baseline'].get('output', {}).get('sources', [])
                     if source.get('state') in ('failed', 'stale')
                     and command['live'].get('output', {}).get('status') == 'complete')
    return {'throttled': throttled, 'outage': outage, 'recovered': recovered}


def cache_order_failure(command, earlier_failed):
    if command['baseline']['requests']:
        return False
    added = list(terminal(command['live']['requests']).values())
    if not added or any(request.get('status') != 200 for request in added):
        return False
    roots = [request for request in added
             if not (request.get('provenance') or {}).get('inputs')]
    if len(roots) != 1 or roots[0]['source'] not in earlier_failed:
        return False
    discoveries = discovery_sequences(roots[0])
    for request in added:
        if request is roots[0]:
            continue
        provenance = request.get('provenance') or {}
        inputs = provenance.get('inputs')
        if (not isinstance(inputs, dict) or not provenance.get('input_context')
                or not valid_ids(inputs.get('ids')) or not inputs['ids']
                or not set(inputs['ids']) <= set(discoveries.get(inputs.get('namespace'), []))):
            return False
    return True


def command_classification(capture, discoveries, peers, complete=False, comparison=None):
    before = terminal(capture['baseline']['requests'])
    after = terminal(capture['live']['requests'])
    causes = command_causes(capture)
    for item in after.values():
        status = item.get('status')
        if not (type(status) is int and (200 <= status < 300 or status == 429)):
            if not (failed_status(status) and (
                    item['identity'] in before and before[item['identity']].get('status') == 200
                    or replacement_failure(item, before, after))):
                return 'failure'
    # A provider failure ends discovery and its downstream work in this command.
    # Other commands still have to explain their own request plans.
    for cause in ('outage', 'throttled'):
        if causes[cause]:
            if not explained_source_failures(capture):
                return 'failure'
            plan = failure_plan(before, after, discoveries, peers)
            return cause if plan in ('healthy', 'churn') else plan
    comparison = comparison or {}
    removed = [before[key] for key in before.keys() - after.keys()]
    added = [after[key] for key in after.keys() - before.keys()]
    plan = (dependent_change(removed, added, discoveries, before, after, peers, complete)
            if removed or added else 'healthy')
    if (plan in ('healthy', 'churn')
            and comparison.get('before_status') in ('partial', 'failed')
            and comparison.get('after_status') == 'complete'):
        return 'recovered'
    return plan


def explained_source_failures(command):
    causes = command_causes(command)
    affected = causes['throttled'] | causes['outage']
    failed = [source for source in command['live'].get('output', {}).get('sources', [])
              if source.get('state') in ('failed', 'stale')]
    return all(source.get('source') in affected
               and source.get('reason') in ('rate-limited', 'unavailable', 'deadline-exceeded')
               for source in failed)


def explained_output(command, comparison, outcome):
    before = command['baseline'].get('output', {})
    after = command['live'].get('output', {})
    old = comparison.get('before_status', before.get('status'))
    new = comparison.get('after_status', after.get('status'))
    if outcome == 'recovered':
        return old in ('partial', 'failed') and new == 'complete'
    if outcome not in ('throttled', 'outage'):
        return False
    return (old in ('complete', 'partial', 'failed') and new in ('partial', 'failed')
            and explained_source_failures(command))


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
                         or ': output status changed:' in item or item == 'page continuation changed') for item in drift)
    if invariant:
        return 'failure'
    discoveries = {}
    peers = tuple(terminal([request for command in witness
                            for request in command[side]['requests']])
                  for side in ('baseline', 'live'))
    outcomes, effects = [], []
    earlier_failed = {}
    earlier_roots = {}
    previous_command, previous_outcome, previous_comparison = None, None, {}
    output_changes = {item for item in drift if ': output status changed:' in item}
    for command, comparison in zip(witness, result['commands']):
        # An earlier failed root was never cached, so later commands may
        # repeat it while their baseline consumed the successful cache entry.
        original = command
        command = dict(command, baseline=dict(command['baseline'], requests=[
            *command['baseline']['requests'], *[root for key, root in earlier_roots.items()
                if key not in terminal(command['baseline']['requests'])
                and key in terminal(command['live']['requests'])
                and failed_status(terminal(command['live']['requests'])[key].get('status'))]]))
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
        outcome = command_classification(command, discoveries, peers, complete, comparison)
        if outcome == 'uncertain' and cache_order_failure(command, earlier_failed):
            root = next(request for request in command['live']['requests']
                        if not (request.get('provenance') or {}).get('inputs'))
            outcome = earlier_failed[root['source']]
        if (outcome == 'healthy' and previous_outcome in ('outage', 'throttled')
                and not before and not after and previous_command is not None
                and all('output' in original[side] and 'output' in previous_command[side]
                        and original[side]['output'] == previous_command[side]['output']
                        for side in ('baseline', 'live'))
                and comparison.get('before_status') != comparison.get('after_status')
                and all(comparison.get(key) == previous_comparison.get(key)
                        for key in ('before_status', 'after_status'))):
            outcome = previous_outcome
        outcomes.append(outcome)
        causes = command_causes(command)
        effect = next((cause for cause in ('outage', 'throttled')
                       if causes[cause]), outcome) if outcome == 'uncertain' else outcome
        effects.append(effect)
        for cause in ('throttled', 'outage'):
            earlier_failed.update({source: cause for source in causes[cause]})
        for key, request in after.items():
            if request.get('status') == 200:
                earlier_roots.pop(key, None)
        earlier_roots.update({key: before[key] for key in before.keys() & after.keys()
                              if failed_status(after[key].get('status'))})
        previous_command, previous_outcome, previous_comparison = original, outcome, comparison
        matched = {item for item in output_changes
                   if item.startswith('command ' + str(command['ordinal'])
                                      + ': output status changed:')}
        if matched:
            if not (explained_output(command, comparison, effect)
                    or not before and not after and outcome in ('outage', 'throttled')):
                return 'failure'
            output_changes -= matched
    if 'page continuation changed' in drift and not any(
            outcome in ('throttled', 'outage', 'recovered', 'churn') for outcome in effects):
        return 'failure'
    if output_changes:
        return 'failure'
    for outcome in ('failure', 'uncertain', 'outage', 'throttled', 'recovered', 'churn'):
        if outcome in outcomes:
            return outcome
    return 'healthy'


def classify_report(report, previous=(), night=None):
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
              ('healthy', 'churn', 'throttled', 'outage', 'recovered', 'uncertain', 'failure')}
    classes = {name: {'cases': [], 'sources': []} for name in
               ('throttled', 'outage', 'recovered')}
    for row in rows:
        outcome = classify_case(row)
        row['classification'] = outcome
        totals[outcome + '_cases'] += 1
        if outcome == 'recovered':
            classes['recovered']['cases'].append(row['case'])
        # Keep all observed causes even when another command makes the case fail.
        for command in row.get('witness', []) if isinstance(row.get('witness'), list) else []:
            for cause, sources in command_causes(command).items():
                if sources and (cause != 'recovered' or outcome == 'recovered'):
                    classes[cause]['cases'].append(row['case'])
                    classes[cause]['sources'].extend(sources)
    # A partial report cannot clear diagnostics hidden outside its sample.
    drift_rows = sum(bool(row.get('result', {}).get('drift')) for row in rows)
    error_rows = sum(row.get('error') is not None for row in rows)
    if drift_rows != counts[0] or error_rows != counts[1]:
        raise ValueError('diagnostic counts do not match retained cases')
    report.update(totals)
    report['classification_version'] = 2
    report['cause_classes'] = {cause: {key: sorted(set(values)) for key, values in data.items()}
                              for cause, data in classes.items()}
    persistent = set(classes['outage']['sources'])
    for offset in (1, 2):
        date = (night - datetime.timedelta(days=offset)).isoformat() if night else None
        prior = next((item for day, item in previous if day == date), None)
        persistent &= set(prior.get('cause_classes', {}).get('outage', {}).get('sources', [])) if prior else set()
    report['persistent_outage_sources'] = sorted(persistent)
    return bool(totals['failure_cases'] or totals['uncertain_cases'] or persistent)


def run_check(scratch, previous=()):
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
        failed = classify_report(report, previous, datetime.datetime.now(datetime.timezone.utc).date())
        (scratch / 'report.json').write_text(json.dumps(report) + '\n')
        return result != 0 or failed
    except (ValueError, KeyError, TypeError, AttributeError) as error:
        print('Invalid live-check report: ' + str(error), file=sys.stderr)
        return True


def read_previous(host, evidence_dir, night):
    """Read only the preceding two calendar nights; never rewrite raw evidence."""
    dates = [(night - datetime.timedelta(days=offset)).strftime('%Y%m%d') for offset in (2, 1)]
    reader = """# nightly-history
import json, pathlib, sys
root = pathlib.Path(sys.argv[1])
rows = []
for date in sys.argv[2:]:
    paths = sorted(root.glob(date + '*/report.json'))
    if paths:
        rows.append([date, paths[-1].read_text()])
print(json.dumps(rows))
"""
    command = shlex.join(['python3', '-c', reader, str(evidence_dir), *dates])
    output = subprocess.check_output(['ssh', '-o', 'BatchMode=yes', '-o', 'ConnectTimeout=30',
                                      host, command], text=True)
    previous = []
    for date, raw in json.loads(output):
        report = json.loads(raw)
        day = datetime.datetime.strptime(date, '%Y%m%d').date()
        classify_report(report)
        previous.append((day.isoformat(), report))
    return previous


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
        previous = read_previous(args.host, args.evidence_dir,
                                 datetime.datetime.now(datetime.timezone.utc).date())
        failed = run_check(scratch, previous)
        deliver(scratch, args.host, Path(args.evidence_dir))
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print('Nightly check failed; retained scratch: ' + str(scratch) + ': ' + str(error),
              file=sys.stderr)
        return 1
    shutil.rmtree(scratch)
    return int(bool(failed))


if __name__ == '__main__':
    sys.exit(main())
