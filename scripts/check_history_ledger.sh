#!/bin/sh
set -eu
case "${1:-focused}" in
red) python3 -m unittest release.test_security.Screening.test_history_exceptions_require_the_exact_blob_rule_line_and_commit ;;
cheap) make check-cheap ;;
focused) python3 -m unittest release.test_security scripts.test_gate_stages ;;
main-only)
    scratch=$(mktemp -d)
    trap 'rm -rf "$scratch"' EXIT HUP INT TERM
    git init --quiet "$scratch/repo"
    git -C "$scratch/repo" fetch --quiet --no-tags "$(pwd)" HEAD:refs/heads/main
    git -C "$scratch/repo" checkout --quiet main
    python3 "$scratch/repo/release/secrets.py" history "$scratch/repo" --cache "$(pwd)/target/security-tools"
    ;;
*) exit 2 ;;
esac
