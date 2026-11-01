#!/bin/sh
set -eu
case "${1:-focused}" in
focused)
    cargo test --locked -p biomcp --test cases corrupted-snapshot-reports-payload-error
    cargo test --locked -p biomcp --test cases name-search-selects-paper-author
    unset UPDATE_SCHEMAS
    cargo test --locked -p biomcp-types --features schema --test schemas checked_in_schemas_match_types -- --exact
    ;;
cheap)
    make fmt lint ratchets
    ;;
generate)
    make schema-generate
    git diff --stat -- crates/biomcp-types/schemas
    tar -czf snapshot-schemas.tar.gz crates/biomcp-types/schemas
    pwd
    ;;
routine)
    make landing-gate
    ;;
esac
