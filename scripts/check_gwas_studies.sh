#!/bin/sh
# Check native GWAS publication study and bounded association search behavior.
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
if test -n "${2:-}"; then
  export BIOMCP_GWAS_PRIOR_STUDIES="$PWD/target/gwas-studies-evidence/$2/studies"
fi
case "${1:-focused}" in
  prepare)
    cargo fmt --all
    cargo check --locked -p biomcp
    cargo test --locked -p biomcp --test cases -- --list
    UPDATE_SCHEMAS=1 cargo test --locked -p biomcp-types --features schema --test schemas checked_in_schemas_match_types -- --exact
    make docs-generate
    ;;
  focused)
    export BIOMCP_BUILD_REVISION=$(git rev-parse HEAD)
    make docs-generate
    cargo build --locked -p biomcp --bin biomcp
    mkdir -p target/gwas-studies
    cargo test --locked -p biomcp --test cases --no-run --message-format=json > target/gwas-studies/artifact.json
    runner=$(python3 -c 'import json; from pathlib import Path; print(next(r["executable"] for line in Path("target/gwas-studies/artifact.json").read_text().splitlines() if (r := json.loads(line)).get("reason") == "compiler-artifact" and r.get("executable") and r["target"]["name"] == "cases"))')
    product=${CARGO_TARGET_DIR:-target}/debug/biomcp
    python3 scripts/trials.py --target cases --runner "$runner" --list
    python3 scripts/trials.py --target cases --runner "$runner" --executable "$product" --installed gwas-search/gwas/examples.yaml gwas-get/gwas/get-examples.yaml gwas-search/gwas/help.yaml gwas-search/gwas/studies.yaml
    ;;
  live)
    export BIOMCP_BUILD_REVISION=$(git rev-parse HEAD)
    cargo build --locked -p biomcp --bin biomcp
    sh "$0" installed
    ;;
  installed)
    BIOMCP_TEST_EXECUTABLE="${CARGO_TARGET_DIR:-target}/debug/biomcp" python3 scripts/check_gwas_studies_installed.py
    ;;
  proof)
    sh "$0" focused
    sh "$0" installed
    ;;
  scans) make check-cheap ;;
  citation)
    make check-cheap
    cargo test --locked -p biomcp-adapters --test source_declarations
    ;;
  review)
    make check-cheap
    sh "$0" focused
    ;;

  routine) exec sh scripts/check_nightly_tests.sh routine ;;
  *) echo 'usage: check_gwas_studies.sh [prepare|focused|installed|scans|citation|review|proof|live|routine]' >&2; exit 2 ;;
esac
