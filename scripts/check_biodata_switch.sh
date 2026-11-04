#!/bin/sh
# Check the dependency switch in a physical checkout with job-owned output.
set -eu
phase=${1:-check}
revision=${3:-$(git rev-parse HEAD)}
export BIOMCP_BUILD_REVISION="$revision"
if [ "$phase" = gate ]; then
    [ "$(git rev-parse HEAD)" = "$revision" ] || exit 1
    python_bin=$(python3.13 -c 'import pathlib, sys; print(pathlib.Path(sys.executable).resolve().parent)')
    export PATH="$python_bin:$PATH"
    exec make landing-gate
fi
root=${2:-$(mktemp -d "${TMPDIR:-/tmp}/biomcp-0459.XXXXXX")}
case "$root" in /private/tmp/biomcp-0459.*|/tmp/biomcp-0459.*) ;; *) echo 'unexpected qualification root' >&2; exit 1;; esac
if [ "$phase" = fixture ] || [ "$phase" = policy ] || [ "$phase" = docs ] || [ "$phase" = docs-generate ]; then
    [ "$(git rev-parse HEAD)" = "$revision" ] || exit 1
    export CARGO_TARGET_DIR="$root/target"
    python_bin=$(python3.13 -c 'import pathlib, sys; print(pathlib.Path(sys.executable).resolve().parent)')
    export PATH="$python_bin:$PATH"
    python3 --version
    if [ "$phase" = fixture ]; then
        exec sh scripts/check_release_recipe.sh schema-reuse
    fi
    if [ "$phase" = docs-generate ]; then
        make docs-generate
        tar -czf "$root/generated-docs-$revision.tar.gz" docs/reference
        exit 0
    fi
    if [ "$phase" = docs ]; then
        exec make docs-check
    fi
    exec make check-cheap
fi
source="$root/source-$revision"
mkdir -p "$source"
git archive "$revision" | /usr/bin/tar -xf - -C "$source"
if [ "$phase" = prepare ]; then
    baseline=${4:?same-main schema reference required}
    mkdir -p "$root/reference"
    git archive "$baseline" crates/biomcp-types/schemas | /usr/bin/tar -xf - -C "$root/reference"
    export BIOMCP_SCHEMA_BASELINE="$root/reference/crates/biomcp-types/schemas"
fi
cd "$source"
export CARGO_TARGET_DIR="$root/target"
printf 'Qualification root: %s\n' "$root"
rustc --version
python_bin=$(python3.13 -c 'import pathlib, sys; print(pathlib.Path(sys.executable).resolve().parent)')
export PATH="$python_bin:$PATH"
python3 --version
case "$phase" in
cheap)
    make check-cheap
    ;;
vendor)
    python3 release/vendor.py -- cargo metadata --frozen --format-version 1 > "$root/offline-metadata.json"
    ;;
check)
    cargo check --workspace --all-targets --locked
    ;;
qualify)
    status=0
    cargo test --locked -p biomcp-types --features schema --tests || status=1
    python3 scripts/schema_release.py || status=1
    cargo build --locked -p biomcp --bin biomcp || exit 1
    BIOMCP_TRANSCRIPT_DIR="$root/replies" cargo test --locked -p biomcp --test cases -- --test-threads=4 || status=1
    exit "$status"
    ;;
cases)
    cargo build --locked -p biomcp --bin biomcp
    BIOMCP_TRANSCRIPT_DIR="$root/replies" cargo test --locked -p biomcp --test cases -- --test-threads=4
    ;;
contracts)
    status=0
    cargo test --locked --no-fail-fast -p biomcp-types --features schema --test biodata_switch --test gene_ids --test protein_contracts --test rust_api --test value_conventions --test variant_values || status=1
    cargo test --locked -p biomcp-types --features schema --test schemas policy:: || status=1
    exit "$status"
    ;;
prepare)
    status=0
    cargo test --locked --no-fail-fast -p biomcp-types --features schema --test biodata_switch --test gene_ids --test protein_contracts --test rust_api --test value_conventions --test variant_values || status=1
    if UPDATE_SCHEMAS=1 cargo test --locked -p biomcp-types --features schema --test schemas checked_in_schemas_match_types -- --exact; then
        tar -czf "$root/consumer-schemas-$revision.tar.gz" -C crates/biomcp-types/schemas .
    else
        status=1
    fi
    cargo test --locked -p biomcp-types --features schema --test schemas || status=1
    exit "$status"
    ;;
case)
    cargo build --locked -p biomcp --bin biomcp
    cargo test --locked -p biomcp --test cases -- "${4:?case name required}" --exact --test-threads=1
    ;;
*) echo 'unknown switch check phase' >&2; exit 1;;
esac
