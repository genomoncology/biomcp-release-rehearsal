"""Repeat every real-time and multi-process check under CPU load; the first failure ends the run."""
import argparse
import concurrent.futures
import hashlib
import json
import math
import os
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'target/stress'
LOAD_PER_CORE = 1.5
ROUNDS = 10
CACHE_PROCESS_ROUNDS = 12
CACHE_PROCESS_AT_ONCE = 4
HANG_GUARD_SECONDS = 3600
CACHE_PROCESS = 'cache_process::concurrent_first_opens_and_distinct_writes_survive_into_a_new_process'
FETCH_TESTS = ('cache', 'retries', 'transport', 'quotas', 'grpc_worker', 'grpc_persistence', 'bundle')
PYTHON_MODULES = (
    'release.test_toolchain', 'release.test_tools', 'release.test_linux', 'release.test_mutations',
    'release.test_installer', 'release.test_checks', 'release.test_gate', 'release.test_windows',
    'release.test_artifacts.ReleaseCommands', 'release.test_workflow', 'release.test_security',
    'release.test_licenses', 'release.test_docs', 'release.test_source_revision',
    'release.test_workflow_contracts', 'release.test_publication', 'release.test_promotion',
)
PYTHON_SCRIPTS = (
    'test_article_licenses', 'test_docs', 'test_source_capture', 'test_source_capture_plan',
    'test_coverage', 'test_reply_diff', 'test_schema_release', 'test_gate_stages',
    'test_build_workers', 'test_trials', 'test_linux_fixture_cache',
)
CASE_TRIALS = (
    'adverse-event-get/late-store-lock-keeps-fetched-faers-record',
    'article-search/current-store-opens-without-write-lock',
    'article-search/late-store-lock-keeps-fetched-items',
    'cache-resilience/checked-gene-survives-write-contention',
    'cache-resilience/checked-protein-command-survives-write-contention',
    'cache-resilience/stale-replies-survive-unrelated-write-until-grace',
    'gene-sections/bundle-etag-refresh',
    'gene-sections/bundle-modified-refresh',
    'gene-sections/bundle-rate-limit-deadline',
    'gene-sections/bundle-rate-limit-short-retry',
    'gene-sections/gencc-first-publisher',
    'gene-sections/gencc-first-publisher-deadline',
    'maude-reports/late-store-lock-keeps-fetched-maude-record',
    'mcp-hosted/replay-partial-at-deadline',
    'mcp-http/framing-resource-deadlines-and-shutdown',
    'mcp-http/ignored-interrupt-keeps-serving',
    'mcp-http/local-health-and-discovery',
    'mcp-http/revisions-parity-and-concurrent-identities',
    'mcp-http/storage-refusal-and-nonlocal-bind',
    'mcp-parity/cancel-preserves-another-active-call',
    'mcp-parity/cancel-stops-work-and-frees-capacity',
    'mcp-parity/oversized-frame-drains-active-call',
)
TOOL_TRIALS = (
    'cache-resilience/hgnc-first-snapshot-recovers',
    'cache-resilience/hgnc-first-snapshot-wait-is-bounded',
    'cache-resilience/hgnc-trials-after-killed-sync',
)


def checks():
    """Name every check, cheapest first; a name selects it with --only."""
    python = sys.executable
    listed = {'first-open': None, 'cache-process': None}
    listed.update({f'fetch/{name}': ('fetch', name) for name in FETCH_TESTS})
    listed.update({f'python/{name}': [python, '-m', 'unittest', name] for name in PYTHON_MODULES})
    listed.update({f'python/{name}': [python, f'scripts/{name}.py'] for name in PYTHON_SCRIPTS})
    listed.update({f'cases/{name}': ('cases', name) for name in CASE_TRIALS})
    listed.update({f'tools/{name}': ('tools', name) for name in TOOL_TRIALS})
    return listed


def executables(build):
    if build is None:
        return {}
    return {row['target']['name']: row['executable']
            for line in Path(build).read_text().splitlines()
            if line.startswith('{') and (row := json.loads(line)).get('reason') == 'compiler-artifact'
            and row.get('executable')}


def command(name, spec, built, binary):
    if name == 'first-open':
        target = Path(binary or built['biomcp']).resolve(strict=True)
        digest = hashlib.sha256(target.read_bytes()).hexdigest()
        source = 'supplied binary' if binary else subprocess.check_output(
            ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
        return [sys.executable, 'scripts/cache_installed_first_open.py', str(target), digest,
                str(OUTPUT / 'first-open' / str(time.time_ns())), '--artifact', 'stress run',
                '--source', source]
    if name == 'cache-process':
        return [built['cache'], '--exact', CACHE_PROCESS, '--nocapture']
    if isinstance(spec, list):
        return spec
    kind, value = spec
    if kind == 'fetch':
        return [built[value], '--test-threads', '2']
    if kind == 'cases':
        return [built['cases'], value, '--exact']
    return [built['tools'], '--proofs', value, '--exact']


def sqlite_steps(log, diagnostics):
    """Return each SQLite operation and code a failed run recorded."""
    text = diagnostics.read_text(errors='replace') + log.read_text(errors='replace')
    steps = set()
    for record in re.findall(r'\{[^{}]*"operation"[^{}]*\}', text):
        row = json.loads(record)
        steps.add((row['operation'], row['primary_code'], row['extended_code']))
    return sorted(steps, key=str)


def run_once(name, argv, path):
    diagnostics = path.with_suffix('.sqlite.jsonl')
    path.parent.mkdir(parents=True, exist_ok=True)
    diagnostics.write_bytes(b'')
    env = dict(os.environ, BIOMCP_SQLITE_DIAGNOSTICS=str(diagnostics))
    started = time.monotonic()
    with path.open('wb') as output:
        try:
            status = subprocess.run(argv, cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT,
                                    timeout=HANG_GUARD_SECONDS).returncode
        except subprocess.TimeoutExpired:
            status = 'hang-guard'
    record = {'check': name, 'exit': status, 'elapsed_seconds': round(time.monotonic() - started, 3),
              'host_load': os.getloadavg(), 'log': str(path.relative_to(ROOT))}
    print(json.dumps(record), flush=True)
    if status:
        steps = sqlite_steps(path, diagnostics)
        for operation, primary, extended in steps:
            print(f'FAILED {name}: sqlite operation={operation} code={primary} extended={extended}', flush=True)
        if not steps:
            print(f'FAILED {name}: no SQLite step recorded', flush=True)
        print(''.join(path.read_text(errors='replace').splitlines(True)[-60:]), flush=True)
    return bool(status)


def run_check(name, spec, rounds, built, binary):
    """Run one check, stopping at its first failure: a failure is never retried."""
    at_once = CACHE_PROCESS_AT_ONCE if name == 'cache-process' else 1
    folder = OUTPUT / name
    with concurrent.futures.ThreadPoolExecutor(max_workers=at_once) as pool:
        for first in range(0, rounds, at_once):
            indexes = range(first, min(first + at_once, rounds))
            runs = [(index, command(name, spec, built, binary)) for index in indexes]
            if any(pool.map(lambda run: run_once(name, run[1], folder / f'{run[0]:02}.log'), runs)):
                return False
    return True


def start_load(cores):
    """Hold the job's booked cores at the set load, leaving the rest of the machine alone."""
    pinned = None
    if hasattr(os, 'sched_setaffinity'):
        pinned = sorted(os.sched_getaffinity(0))[:max(1, math.floor(cores))]
        os.sched_setaffinity(0, pinned)
    count = math.ceil(cores * LOAD_PER_CORE)
    burners = [subprocess.Popen([sys.executable, '-c', 'while True: pass']) for _ in range(count)]
    return burners, pinned


def main(arguments=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build', nargs='?', help='Cargo JSON build output naming the executables')
    parser.add_argument('--only', nargs='*', default=[], metavar='NAME',
                        help='touched checks, by whole name or prefix; --list shows the names')
    parser.add_argument('--rounds', type=int, help=f'repeats of each check; default {ROUNDS}')
    parser.add_argument('--binary', help='run first-open against this installed binary')
    parser.add_argument('--list', action='store_true', help='print check names and stop')
    args = parser.parse_args(arguments)
    listed = checks()
    if args.list:
        print('\n'.join(listed))
        return 0
    selected = {name: spec for name, spec in listed.items()
                if not args.only or any(name == item or name.startswith(item) for item in args.only)}
    unknown = [item for item in args.only if not any(name.startswith(item) for name in listed)]
    if unknown or not selected:
        parser.error('no check named ' + ', '.join(unknown or ['in this selection']))
    built = executables(args.build)
    cores = float(os.environ.get('PM_JOB_CORES', os.cpu_count() or 1))
    burners, pinned = start_load(cores)
    print(json.dumps({'checks': len(selected), 'rounds': args.rounds or ROUNDS, 'cores': cores,
                      'burners': len(burners), 'pinned': pinned, 'host_load': os.getloadavg()}), flush=True)
    finished = 0
    try:
        for name, spec in selected.items():
            # first-open repeats fifty rounds of eight processes itself.
            rounds = 1 if name == 'first-open' else args.rounds or (
                CACHE_PROCESS_ROUNDS if name == 'cache-process' else ROUNDS)
            if not run_check(name, spec, rounds, built, args.binary):
                print(f'stress: failed at {name} after {finished} passed checks', flush=True)
                return 1
            finished += 1
    finally:
        for burner in burners:
            burner.terminate()
        for burner in burners:
            burner.wait()
    print(f'stress: {finished}/{len(selected)} checks passed', flush=True)
    return 0


if __name__ == '__main__':
    sys.exit(main())
