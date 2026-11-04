#!/bin/sh
# Focused installed drug name proof; routine reuses the configured gate launcher.
set -eu
export RUSTUP_TOOLCHAIN=1.95.0
case "${1:-focused}" in
  routine) exec sh scripts/check_nightly_tests.sh routine ;;
  grammar-fixture)
    export BIOMCP_TEST_EXECUTABLE="$PWD/target/drug-names-installed/bin/biomcp"
    cargo test --locked -p biomcp --test cases grammar/ -- --nocapture
    ;;
  final-coverage)
    exec python3 scripts/timed_stage.py coverage make --no-print-directory -o docs-check -o library-tests coverage
    ;;
  review-fixes)
    sh "$0" prepare
    make check-cheap
    sh "$0" focused
    python3 scripts/drug_names_proof.py upgrade
    ;;
  prepare-focused)
    sh "$0" prepare
    sh "$0" focused
    ;;
  prepare)
    cargo fmt --all
    git diff --binary > target/drug-names-format.patch
    cargo check --locked -p biomcp
    UPDATE_SCHEMAS=1 cargo test --locked -p biomcp-types --features schema --test schemas checked_in_schemas_match_types -- --exact
    git diff --binary > target/drug-names-prepare.patch
    ;;
  scan) python3 scripts/drug_names_proof.py scan ;;
  live) python3 scripts/drug_names_proof.py live ;;
  witness) python3 scripts/drug_names_proof.py witness ;;
  parity)
    python3 scripts/drug_names_proof.py extract
    export BIOMCP_TEST_EXECUTABLE="$PWD/target/drug-names-installed/bin/biomcp"
    export BIOMCP_TRANSCRIPT_DIR="$PWD/target/drug-names-proof/transcripts"
    export BIOMCP_EVIDENCE_DIR="$PWD/target/drug-names-proof/evidence"
    cargo test --locked -p biomcp --test cases drug -- --skip drug-name-get --nocapture
    export BIOMCP_TEST_EXECUTABLE="$PWD/target/drug-names-proof/baseline-bin/biomcp"
    export BIOMCP_TRANSCRIPT_DIR="$PWD/target/drug-names-proof/baseline-transcripts"
    export BIOMCP_EVIDENCE_DIR="$PWD/target/drug-names-proof/baseline-evidence"
    cargo test --locked -p biomcp --test cases drug -- --skip drug-name-get --nocapture
    python3 scripts/drug_names_proof.py compare
    ;;
  focused)
    cargo fmt --all
    git diff --binary > target/drug-names-format.patch
    cargo install --locked --debug --path crates/biomcp --root target/drug-names-installed --force
    export BIOMCP_TEST_EXECUTABLE="$PWD/target/drug-names-installed/bin/biomcp"
    export BIOMCP_TRANSCRIPT_DIR="$PWD/target/drug-names-proof/transcripts"
    export BIOMCP_EVIDENCE_DIR="$PWD/target/drug-names-proof/evidence"
    mkdir -p target/drug-names-proof
    git rev-parse HEAD > target/drug-names-proof/source.txt
    make docs-generate
    git diff --binary > target/drug-names-docs.patch
    cargo test --locked -p biomcp --test cases drug/ -- --nocapture
    cargo test --locked -p biomcp-types --test drug_contracts
    ;;
  *) exit 2 ;;
esac
