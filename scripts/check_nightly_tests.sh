#!/bin/sh
# Check the offline nightly wrapper and its source-owned discovery fixtures.
set -eu
cd "$(dirname "$0")/.."
case "${1:-focused}" in
  routine)
    exec python3 -c 'import json, os; os.execlp("sh", "sh", "-c", json.load(open("sdlc/pm.json"))["ticketsGate"])' ;;
  scans)
    baseline=${2:?scans requires the reviewed baseline commit}
    mkdir -p target/nightly-scans/baseline
    git show "$baseline":scripts/nightly_live_check.py > target/nightly-scans/baseline/nightly_live_check.py
    git show "$baseline":scripts/test_nightly_live_check.py > target/nightly-scans/baseline/test_nightly_live_check.py
    scanner=$(python3 - <<'SCANNER'
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
    for side in baseline current; do
      prefix=scripts
      test "$side" != baseline || prefix=target/nightly-scans/baseline
      for name in nightly_live_check test_nightly_live_check; do
        "$scanner" "$prefix/$name.py" --format python --max-size 10mb \
          --reporters json --output "target/nightly-scans/$side-$name"
      done
    done
    uvx --from vulture==2.14 vulture scripts/nightly_live_check.py --min-confidence 60
    ruff --version
    ruff check --select F401,F841,ERA,BLE,E722,S110 scripts/nightly_live_check.py scripts/test_nightly_live_check.py
    exit ;;
  focused) ;;
  *) echo 'usage: check_nightly_tests.sh [focused|scans BASE_COMMIT|routine]' >&2; exit 2 ;;
esac
cargo fmt --all -- --check
ruff check --select F401,F841,ERA,BLE,E722,S110 scripts/nightly_live_check.py scripts/reclassify_nightly.py scripts/nightly_recommendation_cases.py scripts/test_nightly_live_check.py
python3 scripts/check_docs.py
python3 scripts/test_nightly_live_check.py
python3 -m unittest scripts.test_nightly_live_check.NightlyCommand.test_complete_recommendation_replacements_follow_exact_discovery
