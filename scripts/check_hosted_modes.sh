#!/bin/sh
# Exercise hosted source rights through the installed native binary.
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
python_bin=$(python3.13 -c 'from pathlib import Path; import sys; print(Path(sys.executable).resolve().parent)')
export PATH="$python_bin:$PATH"
case "${1:-focused}" in
  focused)
    mkdir -p "${CARGO_TARGET_DIR:-target}"
    cargo build --locked -p biomcp --bin biomcp
    build_output="${CARGO_TARGET_DIR:-target}/hosted-mode-runner.jsonl"
    cargo test --locked -p biomcp --test cases --no-run --message-format=json > "$build_output"
    runner_path=$(python3 -c 'import json,sys; rows=[json.loads(line) for line in open(sys.argv[1])]; print(next(row["executable"] for row in rows if row.get("reason")=="compiler-artifact" and row.get("executable") and row["target"]["name"]=="cases"))' "$build_output")
    python3 - "$runner_path" "${CARGO_TARGET_DIR:-target}/debug/biomcp" <<'CASES'
import subprocess
import sys
listed = subprocess.check_output(
    [sys.executable, 'scripts/trials.py', '--runner', sys.argv[1], '--list'], text=True)
names = [line.removesuffix(': test') for line in listed.splitlines()
         if line.endswith(': test') and 'mcp-hosted/use/' in line]
if not names:
    raise SystemExit('No hosted use trials found in runner catalog')
print('\n'.join(names), flush=True)
subprocess.run([sys.executable, 'scripts/trials.py', '--runner', sys.argv[1],
                '--executable', sys.argv[2], '--installed', *names], check=True)
CASES
    ;;
  structural)
    cargo test --locked -p biomcp --test ratchets repository_obeys_structural_contract -- --exact
    ;;
  cheap)
    make check-cheap
    make docs-check
    ;;
  scans)
    exec sh scripts/check_trial_results.sh scans "${2:?scans requires baseline}"
    ;;
  routine)
    exec sh scripts/check_nightly_tests.sh routine
    ;;
  *) echo 'usage: check_hosted_modes.sh [focused|structural|cheap|scans BASE_COMMIT|routine]' >&2; exit 2 ;;
esac
