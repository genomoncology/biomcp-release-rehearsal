"""Run the installed smoke section using retained release bytes, without building."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import threading
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from release.artifacts import inspect


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def identities(pid):
    """Read only descendants' identity and scheduling metadata, never arguments."""
    found = []
    pending = [pid]
    while pending:
        current = pending.pop()
        root = Path('/proc') / str(current)
        try:
            for task in (root / 'task').iterdir():
                pending.extend(int(value) for value in (task / 'children').read_text().split())
            status = dict(line.split(':', 1) for line in (root / 'status').read_text().splitlines())
            found.append({'pid': current, 'ppid': int(status['PPid']),
                          'executable': (root / 'exe').readlink().name,
                          'state': status['State'].strip()})
        except (FileNotFoundError, ProcessLookupError):
            continue
    return found


def bind_fixtures(candidate, proof, output):
    helpers = [str(Path('scripts') / path.relative_to(proof / 'fixtures/support/scripts'))
               for path in (proof / 'fixtures/support/scripts').rglob('*') if path.is_file()]
    fixture_tree = subprocess.check_output(['git', 'ls-tree', '-rz', candidate, '--', 'tests',
                                            'sdlc/planning/slices/gene.md', *helpers])
    fixture_manifest = {}
    for entry in fixture_tree.split(b'\0'):
        if not entry:
            continue
        metadata, name = entry.split(b'\t', 1)
        path = Path(os.fsdecode(name))
        data = os.fsencode(os.readlink(path)) if path.is_symlink() else path.read_bytes()
        blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
        expected = metadata.decode().split()[2]
        if blob != expected:
            raise ValueError('runtime fixture differs from candidate: ' + str(path))
        fixture_manifest[str(path)] = {'git_blob': expected, 'sha256': hashlib.sha256(data).hexdigest()}
    actual_paths = {str(path) for path in Path('tests').rglob('*') if path.is_file() or path.is_symlink()}
    actual_paths.update(['sdlc/planning/slices/gene.md', *helpers])
    if actual_paths != set(fixture_manifest):
        raise ValueError('runtime fixture file selection differs from candidate')
    (output / 'fixture-source.json').write_text(json.dumps(fixture_manifest, sort_keys=True, indent=2) + '\n')
    retained = {str(path.relative_to(proof / 'fixtures')): digest(path)
                for path in (proof / 'fixtures').rglob('*') if path.is_file()}
    (output / 'retained-fixtures.json').write_text(json.dumps(retained, sort_keys=True, indent=2) + '\n')


def observe(child, stopped, errors, path):
    try:
        with path.open('a') as log:
            while not stopped.is_set():
                log.write(json.dumps({'monotonic_ns': time.monotonic_ns(), 'processes': identities(child.pid)}) + '\n')
                log.flush()
                stopped.wait(20)
    except OSError as error:
        errors.append(type(error).__name__)
        try:
            child.terminate()
        except ProcessLookupError:
            pass


def cleanup(pid):
    try:
        os.killpg(pid, signal.SIGTERM)
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            os.killpg(pid, 0)
            time.sleep(0.1)
        os.killpg(pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


def main():
    if sys.argv[1:] == ['check']:
        subprocess.run(['uvx', '--offline', '--from', 'ruff==0.17.2', 'ruff', 'check', '--select',
                        'F401,F841,ERA,BLE,E722,S110', 'release/smoke.py', 'release/test_smoke.py',
                        'scripts/installed_smoke_section.py', 'scripts/test_installed_smoke_section.py'], check=True)
        subprocess.run([sys.executable, '-m', 'unittest', '-v', 'scripts.test_installed_smoke_section'], check=True)
        subprocess.run(['uvx', '--offline', '--from', 'vulture==2.14', 'vulture', 'release/gate.py',
                        'release/smoke.py', 'scripts/installed_smoke_section.py',
                        'scripts/test_installed_smoke_section.py', '--min-confidence', '60'], check=True)
        scanner = next((path.parent.parent / '.bin/jscpd' for path in Path.home().glob('.npm/_npx/*/node_modules/jscpd/package.json')
                        if json.loads(path.read_text()).get('version') == '4.0.5'), None)
        if scanner is None:
            raise ValueError('cached jscpd 4.0.5 unavailable')
        root = Path('target/installed-smoke-tooling')
        baseline = root / 'baseline'
        baseline.mkdir(parents=True)
        old = subprocess.check_output(['git', 'archive', '1bb97e64f41634b19efc42b937b195e55a20ece4',
                                       'release/smoke.py', 'release/test_smoke.py'])
        subprocess.run(['tar', '-xf', '-', '-C', str(baseline)], input=old, check=True)
        for kind, files in [('product', ['release/smoke.py', 'scripts/installed_smoke_section.py']),
                            ('tests', ['release/test_smoke.py', 'scripts/test_installed_smoke_section.py'])]:
            for side in ['baseline', 'current']:
                selected = [str(baseline / files[0])] if side == 'baseline' else files
                subprocess.run([str(scanner), *selected, '--format', 'python', '--max-size', '10mb',
                                '--reporters', 'json', '--output', str(root / (side + '-' + kind))], check=True)
            before = json.loads((root / ('baseline-' + kind) / 'jscpd-report.json').read_text())
            after = json.loads((root / ('current-' + kind) / 'jscpd-report.json').read_text())
            if after['statistics']['total']['duplicatedTokens'] > before['statistics']['total']['duplicatedTokens']:
                raise ValueError('new Python duplicate tokens: ' + kind)
        return 0
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('artifacts', type=Path)
    parser.add_argument('proof', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--candidate', required=True)
    args = parser.parse_args()
    artifacts, proof = args.artifacts.resolve(), args.proof.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    wheel, = artifacts.glob('*.whl')
    record = inspect(wheel)
    bootstrap = json.loads((proof / 'bootstrap.json').read_text())
    if record['revision'] != args.candidate or bootstrap['product_revision'] != args.candidate or bootstrap['tooling_revision'] != args.candidate:
        raise ValueError('artifact and runner source must match the selected candidate')
    installed = output / 'installed'
    subprocess.run([sys.executable, '-m', 'venv', str(installed)], check=True)
    subprocess.run([str(installed / 'bin/python'), '-m', 'pip', 'install', '--no-index', '--no-deps', str(wheel)], check=True)
    binary = installed / 'bin/biomcp'
    if digest(binary) != record['binary_sha256']:
        raise ValueError('installed binary differs from release manifest')
    bind_fixtures(args.candidate, proof, output)
    for name in ['case-runner', 'tool-runner']:
        listing = subprocess.check_output([str(proof / name), *(['--proofs'] if name == 'tool-runner' else []), '--list'], env=dict(os.environ, BIOMCP_TEST_FIXTURES=str(proof / 'fixtures')), text=True)
        (output / (name + '-list.txt')).write_text(listing)
    binding = {'candidate': args.candidate, 'product': record, 'bootstrap': bootstrap,
               'fixture_source_sha256': digest(output / 'fixture-source.json'),
               'wheel_sha256': digest(wheel), 'binary_sha256': digest(binary),
               'runners': {name: digest(proof / name) for name in ['case-runner', 'tool-runner']},
               'tooling_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
               'python': sys.version, 'retained_fixture_sha256': digest(output / 'retained-fixtures.json'),
               'toolchain': json.loads(Path('release/toolchain.json').read_text()),
               'source_files': {str(path): digest(path) for path in [Path(__file__), Path('release/smoke.py'), Path('release/test_smoke.py'), Path('release/gate.py')]}}
    (output / 'binding.json').write_text(json.dumps(binding, indent=2) + '\n')
    env = dict(os.environ, BIOMCP_INSTALLED_BINARY=str(binary), BIOMCP_CASE_RUNNER=str(proof / 'case-runner'),
               BIOMCP_TEST_FIXTURES=str(proof / 'fixtures'),
               BIOMCP_INSTALLED_VERSION=record['version'], BIOMCP_EVIDENCE_DIR=str(output / 'fixture-evidence'),
               BIOMCP_SMOKE_PHASE_FILE=str(output / 'phases.jsonl'))
    command = [sys.executable, '-m', 'unittest', '-v',
               'release.test_smoke.InstalledTemporarySpace.test_isolated_installed_commands_and_cleanup']
    stopped = threading.Event()
    observation_errors = []
    with (output / 'stdout.log').open('wb') as stdout, (output / 'stderr.log').open('wb') as stderr:
        child = subprocess.Popen(command, env=env, stdout=stdout, stderr=stderr, start_new_session=True)
        observer = threading.Thread(target=observe, args=(child, stopped, observation_errors, output / 'processes.jsonl'))
        observer.start()
        try:
            status = child.wait()
        finally:
            stopped.set()
            observer.join()
            cleanup(child.pid)
    status = status or int(bool(observation_errors))
    (output / 'result.json').write_text(json.dumps({'exit': status, 'command': command, 'observation_errors': observation_errors}) + '\n')
    print(json.dumps({'exit': status, 'output': str(output)}), flush=True)
    return status


if __name__ == '__main__':
    sys.exit(main())
