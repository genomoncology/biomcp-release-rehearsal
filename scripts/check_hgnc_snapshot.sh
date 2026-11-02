#!/bin/sh
# Focused routine checks for SQLite diagnostics and first-snapshot recovery.
set -eu
cd "$(dirname "$0")/.."
export PATH="$HOME/.local/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.95.0
case "${1:-}" in
    '')
        cargo fmt --all -- --check
        cargo clippy --locked -p biomcp-fetch -p biomcp-query --all-targets -- -D warnings
        cargo clippy --locked -p biomcp --test tools -- -D warnings
        cargo test --locked -p biomcp-fetch --lib sqlite_diagnostic
        cargo test --locked -p biomcp-fetch --lib cache::contention
        ;;
    --proofs-only) ;;
    *) echo "Usage: $0 [--proofs-only]" >&2; exit 2 ;;
esac
python3.13 scripts/trials.py --target tools --proofs \
    cache-resilience/hgnc-first-snapshot-recovers \
    cache-resilience/hgnc-trials-during-live-sync \
    cache-resilience/hgnc-trials-after-killed-sync \
    cache-resilience/hgnc-first-snapshot-wait-is-bounded
python3.13 scripts/trials.py --target cases \
    store-coexistence/interrupted-transfer-retains-both-files
