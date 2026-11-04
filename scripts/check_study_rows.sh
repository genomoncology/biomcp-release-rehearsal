#!/bin/sh
# Build and exercise source-scoped study rows with the installed outside-in runner.
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
run_installed() {
    runner_target=$1
    shift
    build_output="${CARGO_TARGET_DIR:-target}/study-rows-$runner_target.json"
    mkdir -p "$(dirname "$build_output")"
    cargo build --locked -p biomcp --bin biomcp
    cargo test --locked -p biomcp --test "$runner_target" --no-run --message-format=json > "$build_output"
    runner_path=$(python3 -c 'import json,sys; rows=[json.loads(line) for line in open(sys.argv[1])]; print(next(row["executable"] for row in rows if row.get("reason")=="compiler-artifact" and row.get("executable") and row["target"]["name"]==sys.argv[2]))' "$build_output" "$runner_target")
    python3 scripts/trials.py --runner "$runner_path" --executable "${CARGO_TARGET_DIR:-target}/debug/biomcp" --installed "$@"
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
    run_installed cases study-rows/study/rows.yaml
    ;;
  routine)
    exec sh scripts/check_nightly_tests.sh routine
    ;;
  *) echo 'usage: check_study_rows.sh [generate|list|focused NAMES...|scans|routine]' >&2; exit 2 ;;
esac
