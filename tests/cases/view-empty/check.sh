#!/bin/sh
# Run installed Markdown cases, retaining their complete sanitized replies.
set -eu
python="$(uv python find --offline 3.13.17)"
export PATH="$(dirname "$python"):$HOME/.cargo/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.95.0
export BIOMCP_TRANSCRIPT_DIR="$PWD/target/view-empty/transcripts"
export BIOMCP_EVIDENCE_DIR="$PWD/target/view-empty/fixtures"
mkdir -p "$PWD/target/view-empty"
if [ "${1:-focused}" = final ]; then
    sh "$0" focused
    sh "$0" parity
    sh "$0" scans
    exec sh scripts/check_nightly_tests.sh routine
fi
if [ "${1:-focused}" = parity ]; then
    scp yellow:/home/ian/workspace/data/biomcp-redux-evidence/0582/installed/bin/biomcp target/view-empty/baseline-biomcp
    scp yellow:/home/ian/workspace/data/biomcp-redux-evidence/0582/source-bindings.json target/view-empty/baseline-source-bindings.json
    python3 tests/cases/view-empty/proof.py prepare
    chmod +x target/view-empty/baseline-biomcp
    export BIOMCP_TRANSCRIPT_DIR="$PWD/target/view-empty/baseline-transcripts"
    python3 scripts/trials.py --installed --executable target/view-empty/baseline-biomcp --cases target/view-empty/json-cases --case-file empty-section.yaml --case-file failed-section.yaml --case-file all-absent.yaml --case-file table-section.yaml
    python3 tests/cases/view-empty/proof.py compare
    exit
fi
if [ "${1:-focused}" = scans ]; then
    python3 tests/cases/view-empty/proof.py scans
    exit
fi
python3 scripts/trials.py --installed --case-file view-empty/empty-section.yaml --case-file view-empty/failed-section.yaml --case-file view-empty/all-absent.yaml --case-file view-empty/table-section.yaml
cargo fmt --all -- --check
