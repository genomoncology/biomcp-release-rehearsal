#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
export PATH="$HOME/.local/bin:$PATH"
python3.13 --version
case "${1:-fixtures}" in
  fixtures)
    ruff check --select F401,F841,ERA,BLE,E722,S110 release/gate.py release/smoke.py release/test_gate.py scripts/trials.py
    python3.13 scripts/check_shell.py
    python3.13 -m unittest release.test_gate release.test_checks
    python3.13 scripts/test_trials.py
    ;;
  proof)
    export BIOMCP_TEST_PYTHON=$(python3.13 -c 'import sys; print(sys.executable)')
    cargo test --locked -p biomcp --test tools -- --proofs --exact live-source-checks/retained-drift-witnesses
    cargo clippy --locked -p biomcp --test tools -- -D warnings
    ;;
  diagnostics)
    python3.13 - <<'PYTHON'
import os
from pathlib import Path
import subprocess
import sys
import tempfile

sys.path.insert(0, str(Path('release').resolve()))
from artifacts import cargo

runner = cargo('dev', tests='tools')['executable']
with tempfile.TemporaryDirectory() as folder:
    unavailable = str(Path(folder) / 'unavailable-proof-python.exe')
    for interpreter, message in [(None, 'iCite classifier proof requires BIOMCP_TEST_PYTHON from its caller'),
                                 (unavailable, 'cannot run iCite proof Python ' + unavailable)]:
        environment = dict(os.environ)
        environment.pop('BIOMCP_TEST_PYTHON', None)
        if interpreter is not None:
            environment['BIOMCP_TEST_PYTHON'] = interpreter
        result = subprocess.run([runner, '--proofs', '--exact', 'live-source-checks/retained-drift-witnesses'],
                                env=environment, capture_output=True, text=True)
        print(result.stdout, end='')
        print(result.stderr, end='', file=sys.stderr)
        if result.returncode == 0 or message not in result.stdout + result.stderr:
            raise SystemExit('proof did not refuse its unavailable interpreter clearly')
PYTHON
    ;;
  *) exit 2 ;;
esac
