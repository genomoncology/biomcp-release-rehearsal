#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
case "${1:-focused}" in
  repair)
    cargo fmt --all
    make check-cheap
    cargo test --locked -p biomcp-types --features schema --test adverse_event_contracts
    shift
    runner=${1:?repair requires the discovered case runner}
    shift
    python3.13 scripts/trials.py --target cases --runner "$runner" --executable "${CARGO_TARGET_DIR:-target}/debug/biomcp" --installed "$@"
    ;;
  generate)
    cargo fmt --all
    make schema-generate docs-generate
    ;;
  evidence)
    sh "$0" scans "$2"
    sh "$0" proof "$3"
    ;;
  scans)
    baseline=${2:?scans requires baseline commit}
    mkdir -p target/0606-scans/baseline
    git archive "$baseline" crates/biomcp-types/src/adverse_event crates/biomcp-adapters/src/maude | tar -xf - -C target/0606-scans/baseline
    for side in baseline current; do
      prefix=.
      test "$side" != baseline || prefix=target/0606-scans/baseline
      npm exec --offline --yes --package=jscpd@4.0.5 -- jscpd \
        "$prefix/crates/biomcp-types/src/adverse_event" "$prefix/crates/biomcp-adapters/src/maude" \
        --format rust --max-size 10mb --reporters json --output "target/0606-scans/$side"
    done
    cargo clippy --locked -p biomcp-types -p biomcp-adapters --all-targets --features biomcp-types/schema -- -D warnings
    ;;
  proof)
    uvx --offline --from ruff==0.17.2 ruff check --select F401,F841,ERA,BLE,E722,S110 scripts/maude_dates_proof.py
    sh "$0" live "$2"
    ;;
  live)
    python3.13 scripts/maude_dates_proof.py "${CARGO_TARGET_DIR:-target}/debug/biomcp" "$2" "target/0606-live"
    ;;
  list)
    python3.13 scripts/trials.py --target cases --list
    ;;
  focused)
    cargo fmt --all -- --check
    cargo test --locked -p biomcp-types --features schema --test adverse_event_contracts
    cargo test --locked -p biomcp-types --features schema --test schemas
    make current-schema-check schema-release-check
    cargo build --locked -p biomcp --bin biomcp
    shift
    runner=${1:?focused requires the discovered case runner}
    shift
    python3.13 scripts/trials.py --target cases --runner "$runner" --executable "${CARGO_TARGET_DIR:-target}/debug/biomcp" --installed "$@"
    ;;
  *) exit 2 ;;
esac
