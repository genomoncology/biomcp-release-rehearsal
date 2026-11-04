#!/bin/sh
# Check native cell-line filters through the installed CLI and MCP replay surfaces.
set -eu
cd "$(dirname "$0")/.."
case "${1:-focused}" in
  generate)
    cargo fmt --all
    UPDATE_SCHEMAS=1 cargo test --locked -p biomcp-types --features schema --test schemas checked_in_schemas_match_types -- --exact
    make docs-generate
    mkdir -p target/cell-line-filters
    git diff > target/cell-line-filters/generated.patch
    ;;
  focused)
    mkdir -p target/cell-line-filters
    cargo build --locked -p biomcp --bin biomcp
    cargo test --locked -p biomcp --test cases --no-run --message-format=json > target/cell-line-filters/runner-build.jsonl
    case_runner=$(python3 -c 'import json; from pathlib import Path; rows=[json.loads(line) for line in Path("target/cell-line-filters/runner-build.jsonl").read_text().splitlines()]; print(next(row["executable"] for row in rows if row.get("reason")=="compiler-artifact" and row["target"]["name"]=="cases" and row.get("executable")))')
    python3 scripts/trials.py --runner "$case_runner" --list > target/cell-line-filters/trials.txt
    python3 -c 'from pathlib import Path; print("\n".join(line for line in Path("target/cell-line-filters/trials.txt").read_text().splitlines() if line.startswith("cell-line-identity/cell-line/")))'
    python3 scripts/trials.py --runner "$case_runner" --executable "${CARGO_TARGET_DIR:-target}/debug/biomcp" --installed cell-line-identity/cell-line/filters.yaml cell-line-identity/cell-line/examples.yaml cell-line-identity/cell-line/help.yaml
    cargo test --locked -p biomcp-types --features schema --test cell_line_contracts
    python3 scripts/check_cell_line_compatibility.py "${2:?baseline executable required}" "${CARGO_TARGET_DIR:-target}/debug/biomcp"
    ;;
  scans)
    python3 scripts/scan_cell_line_filters.py "${2:?reviewed baseline required}"
    ;;
  *) echo 'usage: check_cell_line_filters.sh [generate|focused BASELINE_EXECUTABLE|scans BASELINE_COMMIT]' >&2; exit 2 ;;
esac
