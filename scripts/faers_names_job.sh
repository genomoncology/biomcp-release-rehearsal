#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
case "${1:-focused}" in
  generate) cargo fmt --all; make schema-generate docs-generate ;;
  list) python3.13 scripts/trials.py --target cases --list ;;
  focused)
    make check-cheap
    cargo test --locked -p biomcp-types --features schema --test adverse_event_contracts
    cargo build --locked -p biomcp --bin biomcp
    shift
    runner=${1:?focused requires discovered runner}; shift
    python3.13 scripts/trials.py --target cases --runner "$runner" --executable "${CARGO_TARGET_DIR:-target}/debug/biomcp" --installed "$@"
    ;;
  proof)
    uvx --offline --from ruff==0.17.2 ruff check --select F401,F841,ERA,BLE,E722,S110 scripts/faers_names_proof.py
    python3.13 scripts/faers_names_proof.py "${CARGO_TARGET_DIR:-target}/debug/biomcp" "$2" target/0609-live
    ;;
  scans)
    baseline=${2:?scans requires baseline commit}
    mkdir -p target/0609-scans/baseline
    git archive "$baseline" crates/biomcp-types/src/adverse_event crates/biomcp-adapters/src/faers crates/biomcp/src/retired/adverse_event.rs | tar -xf - -C target/0609-scans/baseline
    for side in baseline current; do
      prefix=.
      test "$side" != baseline || prefix=target/0609-scans/baseline
      npm exec --offline --yes --package=jscpd@4.0.5 -- jscpd "$prefix/crates/biomcp-types/src/adverse_event" "$prefix/crates/biomcp-adapters/src/faers" "$prefix/crates/biomcp/src/retired/adverse_event.rs" --format rust --max-size 10mb --reporters json --output "target/0609-scans/$side"
    done
    cargo clippy --locked -p biomcp-types -p biomcp-adapters --all-targets --features biomcp-types/schema -- -D warnings
    ;;
  *) exit 2 ;;
esac
