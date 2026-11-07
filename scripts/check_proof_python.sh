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
  case-files-scans)
    baseline=${2:?case-files-scans requires the reviewed baseline commit}
    folder=target/case-file-scans
    mkdir -p "$folder/baseline"
    for name in trials test_trials; do
        git show "$baseline:scripts/$name.py" > "$folder/baseline/$name.py"
    done
    scanner=$(python3.13 - <<'SCANNER'
import json
from pathlib import Path
for package in (Path.home() / '.npm/_npx').glob('*/node_modules/jscpd/package.json'):
    if json.loads(package.read_text())['version'] == '4.0.5':
        print(package.parent.parent / '.bin/jscpd')
        break
else:
    raise SystemExit('cached jscpd 4.0.5 is required')
SCANNER
    )
    "$scanner" --version
    "$scanner" "$folder/baseline" --format python --reporters json --output "$folder/before"
    "$scanner" scripts/trials.py scripts/test_trials.py --format python --reporters json --output "$folder/after"
    uvx --offline --from vulture==2.14 vulture scripts/trials.py --min-confidence 60
    ruff check --select F401,F841,ERA,BLE,E722,S110 scripts/trials.py scripts/test_trials.py
    ;;
  *) exit 2 ;;
esac
