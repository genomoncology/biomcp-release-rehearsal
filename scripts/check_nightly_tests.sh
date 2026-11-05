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
    for name in nightly_live_check nightly_evidence test_nightly_live_check nightly_recommendation_cases nightly_test_support test_nightly_classification; do
      if git cat-file -e "$baseline:scripts/$name.py" 2>/dev/null; then
        git show "$baseline:scripts/$name.py" > "target/nightly-scans/baseline/$name.py"
      fi
    done
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
      for name in nightly_live_check nightly_evidence test_nightly_live_check nightly_recommendation_cases nightly_test_support test_nightly_classification; do
        if test "$side" = baseline && ! git cat-file -e "$baseline:scripts/$name.py" 2>/dev/null; then
          continue
        fi
        "$scanner" "$prefix/$name.py" --format python --max-size 10mb \
          --reporters json --output "target/nightly-scans/$side-$name"
      done
    done
    uvx --from vulture==2.14 vulture scripts/nightly_live_check.py scripts/nightly_evidence.py --min-confidence 60
    ruff --version
    ruff check --select F401,F841,ERA,BLE,E722,S110 scripts/nightly_live_check.py scripts/nightly_evidence.py scripts/test_nightly_live_check.py scripts/test_nightly_classification.py scripts/nightly_test_support.py scripts/nightly_recommendation_cases.py release/gate.py
    exit ;;
  focused) ;;
  *) echo 'usage: check_nightly_tests.sh [focused|scans BASE_COMMIT|routine]' >&2; exit 2 ;;
esac
cargo fmt --all -- --check
ruff check --select F401,F841,ERA,BLE,E722,S110 scripts/nightly_live_check.py scripts/nightly_evidence.py scripts/reclassify_nightly.py scripts/nightly_recommendation_cases.py scripts/test_nightly_live_check.py scripts/test_nightly_classification.py scripts/nightly_test_support.py
python3 scripts/check_docs.py
python3 scripts/test_nightly_live_check.py
