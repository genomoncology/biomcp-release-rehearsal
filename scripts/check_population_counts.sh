#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
case "${1:-prepare}" in
  prepare)
    make schema-generate
    make docs-generate
    mkdir -p target/population-counts-proof
    git diff --binary > target/population-counts-proof/generated.patch
    make check-cheap
    exec sh "$0" proof ;;
  proof)
    python3 scripts/check_shell.py
    ruff check --select F401,F841,ERA,BLE,E722,S110 scripts/population_counts_job.py
    python3 scripts/population_counts_job.py
    ;;
  routine)
    exec sh scripts/check_nightly_tests.sh routine ;;
  scans)
    baseline=${2:?scans requires the reviewed baseline}
    scanner=$(python3 - <<'PY'
import json
from pathlib import Path
for package in (Path.home() / '.npm/_npx').glob('*/node_modules/jscpd/package.json'):
    if json.loads(package.read_text())['version'] == '4.0.5':
        print(package.parent.parent / '.bin/jscpd')
        break
else:
    raise SystemExit('cached jscpd 4.0.5 is required')
PY
    )
    "$scanner" --version
    for side in baseline current; do
      folder="target/population-counts-scans/$side"
      mkdir -p "$folder/source"
      for file in crates/biomcp/tests/runner/schema.rs crates/biomcp/tests/runner/schema_proof.rs crates/biomcp-adapters/src/gnomad/variant.rs crates/biomcp-adapters/src/gnomad/variant/raw.rs crates/biomcp-adapters/src/gnomad/variant/map.rs crates/biomcp-query/src/variant/population.rs crates/biomcp-query/src/variant/population_release.rs crates/biomcp-types/src/response/variant_population.rs; do
        output="$folder/source/$(echo "$file" | tr / _)"
        if [ "$side" = baseline ]; then git show "$baseline:$file" > "$output"; else cp "$file" "$output"; fi
      done
      "$scanner" "$folder/source" --format rust --max-size 10mb --reporters console,json --output "$folder/report"
      mkdir -p "$folder/python"
      if [ "$side" = current ]; then cp scripts/population_counts_job.py "$folder/python/"; fi
      if [ "$side" = baseline ]; then
        git show "$baseline:scripts/trials.py" > "$folder/python/trials.py"
      else
        cp scripts/trials.py "$folder/python/trials.py"
      fi
      "$scanner" "$folder/python" --format python --max-size 10mb --reporters console,json --output "$folder/python-report"
    done
    ;;
  *) echo 'usage: check_population_counts.sh [prepare|proof|routine|scans BASELINE]' >&2; exit 2 ;;
esac
