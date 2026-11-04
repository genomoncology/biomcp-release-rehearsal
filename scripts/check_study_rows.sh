#!/bin/sh
# Build and exercise source-scoped study rows with the installed outside-in runner.
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
run_installed() {
    runner_target=$1
    shift
    python3 scripts/trials.py --target "$runner_target" --installed "$@"
}
case "${1:-focused}" in
  generate)
    cargo check --locked --workspace
    make schema-generate docs-generate
    COPYFILE_DISABLE=1 tar -czf /tmp/biomcp-study-rows-generated.tar.gz crates/biomcp-types/schemas docs/reference/commands.md docs/recipes.md skills/SKILL.md docs/reference/source-pacing.md
    pwd
    ;;
  list)
    cargo test --locked -p biomcp --test cases -- --list | rg study
    ;;
  focused)
    shift
    run_installed cases "$@"
    ;;
  scans)
    cargo fmt --all -- --check
    cargo clippy --locked -p biomcp-adapters -p biomcp-query -p biomcp-types --features biomcp-types/schema --all-targets -- -D warnings
    run_installed cases --case-file study/rows.yaml
    ;;
  routine)
    exec sh scripts/check_nightly_tests.sh routine
    ;;
  *) echo 'usage: check_study_rows.sh [generate|list|focused NAMES...|scans|routine]' >&2; exit 2 ;;
esac
