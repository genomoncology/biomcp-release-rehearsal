#!/bin/sh
# Build once and run installed grammar cases selected from the runner's discovery.
set -eu
cd "$(dirname "$0")/.."
case "${1:-focused}" in
  docs)
    exec python3 -c 'import json, os; command=json.load(open("sdlc/pm.json"))["ticketsGate"]; assert command.count("make landing-gate") == 1; os.execlp("sh", "sh", "-c", command.replace("make landing-gate", "make docs-check"))' ;;
  scans)
    mkdir -p target/usage-installed/scans/baseline
    git archive 00b80ec46 crates/biomcp/src/grammar.rs crates/biomcp/src/grammar tests/cases/grammar | tar -xf - -C target/usage-installed/scans/baseline
    python3 - <<'SCAN'
import json
from pathlib import Path
import subprocess
scanner = next(path.parent.parent / '.bin/jscpd'
               for path in (Path.home() / '.npm/_npx').glob('*/node_modules/jscpd/package.json')
               if json.loads(path.read_text())['version'] == '4.0.5')
subprocess.run([str(scanner), '--version'], check=True)
for label, root in [('baseline', Path('target/usage-installed/scans/baseline')), ('current', Path('.'))]:
    for kind, paths, formats in [('product', ['crates/biomcp/src/grammar.rs', 'crates/biomcp/src/grammar'], 'rust'), ('tests', ['tests/cases/grammar'], 'yaml')]:
        subprocess.run([str(scanner), *[str(root / path) for path in paths], '--format', formats,
                        '--max-size', '10mb', '--reporters', 'json', '--output',
                        str(Path('target/usage-installed/scans') / (label + '-' + kind))], check=True)
SCAN
    exit ;;
  cases)
    runner=${2:?cases requires the prepared runner}
    binary=${3:?cases requires the installed binary}
    python3 scripts/trials.py --runner "$runner" --executable "$binary" --installed \
      query-validation/grammar/boolean-literals.yaml \
      query-validation/grammar/semantic-arguments.yaml \
      query-validation/grammar/source-filter-errors.yaml \
      query-validation/grammar/usage-bad-enum.yaml \
      query-validation/grammar/usage-bad-typed-id.yaml \
      query-validation/grammar/usage-missing-option.yaml \
      query-validation/grammar/usage-unknown-option.yaml
    exec python3 scripts/test_usage_requests.py "$binary" target/usage-installed ;;
  focused) ;;
  *) echo 'usage: check_usage_tests.sh [focused|scans]' >&2; exit 2 ;;
esac
make check-cheap
python3 - <<'PY'
from pathlib import Path
import shutil
import subprocess
import json
import math
import os
workers = str(max(1, math.floor(float(os.environ.get('PM_JOB_CORES', '2')))))
os.environ.setdefault('CARGO_BUILD_JOBS', workers)
os.environ.setdefault('RUST_TEST_THREADS', workers)

def built_executable(target):
    command = ['cargo', 'build' if target == 'biomcp' else 'test', '--locked', '-p', 'biomcp', '--message-format=json']
    command += ['--bin', target] if target == 'biomcp' else ['--test', target, '--no-run']
    result = subprocess.run(command, stdout=subprocess.PIPE, text=True, check=True)
    artifacts = [json.loads(line) for line in result.stdout.splitlines()]
    return next(row['executable'] for row in reversed(artifacts)
                if row.get('reason') == 'compiler-artifact' and row.get('executable') and row['target']['name'] == target)

binary = built_executable('biomcp')
runner = built_executable('cases')
stage = Path('target/usage-installed').resolve()
stage.mkdir(parents=True, exist_ok=True)
shutil.copy2(binary, stage / 'biomcp')
listed = subprocess.check_output([runner, '--list'], text=True, cwd='crates/biomcp')
(stage / 'cases-list.txt').write_text(listed)
print(listed, end='')
names = [line.removesuffix(': test') for line in listed.splitlines()
         if line.startswith('query-validation/grammar/') and line.endswith(': test')]
if not names:
    raise SystemExit('No grammar cases discovered')
subprocess.run(['python3', 'scripts/trials.py', '--runner', runner,
                '--executable', str(stage / 'biomcp'), '--installed', *names], check=True)
subprocess.run(['python3', 'scripts/test_usage_requests.py', str(stage / 'biomcp'), str(stage)], check=True)
subprocess.run(['make', 'docs-generate'], check=True)
PY
