#!/bin/sh
# Exercise FAERS reaction counts at the caller boundary.
set -eu
cd "$(dirname "$0")/.."
faers_python_bin=$(python3.13 -c 'from pathlib import Path; import sys; print(Path(sys.executable).resolve().parent)')
export PATH="$faers_python_bin:$HOME/.cargo/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.95.0
case "${1:-focused}" in
  generate)
    cargo fmt --all
    make schema-generate docs-generate
    ;;
  focused)
    cargo test --locked -p biomcp-types --features schema --test adverse_event_contracts
    for faers_case in faers-reaction-counts examples faers-several-names help; do
      cargo test --locked -p biomcp --test cases -- "adverse-event-search/adverse-event/$faers_case.yaml" --exact
    done
    cargo test --locked -p biomcp --test tools -- --cases "$PWD/tests/tool-cases" adverse-event-search/adverse-event/faers-reaction-counts.yaml --exact
    ;;
  scans)
    baseline=${2:?scans requires the current main baseline}
    mkdir -p target/0662-scans/baseline
    git archive "$baseline" crates/biomcp-types/src/adverse_event crates/biomcp-query/src/adverse_event.rs crates/biomcp-query/src/adverse_event crates/biomcp-adapters/src/faers > target/0662-scans/baseline.tar
    rm -rf target/0662-scans/baseline
    mkdir -p target/0662-scans/baseline
    tar -xf target/0662-scans/baseline.tar -C target/0662-scans/baseline
    for side in baseline current; do
      prefix=.
      test "$side" != baseline || prefix=target/0662-scans/baseline
      npm exec --offline --yes --package=jscpd@4.0.5 -- jscpd "$prefix/crates/biomcp-types/src/adverse_event" "$prefix/crates/biomcp-query/src/adverse_event.rs" "$prefix/crates/biomcp-query/src/adverse_event" "$prefix/crates/biomcp-adapters/src/faers" --format rust --max-size 10mb --reporters console,json --output "target/0662-scans/$side"
    done
    ;;
  mechanical)
    cargo fmt --all -- --check
    cargo clippy --keep-going --locked -p biomcp-types -p biomcp-adapters -p biomcp-query -p biomcp --all-targets --features biomcp-types/schema -- -D warnings
    cargo test --locked -p biomcp --test ratchets -- repository_obeys_structural_contract --exact
    ;;
  guidance)
    cargo test --locked -p biomcp --test cases -- query-validation/grammar/numeric-description-guidance.yaml --exact
    ;;
  routine)
    sh scripts/faers_counts_job.sh scans "${2:?routine requires the current main baseline}"
    make landing-gate
    ;;
  *) exit 2 ;;
esac
