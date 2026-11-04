"""Run bounded native mutation shards. Prints one summary; raw logs stay in output."""
import argparse
import fnmatch
import json
import math
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time

from toolchain import manifest

CAP_SECONDS = 90 * 60
CALIBRATION_SIZE = 20
RULE_FILES = ('crates/biomcp-query/src/status.rs', 'crates/biomcp-query/src/**/next.rs',
              'crates/biomcp-adapters/src/mapped.rs', 'crates/biomcp/src/spell.rs',
              'crates/biomcp/src/grammar*.rs', 'crates/biomcp/src/grammar/*.rs',
              'crates/biomcp-fetch/src/*request*.rs')
CATEGORIES = {'CaughtMutant': 'caught', 'MissedMutant': 'survivors',
              'Unviable': 'unviable', 'Timeout': 'timeouts'}


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n')


def read(path):
    return json.loads(path.read_text())


def git(*args):
    return subprocess.check_output(['git', *args], text=True).strip()


def stop(child):
    """Native Cargo children have their own groups; retain them before parents exit."""
    rows = [tuple(map(int, line.split())) for line in subprocess.check_output(
        ['ps', '-axo', 'pid=,ppid=,pgid='], text=True, timeout=2).splitlines()]
    owned = {child.pid}
    while True:
        descendants = owned | {pid for pid, parent, _ in rows if parent in owned}
        if descendants == owned:
            break
        owned = descendants
    groups = {group for pid, _, group in rows if pid in owned} | {child.pid}
    groups.discard(os.getpgrp())
    for sig in (signal.SIGTERM, signal.SIGKILL):
        for group in groups:
            try:
                os.killpg(group, sig)
            except ProcessLookupError:
                pass
        if sig == signal.SIGTERM:
            try:
                child.wait(timeout=2)
            except subprocess.TimeoutExpired:
                pass
    child.wait()


def execute(command, deadline, log, extra_env=None):
    """Stop every owned child group on the shared cap or caller cancellation."""
    remaining = deadline - time.time()
    if remaining <= 0:
        return None
    def cancel(number, _frame):
        raise SystemExit(128 + number)
    handlers = {number: signal.signal(number, cancel) for number in (signal.SIGTERM, signal.SIGINT)}
    child = None
    try:
        with log.open('w') as stream:
            env = dict(os.environ, CARGO_BUILD_JOBS='2', RUST_TEST_THREADS='2')
            env.update(extra_env or {})
            child = subprocess.Popen(command, stdout=stream, stderr=subprocess.STDOUT,
                                     start_new_session=True, env=env)
            try:
                return child.wait(timeout=remaining)
            except subprocess.TimeoutExpired:
                stop(child)
                child = None
                return None
    except BaseException:
        if child is not None:
            stop(child)
        raise
    finally:
        for number, handler in handlers.items():
            signal.signal(number, handler)


def native_args():
    return ['cargo', 'mutants', '--no-shuffle', '--test-package', 'biomcp',
            '--cargo-arg=--locked', '--cargo-arg=--test=cases', '--cargo-arg=--package=biomcp']


def tool_version(deadline, log, extra_env=None):
    expected = 'cargo-mutants ' + manifest()['cargo-mutants']
    code = execute(['cargo', 'mutants', '--version'], deadline, log, extra_env)
    if code != 0 or log.read_text().strip() != expected:
        raise ValueError('Install the pinned cargo-mutants before planning')


def install(args):
    args.output.mkdir(parents=True, exist_ok=False)
    args.output = args.output.resolve()
    prefix = args.output / 'tool'
    env = dict(CARGO_HOME=str(args.output / 'cargo-home'),
               PATH=str(prefix / 'bin') + os.pathsep + os.environ['PATH'])
    version = manifest()['cargo-mutants']
    code = execute(['cargo', 'install', 'cargo-mutants', '--locked', '--version', version,
                    '--root', str(prefix)],
                   args.started + CAP_SECONDS, args.output / 'install.log', env)
    if code != 0:
        raise ValueError('cargo-mutants installation failed or reached the cap')
    tool_version(args.started + CAP_SECONDS, args.output / 'version.log', env)


def plan(args):
    if git('status', '--porcelain', '--untracked-files=no'):
        raise ValueError('Mutation plans require a clean tracked source tree')
    args.output.mkdir(parents=True, exist_ok=False)
    tool_version(args.started + CAP_SECONDS, args.output / 'version.log')
    candidate = git('rev-parse', 'HEAD')
    previous = git('rev-parse', '--verify', args.previous + '^{commit}')
    tracked = git('ls-files', 'crates').splitlines()
    product = [p for p in tracked if '/src/' in p and p.endswith('.rs')]
    changed = git('diff', '--name-only', previous, candidate, '--', 'crates').splitlines()
    files = [p for p in changed if p in product]
    for pattern in RULE_FILES:
        files.extend(p for p in product if fnmatch.fnmatchcase(p, pattern) and p not in files)
    if not files:
        raise ValueError('Mutation scope has no product files')
    command = native_args() + [arg for p in files for arg in ('--file', p)]
    code = execute(command + ['--list', '--json'], args.started + CAP_SECONDS,
                   args.output / 'inventory.json')
    if code != 0:
        raise ValueError('Mutation discovery failed or reached the cap')
    inventory = read(args.output / 'inventory.json')
    inventory.sort(key=lambda row: files.index(row['file']))
    names = [row['name'] for row in inventory]
    if len(names) != len(set(names)):
        raise ValueError('Native mutation inventory has duplicate names')
    remaining = max(0, args.started + CAP_SECONDS - time.time() - args.baseline_seconds)
    limit = CALIBRATION_SIZE if args.measure else math.floor(remaining / args.cost_seconds)
    if args.measure and len(names) < CALIBRATION_SIZE:
        raise ValueError('Calibration scope contains fewer than the required mutants')
    selected = names[:limit]
    if not selected or args.shards > len(selected):
        raise ValueError('Scope has too few mutants for the requested shards')
    value = dict(candidate=candidate, previous=previous, files=files, inventory=names,
                 selected=selected, shards=args.shards, started=args.started,
                 deadline=args.started + CAP_SECONDS, tool=manifest()['cargo-mutants'],
                 calibration=args.measure, cost_seconds=args.cost_seconds,
                 baseline_seconds=args.baseline_seconds)
    save(args.output / 'plan.json', value)
    print(json.dumps(dict(candidate=candidate, selected=len(selected), scope=len(names),
                          shards=args.shards, plan=str(args.output / 'plan.json'))))


def shard(args):
    value = read(args.plan)
    if (git('rev-parse', 'HEAD') != value['candidate']
            or git('status', '--porcelain', '--untracked-files=no')):
        raise ValueError('Shard source differs from the mutation plan')
    output = args.plan.parent / ('shard-' + str(args.index))
    output.mkdir(exist_ok=False)
    if time.time() < value['deadline']:
        tool_version(value['deadline'], output / 'version.log')
    if value['tool'] != manifest()['cargo-mutants']:
        raise ValueError('Shard tool differs from the mutation plan')
    if not 0 <= args.index < value['shards']:
        raise ValueError('Shard index is outside the plan')
    # Rust regex does not accept every Python re.escape escape; escape its metacharacters only.
    escaped = [re.sub(r'([\\.^$|?*+()\[\]{}])', r'\\\1', name) for name in value['selected']]
    inventory = {row['name']: row for row in read(args.plan.parent / 'inventory.json')}
    expected = {name: inventory[name] for name in value['selected']}
    files = sorted({row['file'] for row in expected.values()})
    command = native_args() + [arg for path in files for arg in ('--file', path)]
    command += ['--re', '^(' + '|'.join(escaped) + ')$']
    if time.time() < value['deadline']:
        discovery = command + ['--list', '--json']
        code = execute(discovery, value['deadline'], output / 'selection.json')
        if code != 0:
            raise ValueError('Shard discovery failed or reached the cap')
        rows = read(output / 'selection.json')
        observed = {row['name']: row for row in rows}
        exact = observed == expected and len(rows) == len(expected)
        save(output / 'selection-check.json', dict(argv=discovery, exact=exact,
             expected=len(expected), observed=len(rows)))
        if not exact:
            raise ValueError('Native shard selection differs from the exact plan')
    command += ['--shard', f"{args.index}/{value['shards']}",
                              '--jobs', '1', '--jobserver-tasks', '2',
                              '--baseline', 'run', '--timeout', str(CAP_SECONDS),
                              '--output', str(output.resolve())]
    started = time.time()
    # Native scratch trees omit Git metadata; bind the existing build contract to this plan.
    code = execute(command, value['deadline'], output / 'command.log',
                   {'BIOMCP_BUILD_REVISION': value['candidate']})
    save(output / 'command.json', dict(argv=command, exit=code, started=started, ended=time.time(),
                                     candidate=value['candidate']))
    print(json.dumps(dict(shard=args.index, exit=code, output=str(output))))
    # Survivors and native timeouts are evidence. Baseline and tool errors fail.
    return int(code not in (None, 0, 2, 3))


def collect(args):
    value = read(args.plan)
    report = dict(candidate=value['candidate'], previous=value['previous'],
                  tool=value['tool'], calibration=value['calibration'],
                  started=value['started'], deadline=value['deadline'],
                  caught=[], survivors=[], unviable=[], timeouts=[], errors=[], phases=[],
                  baseline_phases=[])
    seen = set()
    elapsed = []
    interrupted = False
    for index in range(value['shards']):
        output = args.plan.parent / ('shard-' + str(index))
        command = output / 'command.json'
        if not command.exists():
            report['errors'].append(f'Shard {index} has no completed command record')
            continue
        invocation = read(command)
        if invocation['candidate'] != value['candidate']:
            raise ValueError('Shard candidate differs from plan')
        elapsed.append(invocation['ended'])
        interrupted |= invocation['exit'] is None
        if invocation['exit'] not in (None, 0, 2, 3):
            report['errors'].append(f"Shard {index} exited {invocation['exit']}")
        outcomes = output / 'mutants.out/outcomes.json'
        if outcomes.exists():
            rows = read(outcomes)['outcomes']
            if invocation['exit'] is not None and not any(r['scenario'] == 'Baseline' for r in rows):
                report['errors'].append(f'Shard {index} has no native baseline result')
            add_outcomes(rows, value, report, seen)
        elif invocation['exit'] is not None:
            report['errors'].append(f'Shard {index} has no native outcomes')
    if not interrupted and not report['errors'] and set(value['selected']) - seen:
        report['errors'].append('Native shards omitted selected mutant outcomes')
    report['unrun'] = [name for name in value['inventory'] if name not in seen]
    report['completed'] = sum(len(report[key]) for key in ('caught', 'survivors', 'unviable'))
    report['elapsed_seconds'] = max(0, max(elapsed, default=value['started']) - value['started'])
    viable = len(report['caught']) + len(report['survivors'])
    report['wall_seconds_per_completed'] = (report['elapsed_seconds'] / report['completed']
                                             if report['completed'] else None)
    phase_cost = sum(p['duration'] for row in report['phases'] for p in row['results'])
    report['seconds_per_viable_mutant'] = phase_cost / viable if viable else None
    report['baseline_seconds'] = sum(p['duration'] for p in report['baseline_phases'])
    report['calibration_complete'] = (value['calibration'] and report['completed'] == CALIBRATION_SIZE
                                      and viable > 0 and not report['timeouts'] and not report['errors'])
    save(args.plan.parent / 'mutation-report.json', report)
    print(json.dumps({key: len(report[key]) for key in
                      ('caught', 'survivors', 'unviable', 'timeouts', 'unrun', 'errors')}))
    return int(bool(report['errors']))


def add_outcomes(outcomes, plan_value, report, seen):
    for row in outcomes:
        scenario = row['scenario']
        if scenario == 'Baseline':
            report['baseline_phases'].extend(row['phase_results'])
            if row['summary'] != 'Success':
                report['errors'].append('Native baseline did not pass')
            continue
        name = scenario['Mutant']['name']
        if name not in plan_value['selected'] or name in seen:
            raise ValueError('Unexpected or repeated native mutant: ' + name)
        category = CATEGORIES.get(row['summary'])
        if category is None:
            report['errors'].append('Unrecognized native outcome: ' + name)
            continue
        seen.add(name)
        report[category].append(name)
        report['phases'].append(dict(mutant=name, results=row['phase_results']))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    setup = commands.add_parser('install', help='Install the manifest pin in an owned native job')
    prepare = commands.add_parser('plan', help='Discover scope and save one candidate deadline')
    for command in (setup, prepare):
        command.add_argument('--started', type=float, required=True, help='Shared job start, Unix seconds')
        command.add_argument('--output', type=Path, required=True)
    prepare.add_argument('--previous', required=True, help='Previous candidate commit')
    size = prepare.add_mutually_exclusive_group(required=True)
    size.add_argument('--measure', action='store_true', help='Select the initial calibration shard')
    size.add_argument('--cost-seconds', type=float, help='Measured wall seconds per viable mutant')
    prepare.add_argument('--shards', type=int, default=1)
    prepare.add_argument('--baseline-seconds', type=float, default=0,
                         help='Measured baseline overhead reserved across all shards')
    run = commands.add_parser('shard', help='Run a planned shard; baseline always runs')
    run.add_argument('--index', type=int, required=True)
    gather = commands.add_parser('collect', help='Attach survivors and every unrun mutant to candidate')
    for command in (run, gather):
        command.add_argument('--plan', type=Path, required=True)
    args = parser.parse_args()
    if args.command == 'plan' and args.measure and args.shards != 1:
        parser.error('initial calibration requires one shard')
    if args.command == 'plan' and (args.shards < 1 or not math.isfinite(args.baseline_seconds)
                                   or args.baseline_seconds < 0
                                   or not args.measure and args.baseline_seconds <= 0
                                   or args.cost_seconds is not None
                                   and (not math.isfinite(args.cost_seconds) or args.cost_seconds <= 0)):
        parser.error('shards and measured cost must be positive and finite')
    if args.command in ('install', 'plan') and (not math.isfinite(args.started)
                                               or args.started > time.time()):
        parser.error('shared start must be finite and cannot be in the future')
    return {'install': install, 'plan': plan, 'shard': shard, 'collect': collect}[args.command](args) or 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        sys.exit(str(error))
