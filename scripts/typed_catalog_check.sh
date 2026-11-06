#!/bin/sh
set -eu
export RUSTUP_TOOLCHAIN=1.95.0
export BIOMCP_TRANSCRIPT_DIR="$(pwd)/target/typed-catalog-transcripts"
cargo fmt --all -- --check
cargo test --locked -p biomcp --test cases --no-run
python3 scripts/measure_typed_catalog.py "${CARGO_TARGET_DIR:-target}/debug/biomcp"
cargo test --locked -p biomcp --test cases mcp-parity/ -- --nocapture
