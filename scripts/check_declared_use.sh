#!/bin/sh
# Run source-use checks with the installed binary and inherited remote tool paths.
set -eu
export PATH="/Users/ian/.local/share/uv/python/cpython-3.13.17-macos-aarch64-none/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.95.0
phase=${1:-focused}
output="/Users/ian/workspace/t/biomcp-redux-0585-${PM_JOB:?}"
mkdir -p "$output"
case "$phase" in
  format)
    cargo fmt --all
    UPDATE_SCHEMAS=1 cargo test --locked -p biomcp-types --features schema --test schemas checked_in_schemas_match_types -- --exact
    git diff > "$output/generated.patch"
    tar -czf "$output/declared-use-generated.tar.gz" crates/biomcp-types/schemas
    ;;
  focused)
    cargo fmt --all
    git diff > "$output/generated.patch"
    cargo build --locked -p biomcp --bin biomcp
    cargo test --locked -p biomcp --test cases --no-run
    mkdir -p "$output/installed/bin"
    cp "${CARGO_TARGET_DIR:-target}/debug/biomcp" "$output/installed/bin/biomcp"
    export BIOMCP_TEST_EXECUTABLE="$output/installed/bin/biomcp"
    export BIOMCP_LEGACY_EXECUTABLE="/Users/ian/workspace/t/biomcp-redux-0585-baseline-3d25a0ea5/installed/bin/biomcp"
    python3 scripts/test_reply_diff.py
    cargo test --locked -p biomcp --test cases mcp-hosted/use/ -- --test-threads=4
    ;;
  capture)
    cargo build --locked -p biomcp --bin biomcp
    mkdir -p "$output/installed/bin"
    cp "${CARGO_TARGET_DIR:-target}/debug/biomcp" "$output/installed/bin/biomcp"
    export BIOMCP_TEST_EXECUTABLE="$output/installed/bin/biomcp"
    export BIOMCP_LEGACY_EXECUTABLE="/Users/ian/workspace/t/biomcp-redux-0585-baseline-3d25a0ea5/installed/bin/biomcp"
    export BIOMCP_TRANSCRIPT_DIR="$output/replies"
    cargo test --locked -p biomcp --test cases -- --test-threads=4
    python3 scripts/reply_diff.py --all-fields /Users/ian/workspace/t/biomcp-redux-0585-baseline-3d25a0ea5/replies "$output/replies" > "$output/reply-diff.jsonl" || [ "$?" -eq 1 ]
    ;;
  gate)
    make landing-gate
    make schema-release-check
    exec sh scripts/check_declared_use.sh capture
    ;;
  *) exit 2 ;;
esac
