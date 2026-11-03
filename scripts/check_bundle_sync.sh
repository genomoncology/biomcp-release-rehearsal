#!/bin/sh
set -eu
export PATH="/Users/ian/.local/share/uv/python/cpython-3.13.17-macos-aarch64-none/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.95.0
export CARGO_TARGET_DIR="$PWD/target/0594-proofs"
mkdir -p "$CARGO_TARGET_DIR"
git rev-parse HEAD > "$CARGO_TARGET_DIR/source.txt"
run_check() {
    label=$1
    shift
    if "$@" > "$CARGO_TARGET_DIR/$label.log" 2>&1; then
        cat "$CARGO_TARGET_DIR/$label.log"
    else
        cat "$CARGO_TARGET_DIR/$label.log"
        return 1
    fi
}
case "${1:-focused}" in
    focused|green)
        cargo fmt --all
        git diff > "$CARGO_TARGET_DIR/format.patch"
        run_check bundle cargo test --locked -p biomcp-fetch --test bundle
        run_check runner cargo test --locked -p biomcp --test cases --no-run
        if [ "${1:-focused}" = focused ]; then
            export BIOMCP_TEST_EXECUTABLE=/Users/ian/workspace/jobs/biomcp-redux-0587/target/0587-proofs/baseline-installed/biomcp
            if run_check red cargo test --locked -p biomcp --test cases gene-sections/bundle- -- --test-threads=1; then
                echo 'Pre-change installed binary unexpectedly passed bundle regressions' >&2
                exit 1
            fi
        fi
        run_check install cargo install --locked --debug --path crates/biomcp --root "$CARGO_TARGET_DIR/installed" --force
        shasum -a 256 "$CARGO_TARGET_DIR/installed/bin/biomcp" > "$CARGO_TARGET_DIR/binary.sha256"
        export BIOMCP_TEST_EXECUTABLE="$CARGO_TARGET_DIR/installed/bin/biomcp"
        run_check green cargo test --locked -p biomcp --test cases gene-sections/bundle- -- --test-threads=1
        ;;
    scan)
        export PATH="/opt/zerobrew/opt/node@22/bin:$PATH"
        npm=/opt/zerobrew/opt/node@22/bin/npm
        run_check jscpd-version "$npm" exec --offline --yes --package=jscpd@4.0.5 -- jscpd --version
        run_check jscpd-product "$npm" exec --offline --yes --package=jscpd@4.0.5 -- jscpd --format rust,python --min-tokens 50 --max-size 10mb --reporters console,json --output "$CARGO_TARGET_DIR/jscpd-product" crates/biomcp-adapters/src/guidance.rs crates/biomcp-fetch/src/bundle/download.rs crates/biomcp-fetch/src/bundle/replay.rs crates/biomcp-query/src/sources.rs crates/biomcp/src/command/sync.rs
        run_check jscpd-tests "$npm" exec --offline --yes --package=jscpd@4.0.5 -- jscpd --format rust,yaml --min-tokens 50 --max-size 10mb --reporters console,json --output "$CARGO_TARGET_DIR/jscpd-tests" crates/biomcp/tests/runner/bundle_sync.rs crates/biomcp/tests/runner/mod.rs crates/biomcp-fetch/tests/bundle/rows.rs
        ;;
    gate) run_check routine make landing-gate ;;
    final)
        sh scripts/check_bundle_sync.sh green
        sh scripts/check_bundle_sync.sh scan
        sh scripts/check_bundle_sync.sh gate
        ;;
    *) exit 2 ;;
esac
