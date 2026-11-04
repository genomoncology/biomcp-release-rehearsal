#!/bin/sh
# Build once, then exercise the same installed product and recorded native pages.
set -eu
export RUSTUP_TOOLCHAIN=1.95.0
case "${1:-focused}" in
  prepare)
    mkdir -p target/label-search-proof
    cargo fmt --all
    UPDATE_SCHEMAS=1 cargo test --locked -p biomcp-types --features schema --test schemas checked_in_schemas_match_types -- --exact
    make docs-generate
    git diff --binary > target/label-search-proof/generated.patch
    ;;
  verify)
    sh "$0" focused
    sh "$0" cheap
    python3 scripts/label_search_proof.py scan
    sh "$0" live
    python3 scripts/label_search_proof.py retain
    ;;
  focused)
    mkdir -p target/label-search-proof
    cargo build --locked -p biomcp --bin biomcp --test cases --message-format=json > target/label-search-proof/build.jsonl
    python3 scripts/label_search_proof.py focused
    ;;
  cheap) make check-cheap ;;
  diagnose) python3 scripts/label_search_proof.py diagnose ;;
  live) python3 scripts/label_search_proof.py live ;;
  routine) exec sh scripts/check_nightly_tests.sh routine ;;
  *) exit 2 ;;
esac
