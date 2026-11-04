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
    python3 scripts/trials.py --installed --case-file cell-line/filters.yaml --case-file cell-line/examples.yaml --case-file cell-line/help.yaml
    cargo test --locked -p biomcp-types --features schema --test cell_line_contracts
    python3 scripts/check_cell_line_compatibility.py "${2:?baseline executable required}" "${CARGO_TARGET_DIR:-target}/debug/biomcp"
    ;;
  scans)
    python3 scripts/scan_cell_line_filters.py "${2:?reviewed baseline required}"
    ;;
  *) echo 'usage: check_cell_line_filters.sh [generate|focused BASELINE_EXECUTABLE|scans BASELINE_COMMIT]' >&2; exit 2 ;;
esac
