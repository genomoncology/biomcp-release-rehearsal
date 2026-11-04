#!/bin/sh
# Run the existing HGNC checks with this host's already installed pinned tools.
set -eu
if [ "${1:-focused}" = routine ]; then
    cd "$(dirname "$0")/.."
    exec python3 -c 'import json, os; os.execlp("sh", "sh", "-c", json.load(open("sdlc/pm.json"))["ticketsGate"])'
fi
export PATH="$HOME/.rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:$HOME/.local/share/uv/python/cpython-3.13.17-linux-x86_64-gnu/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.95.0
out=target/0596-proofs
mkdir -p "$out"
git rev-parse HEAD > "$out/source.txt"
case "${1:-focused}" in
    focused)
        python3 scripts/trials.py --target tools --list > "$out/tools-list.txt"
        python3 scripts/trials.py --target tools --proofs --list > "$out/proofs-list.txt"
        python3 scripts/trials.py --target tools --installed \
            --executable /tmp/biomcp-0596-proof/biomcp \
            cache-resilience/hgnc-trials-during-live-sync
        python3 scripts/trials.py --target tools --proofs --installed \
            --executable /tmp/biomcp-0596-proof/biomcp \
            cache-resilience/hgnc-trials-after-killed-sync \
            cache-resilience/hgnc-first-snapshot-recovers \
            cache-resilience/hgnc-first-snapshot-wait-is-bounded
        ;;
    ordinary)
        python3 scripts/trials.py --list > "$out/cases-list.txt"
        python3 scripts/trials.py --installed --executable /tmp/biomcp-0596-proof/biomcp \
            cache-resilience/hgnc-trials-during-live-sync
        ;;
    scan)
        scanner="$HOME/.npm/_npx/f96b6f8e572bd88e/node_modules/.bin/jscpd"
        "$scanner" --version
        baseline="$out/baseline-source"
        mkdir -p "$baseline"
        set -- crates/biomcp/tests/runner/hgnc_sync.rs crates/biomcp/tests/runner/mod.rs \
            crates/biomcp/tests/runner/bundle_wait.rs crates/biomcp/tests/runner/bundle_refresh.rs
        git archive 503e732bc -- "$@" | tar -xf - -C "$baseline"
        "$scanner" "$baseline/crates/biomcp/tests/runner" --format rust --min-tokens 50 \
            --max-size 10mb --reporters console,json --output "$out/jscpd-baseline-tests"
        "$scanner" "$@" --format rust --min-tokens 50 --max-size 10mb \
            --reporters console,json --output "$out/jscpd-tests"
        echo 'Product unchanged; no product scan needed.'
        ;;
    *) exit 2 ;;
esac
