#!/bin/sh
set -eu
case "${1:-focused}" in
red) cargo test --locked -p biomcp --test cases source-pacing/settings-and-ceilings ;;
cheap) make fmt lint ratchets ;;
review) sh "$0" cheap; sh "$0" focused ;;
binding)
    make fmt
    cargo test --locked -p biomcp --bin biomcp grammar::reference::reference_matches_grammar
    python3 scripts/check_docs.py
    ;;
focused)
    cargo test --locked -p biomcp-fetch --test quotas
    cargo test --locked -p biomcp-adapters --test source_declarations
    cargo test --locked -p biomcp --test cases source-pacing/
    make docs-generate
    make docs-check
    ;;
*) exit 2 ;;
esac
