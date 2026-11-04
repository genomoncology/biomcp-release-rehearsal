#!/bin/sh
# Source-bound submission ordering and installed paging checks.
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
python_bin=$(python3.13 -c 'from pathlib import Path; import sys; print(Path(sys.executable).resolve().parent)')
export PATH="$python_bin:$PATH"
case "${1:-focused}" in
  prepare)
    python3.13 scripts/schema_release.py --generate
    output="$HOME/workspace/t/biomcp-redux-0601-$(git rev-parse HEAD)"
    mkdir -p "$output"
    git diff --binary > "$output/generated.patch"
    printf 'Generated patch: %s/generated.patch\n' "$output"
    ;;
  focused)
    output="$HOME/workspace/t/biomcp-redux-0601-$(git rev-parse HEAD)"
    mkdir -p "$output/transcripts"
    cargo build --locked -p biomcp --bin biomcp
    cargo test --locked -p biomcp --test cases --no-run --message-format=json > "$output/case-artifacts.jsonl"
    runner=$(python3.13 - "$output/case-artifacts.jsonl" <<'RUNNER'
import json
from pathlib import Path
import sys
artifacts = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]
print(next(item['executable'] for item in artifacts if item.get('reason') == 'compiler-artifact'
           and item.get('executable') and item['target']['name'] == 'cases'))
RUNNER
    )
    binary="${CARGO_TARGET_DIR:-target}/debug/biomcp"
    BIOMCP_TRANSCRIPT_DIR="$output/transcripts" python3.13 scripts/trials.py --installed --runner "$runner" --executable "$binary" drug-approvals/newest-submission-pages drug-approvals/undated-and-numeric-ties
    ;;
  review)
    sh scripts/check_trial_results.sh scans "${2:?review requires the explicit reviewed baseline commit}"
    make check-cheap
    ;;
  cheap)
    make check-cheap
    ;;
  *) echo 'usage: check_approvals_order.sh [prepare|focused|review BASE_COMMIT|cheap]' >&2; exit 2 ;;
esac
