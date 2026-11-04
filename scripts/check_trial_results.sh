#!/bin/sh
# Posted trial results: generated contracts and installed, source-backed cases.
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
python_bin=$(python3.13 -c 'from pathlib import Path; import sys; print(Path(sys.executable).resolve().parent)')
export PATH="$python_bin:$PATH"
case "${1:-focused}" in
  prepare)
    python3.13 scripts/schema_release.py --generate
    make docs-generate
    cargo build --locked -p biomcp --bin biomcp
    python3.13 scripts/refresh_help_cases.py "${CARGO_TARGET_DIR:-target}/debug/biomcp"
    output="$HOME/workspace/t/biomcp-redux-0607-$(git rev-parse HEAD)"
    mkdir -p "$output"
    git diff --binary > "$output/generated.patch"
    printf 'Generated patch: %s/generated.patch\n' "$output"
    ;;
  focused)
    python3.13 scripts/trials.py --installed trial-results/posted-values-and-event-pages trial-results/no-posted-results
    cargo test --locked -p biomcp --test cases trial/get/
    cargo test --locked -p biomcp --test cases trial/sites/
    cargo test --locked -p biomcp --test cases trial/references
    cargo test --locked -p biomcp --test cases trial/help
    make docs-check
    make schema-release-check
    ;;
  schema)
    cargo test --locked -p biomcp-types --features schema
    ;;
  cheap)
    make check-cheap
    ;;
  review)
    make docs-check
    make schema-release-check
    sh "$0" scans "${2:?review requires the explicit reviewed baseline commit}"
    ;;
  scans)
    baseline=${2:?scans requires the explicit reviewed baseline commit}
    export TRIAL_RESULTS_BASELINE="$baseline"
    python3.13 - <<'SCANS'
import json
from pathlib import Path
import os
import subprocess

root = Path('target/trial-results-scans')
root.mkdir(parents=True, exist_ok=True)
for package in (Path.home() / '.npm/_npx').glob('*/node_modules/jscpd/package.json'):
    if json.loads(package.read_text())['version'] == '4.0.5':
        scanner = package.parent.parent / '.bin/jscpd'
        break
else:
    raise SystemExit('cached jscpd 4.0.5 is required')
subprocess.run([str(scanner), '--version'], check=True)
baseline = os.environ['TRIAL_RESULTS_BASELINE']
paths = subprocess.check_output(['git', 'diff', '--name-only', baseline, 'HEAD', '--', '*.rs'], text=True).splitlines()
previous = []
for name in paths:
    value = subprocess.run(['git', 'show', baseline + ':' + name], capture_output=True)
    if value.returncode == 0:
        target = root / 'baseline' / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(value.stdout)
        previous.append(str(target))
for label, selected in [('baseline', previous), ('current', paths)]:
    subprocess.run([str(scanner), *selected, '--format', 'rust', '--max-size', '10mb',
                    '--reporters', 'json', '--output', str(root / label)], check=True)
print('Scoped Rust scanner baseline:', baseline)
SCANS
    cargo clippy --locked -p biomcp --test cases -- -D warnings
    cargo test --locked -p biomcp --test ratchets
    ;;
  *) echo 'usage: check_trial_results.sh [prepare|focused|schema|cheap|review BASE_COMMIT|scans BASE_COMMIT]' >&2; exit 2 ;;
esac
