#!/bin/sh
set -eu
export PATH="/Users/ian/.local/share/uv/python/cpython-3.13.17-macos-aarch64-none/bin:$HOME/.cargo/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.95.0
mkdir -p target/0595-proofs
if [ "${1:-}" = scan ]; then
    scanner=/Users/ian/.npm/_npx/f96b6f8e572bd88e/node_modules/.bin/jscpd
    "$scanner" --version
    baseline=target/0595-proofs/baseline-source
    mkdir -p "$baseline"
    git archive 0afa5e26afa2a9f38a90e965c844bd3f9d95ebf2 \
        crates/biomcp-types/src/response/variant.rs crates/biomcp-types/src/response/mod.rs \
        crates/biomcp-types/src/lib.rs crates/biomcp-query/src/variant/search.rs \
        crates/biomcp-adapters/src/myvariant/search.rs crates/biomcp-adapters/src/myvariant/search_build.rs \
        crates/biomcp-adapters/src/myvariant/mod.rs crates/biomcp-adapters/src/clinvar/mod.rs \
        crates/biomcp/tests/runner/variant.rs crates/biomcp/tests/runner/variant/search_policy.rs | tar -xf - -C "$baseline"
    "$scanner" "$baseline/crates/biomcp-types/src" "$baseline/crates/biomcp-query/src" "$baseline/crates/biomcp-adapters/src" --format rust --max-size 10mb --reporters json --output target/0595-proofs/jscpd-baseline-product
    "$scanner" "$baseline/crates/biomcp/tests" --format rust --max-size 10mb --reporters json --output target/0595-proofs/jscpd-baseline-tests
    "$scanner" crates/biomcp-types/src/response/variant.rs crates/biomcp-types/src/response/mod.rs crates/biomcp-types/src/lib.rs crates/biomcp-query/src/variant/search.rs crates/biomcp-adapters/src/myvariant/search.rs crates/biomcp-adapters/src/myvariant/search_build.rs crates/biomcp-adapters/src/myvariant/corroboration.rs crates/biomcp-adapters/src/myvariant/mod.rs crates/biomcp-adapters/src/clinvar/mod.rs crates/biomcp-adapters/src/clinvar/reference.rs --format rust --max-size 10mb --reporters json --output target/0595-proofs/jscpd-product
    "$scanner" crates/biomcp/tests/runner/variant.rs crates/biomcp/tests/runner/variant/search_corroboration.rs crates/biomcp/tests/runner/variant/search_policy.rs --format rust --max-size 10mb --reporters json --output target/0595-proofs/jscpd-tests
    exit
fi
if [ "${1:-}" = format ]; then
    cargo fmt --all
    git diff --binary > target/0595-proofs/format.patch
    cargo check --locked -p biomcp-query
    exit
fi
if [ "${1:-}" = schemas ]; then
    UPDATE_SCHEMAS=1 cargo test --locked -p biomcp-types --features schema --test schemas checked_in_schemas_match_types -- --exact
    git diff --binary > target/0595-proofs/schemas.patch
    exit
fi
if [ "${1:-}" = gate ]; then
    make landing-gate
    exit
fi
cargo fmt --all -- --check
if [ "${1:-}" != reuse ]; then
    cargo install --locked --debug --path crates/biomcp --root target/0595-proofs/installed --force
fi
export BIOMCP_TEST_EXECUTABLE="$PWD/target/0595-proofs/installed/bin/biomcp"
export BIOMCP_EVIDENCE_DIR="$PWD/target/0595-proofs/evidence"
export BIOMCP_TRANSCRIPT_DIR="$PWD/target/0595-proofs/replies/$(git rev-parse HEAD)"
cargo test --locked -p biomcp --test cases variant-search/ -- --nocapture

python3 scripts/snpeff_parity.py "${1:-focused}" $(git diff --name-only 0afa5e26afa2a9f38a90e965c844bd3f9d95ebf2 HEAD -- crates scripts/snpeff_fallback_job.sh scripts/snpeff_parity.py sdlc/features.md)
if [ "${1:-}" = final ] || [ "${1:-}" = reuse ]; then
    sh scripts/snpeff_fallback_job.sh scan
    unset BIOMCP_TEST_EXECUTABLE BIOMCP_TRANSCRIPT_DIR BIOMCP_EVIDENCE_DIR
    make landing-gate
fi
