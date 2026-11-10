#!/usr/bin/env bash
# Focused, offline public-surface proof and declaration refresh for explicit label aliases.
set -euo pipefail
case "${1:-focused}" in
  generate)
    make schema-generate docs-generate
    python3 scripts/refresh_help_cases.py target/debug/biomcp
    git diff --binary > "${2:?persistent patch destination}"
    ;;
  catalog)
    make schema-generate docs-generate
    git diff --binary > "${2:?persistent patch destination}"
    git add -u
    git write-tree
    python3 scripts/trials.py mcp-parity/typed-hosted-catalog mcp-parity/compact-typed-schemas \
      drug-label/explicit-alias-native-union --case-file drug/help.yaml
    cargo fmt --all -- --check
    ;;
  focused)
    python3 scripts/trials.py drug-label/explicit-alias-native-union drug-label/alias-uncertainty-and-controls drug-label/alias-frozen-word-tier --case-file drug/help.yaml
    cargo fmt --all -- --check
    cargo clippy --locked -p biomcp --bin biomcp --test cases --test tools -- -D warnings
    cargo test --locked -p biomcp --test ratchets
    make schema-release-check
    python3 scripts/check_docs.py
    ;;
  *) exit 2 ;;
esac
