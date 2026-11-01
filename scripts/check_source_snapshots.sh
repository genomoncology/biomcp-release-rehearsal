#!/bin/sh
set -eu
case "${1:-cases}" in
qualify)
    sh "$0" cheap
    sh "$0" cases
    ;;
cheap)
    make fmt lint ratchets
    ;;
focused)
    cargo test --locked -p biomcp --test cases cache-resilience/
    cargo test --locked -p biomcp --test cases author/
    cargo test --locked -p biomcp --test cases author-papers/
    cargo test --locked -p biomcp --test cases pathway/
    cargo test --locked -p biomcp --test cases phenotype/
    cargo test --locked -p biomcp --test cases pgx/
    cargo test --locked -p biomcp --test cases maude/
    cargo test --locked -p biomcp --test cases trial-get/
    unset UPDATE_SCHEMAS
    cargo test --locked -p biomcp-types --features schema --test schemas checked_in_schemas_match_types -- --exact
    ;;
cases)
    cargo test --locked -p biomcp --test cases
    unset UPDATE_SCHEMAS
    cargo test --locked -p biomcp-types --features schema --test schemas checked_in_schemas_match_types -- --exact
    ;;
*)
    echo "expected cheap, focused, cases or qualify" >&2
    exit 2
    ;;
esac
