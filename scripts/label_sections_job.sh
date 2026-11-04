#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
case "${1:-focused}" in
  generate)
    cargo fmt --all
    make schema-generate docs-generate
    mkdir -p target
    git diff --binary > target/0608-generated.patch
    git status --short
    sh "$0" list
    ;;
  build-cases)
    mkdir -p target
    cargo test --locked -p biomcp --test cases --no-run --message-format=json > target/0608-cargo.json
    python3.13 -c 'import json; from pathlib import Path; rows = [json.loads(line) for line in Path("target/0608-cargo.json").read_text().splitlines()]; print(next(row["executable"] for row in rows if row.get("reason") == "compiler-artifact" and row.get("executable") and row["target"]["name"] == "cases"))' > target/0608-runner
    ;;
  list)
    sh "$0" build-cases
    runner=$(cat target/0608-runner)
    "$runner" --list
    ;;
  focused|installed)
    if [ "$1" = focused ]; then
      make check-cheap
    else
      cargo build --locked -p biomcp --bin biomcp
    fi
    shift
    sh "$0" build-cases
    runner=$(cat target/0608-runner)
    python3.13 scripts/trials.py --target cases --runner "$runner" --executable "${CARGO_TARGET_DIR:-target}/debug/biomcp" --installed "$@"
    ;;
  scans)
    baseline=${2:?scans requires baseline commit}
    mkdir -p target/0608-scans/baseline
    git archive "$baseline" crates/biomcp-types/src/drug crates/biomcp-adapters/src/openfda_label crates/biomcp-query/src/drug crates/biomcp/tests/runner | tar -xf - -C target/0608-scans/baseline
    for side in baseline current; do
      prefix=.
      test "$side" != baseline || prefix=target/0608-scans/baseline
      npm exec --offline --yes --package=jscpd@4.0.5 -- jscpd "$prefix/crates/biomcp-types/src/drug" "$prefix/crates/biomcp-adapters/src/openfda_label" "$prefix/crates/biomcp-query/src/drug" --format rust --max-size 10mb --reporters json --output "target/0608-scans/$side-product"
      npm exec --offline --yes --package=jscpd@4.0.5 -- jscpd "$prefix/crates/biomcp/tests/runner" --format rust --max-size 10mb --reporters json --output "target/0608-scans/$side-tests"
    done
    ;;
  case-lint) cargo clippy --locked -p biomcp --test cases --test tools -- -D warnings ;;
  section-lint) cargo clippy --locked -p biomcp-query --lib -- -D warnings ;;
  verify)
    shift
    sh "$0" installed "$@"
    sh scripts/check_nightly_tests.sh routine
    ;;
  routine) exec sh scripts/check_nightly_tests.sh routine ;;
  *) exit 2 ;;
esac
