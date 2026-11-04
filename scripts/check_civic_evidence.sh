#!/bin/sh
# Exercise evidence filters against native request recordings through the installed CLI.
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
case "${1:-focused}" in
  focused)
    output="${CARGO_TARGET_DIR:-target}/civic-evidence"
    mkdir -p "$output"
    make -s --no-print-directory civic-evidence-build > "$output/build.jsonl"
    runner=$(python3 -c 'import json,sys; rows=[json.loads(line) for line in open(sys.argv[1]) if line.startswith("{")]; print(next(row["executable"] for row in rows if row.get("reason")=="compiler-artifact" and row["target"]["name"]=="cases" and row.get("executable")))' "$output/build.jsonl")
    python3 scripts/trials.py --runner "$runner" --executable "${CARGO_TARGET_DIR:-target}/debug/biomcp" --list > "$output/list.txt"
    rg 'civic-evidence' "$output/list.txt"
    python3 scripts/trials.py --runner "$runner" --executable "${CARGO_TARGET_DIR:-target}/debug/biomcp" --installed \
      drug-civic/drug/civic-evidence-public.yaml drug-civic/drug/civic-evidence-streams.yaml variant-civic/variant/civic-evidence-filters.yaml variant-civic/variant/native-civic.yaml
    baseline="$output/baseline/biomcp"
    mkdir -p "$(dirname "$baseline")"
    if [ ! -f "$baseline" ]; then
      cp "${CIVIC_BASELINE_EXECUTABLE:-/Users/ian/workspace/jobs/biomcp-redux-0599/target/drug-names-proof/baseline-bin/biomcp}" "$baseline"
    fi
    python3 -c 'import hashlib,sys; from pathlib import Path; assert hashlib.sha256(Path(sys.argv[1]).read_bytes()).hexdigest() == "b50998e586ccc38a2a4f6d851cce229651e380560814e5d4212d4694f89c7660"' "$baseline"
    python3 scripts/check_civic_evidence_compatibility.py "$baseline" "${CARGO_TARGET_DIR:-target}/debug/biomcp"
    ;;
  combined)
    python_bin=$(python3.13 -c 'from pathlib import Path; import sys; print(Path(sys.executable).resolve().parent)')
    export PATH="$python_bin:$PATH"
    make schema-generate docs-generate
    output="${CARGO_TARGET_DIR:-target}/civic-evidence"
    mkdir -p "$output"
    git diff --binary > "$output/combined-generated.patch"
    make -s --no-print-directory civic-evidence-build > "$output/build.jsonl"
    runner=$(python3 -c 'import json,sys; rows=[json.loads(line) for line in open(sys.argv[1]) if line.startswith("{")]; print(next(row["executable"] for row in rows if row.get("reason")=="compiler-artifact" and row["target"]["name"]=="cases" and row.get("executable")))' "$output/build.jsonl")
    python3 scripts/trials.py --runner "$runner" --executable "${CARGO_TARGET_DIR:-target}/debug/biomcp" --list > "$output/combined-list.txt"
    rg 'civic-evidence|fda-document-text/diagnostic/fda-documents' "$output/combined-list.txt"
    python3 scripts/trials.py --runner "$runner" --executable "${CARGO_TARGET_DIR:-target}/debug/biomcp" --installed \
      drug-civic/drug/civic-evidence-public.yaml drug-civic/drug/civic-evidence-streams.yaml variant-civic/variant/civic-evidence-filters.yaml variant-civic/variant/native-civic.yaml fda-document-text/diagnostic/fda-documents.yaml
    make current-schema-check schema-release-check docs-check
    ;;
  generate)
    make schema-generate docs-generate
    python3 scripts/refresh_help_cases.py "${CARGO_TARGET_DIR:-target}/debug/biomcp"
    mkdir -p target/civic-evidence
    git diff --binary > target/civic-evidence/generated.patch
    ;;
  prepare)
    make --no-print-directory civic-evidence-focused
    sh "$0" generate
    ;;
  scans)
    baseline=${2:?scans requires the reviewed baseline commit}
    output="${CARGO_TARGET_DIR:-target}/civic-evidence-scans"
    mkdir -p "$output/baseline-source"
    git archive "$baseline" crates/biomcp-adapters/src/civic crates/biomcp-types/src/drug/query.rs crates/biomcp-types/src/query/variant.rs | tar -xf - -C "$output/baseline-source"
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
    "$scanner" "$output/baseline-source" --format rust --max-size 10mb --reporters json --output "$output/baseline"
    "$scanner" crates/biomcp-adapters/src/civic crates/biomcp-types/src/drug/query.rs crates/biomcp-types/src/query/variant.rs crates/biomcp-types/src/query/civic_filter.rs --format rust --max-size 10mb --reporters json --output "$output/current"
    ruff check --select F401,F841,ERA,BLE,E722,S110 scripts/civic_evidence_fixtures.py scripts/civic_variant_fixtures.py scripts/check_civic_evidence_compatibility.py
    ;;
  cheap) make check-cheap ;;
  routine) exec sh scripts/check_nightly_tests.sh routine ;;
  *) echo 'usage: check_civic_evidence.sh [prepare|focused|combined|generate|scans BASELINE_COMMIT|cheap|routine]' >&2; exit 2 ;;
esac
