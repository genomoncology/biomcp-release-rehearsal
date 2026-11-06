"""Run changed deadline checks once, or ten times with CPU contention."""
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import time


def retain(root):
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    archive = root / f'{revision}-{time.time_ns()}.tar.gz'
    with tarfile.open(archive, 'w:gz') as output:
        for path in root.iterdir():
            if path.suffix in ('.log', '.jsonl'):
                output.add(path, arcname=path.name)
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    destination = '/home/ian/workspace/data/biomcp-redux-evidence/0669'
    subprocess.run(['ssh', 'yellow', 'mkdir', '-p', destination], check=True)
    subprocess.run(['scp', str(archive), f'yellow:{destination}/'], check=True)
    received = subprocess.check_output(['ssh', 'yellow', 'sha256sum', f'{destination}/{archive.name}'], text=True).split()[0]
    if received != digest:
        raise ValueError('receiving evidence hash differs')
    print(json.dumps({'evidence': f'yellow:{destination}/{archive.name}', 'sha256': digest}), flush=True)


def main():
    root = Path('target/deadline-setup').resolve()
    root.mkdir(parents=True, exist_ok=True)
    with (root / 'build.jsonl').open('w') as output:
        subprocess.run(['cargo', 'build', '--release', '--locked', '-p', 'biomcp', '--bin', 'biomcp',
                        '--test', 'cases', '--test', 'tools', '--message-format=json'], stdout=output, check=True)
    artifacts = {row['target']['name']: row
                 for line in (root / 'build.jsonl').read_text().splitlines()
                 if (row := json.loads(line)).get('reason') == 'compiler-artifact' and row.get('executable')}
    binaries = {name: row['executable'] for name, row in artifacts.items()}
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    installed = root / 'installed'
    installed.mkdir(exist_ok=True)
    bindings = {}
    for name, target in [('biomcp', 'biomcp'), ('cases', 'case-runner'), ('tools', 'tool-runner')]:
        destination = installed / target
        shutil.copy2(binaries[name], destination)
        bindings[name] = {'path': str(destination), 'sha256': hashlib.sha256(destination.read_bytes()).hexdigest(),
                          'cargo_profile': artifacts[name]['profile']}
    identity = subprocess.check_output([str(installed / 'biomcp'), 'version', '--verbose'], text=True)
    if f'Commit: {source}' not in identity.splitlines():
        raise ValueError('installed proof binary differs from source commit')
    env = dict(os.environ, BIOMCP_INSTALLED_BINARY=str(installed / 'biomcp'),
               BIOMCP_CASE_RUNNER=str(installed / 'case-runner'), BIOMCP_INSTALLED_VERSION=identity.splitlines()[0].removeprefix('biomcp '),
               BIOMCP_TEST_PYTHON=sys.executable,
               BIOMCP_EVIDENCE_DIR=str(root / 'fixture-evidence'))
    commands = [
        [sys.executable, '-m', 'unittest', '-v', '-f',
         'release.test_smoke.InstalledTemporarySpace.test_installed_schema_change_names_the_case'],
        [sys.executable, '-m', 'unittest', '-v', '-f', 'release.test_checks', 'release.test_gate'],
        [sys.executable, 'scripts/typed_harness_preflight.py', 'check'],
        [binaries['cases'], 'gene-sections/gencc-first-publisher', '--test-threads', '2'],
        [binaries['tools'], '--proofs', 'mcp-http/readiness-rejects-eof-and-hung-child', '--exact'],
        [binaries['cases'], 'mcp-parity/mcp-typed-tools', '--exact'],
    ]
    loaded = sys.argv[1:] == ['load']
    if loaded:
        commands.append([sys.executable, '-m', 'unittest', '-v', '-f', 'release.test_smoke'])
    cores = float(os.environ.get('PM_JOB_CORES', os.cpu_count() or 1))
    affinity = None
    if loaded:
        # Gator reserves physical CPUs 10–11 for Windows; keep this proof on
        # its booked allocation, including all burner and test descendants.
        available = sorted(os.sched_getaffinity(0) - {10, 11})
        affinity = available[-max(1, math.floor(cores)):]
        os.sched_setaffinity(0, affinity)
        cores = len(affinity)
    count = math.ceil(cores * 1.5) if loaded else 0
    metadata = {'source_commit': source, 'profile': 'release', 'artifacts': bindings,
                'mode': 'load' if loaded else 'focused', 'allocated_cores': cores, 'burners': count,
                'host_load': os.getloadavg(), 'logical_cpus': os.cpu_count(),
                'affinity': affinity, 'python': sys.version}
    (root / 'run.jsonl').write_text(json.dumps(metadata) + '\n')
    print(json.dumps(metadata), flush=True)
    burners = [subprocess.Popen([sys.executable, '-c', 'while True: pass']) for _ in range(count)]
    failures = 0
    try:
        for repeat in range(10 if loaded else 1):
            for index, command in enumerate(commands):
                path = root / f'{repeat:02}-{index:02}.log'
                started = time.monotonic()
                with path.open('wb') as output:
                    result = subprocess.run(command, env=env, stdout=output, stderr=subprocess.STDOUT)
                record = {'repeat': repeat, 'command': command, 'exit': result.returncode,
                          'elapsed_seconds': time.monotonic() - started,
                          'host_load': os.getloadavg(), 'log': str(path)}
                with (root / 'run.jsonl').open('a') as output:
                    output.write(json.dumps(record) + '\n')
                print(json.dumps(record), flush=True)
                if result.returncode:
                    print(path.read_text(), flush=True)
                    failures += 1
                    break
            if failures:
                break
    finally:
        for child in burners:
            child.terminate()
        for child in burners:
            child.wait()
    retain(root)
    print(f'failures: {failures}', flush=True)
    return bool(failures)


if __name__ == '__main__':
    sys.exit(main())
