#!/bin/sh
# Check generated source pairs through registry coverage and installed discovery.
set -eu
python="$(uv python find --offline 3.13.17)"
export PATH="$(dirname "$python"):$HOME/.cargo/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.95.0
case "${1:-focused}" in
    routine) exec sh scripts/check_nightly_tests.sh routine ;;
    focused)
        mkdir -p target/front-map-pairs
        make check-cheap
        cargo test --locked -p biomcp --test ratchets -- --list > target/front-map-pairs/trials.txt
        cargo test --locked -p biomcp --test ratchets front_map::tests::
        python3 scripts/check_front_map.py "${CARGO_TARGET_DIR:-target}/debug/biomcp" --revision "$(git rev-parse HEAD)" --output target/front-map-pairs/installed
        cargo test --locked -p biomcp --test cases -- --list > target/front-map-pairs/case-trials.txt
        export BIOMCP_TRANSCRIPT_DIR="$PWD/target/front-map-pairs/affected-transcripts"
        export BIOMCP_EVIDENCE_DIR="$PWD/target/front-map-pairs/affected-fixtures"
        sh scripts/check_study_rows.sh focused trial-results/posted-values-and-event-pages trial-results/no-posted-results study-rows/study/rows.yaml
        ;;
    scans)
        python3 - "$2" <<'PY'
import json
from pathlib import Path
import subprocess
import sys
root = Path('target/front-map-pairs')
baseline = root / 'baseline-source'
baseline.mkdir(parents=True, exist_ok=True)
paths = subprocess.check_output(['git', 'diff', '--name-only', sys.argv[1], 'HEAD', '--', 'crates'], text=True).splitlines()
paths = [path for path in paths if path.endswith('.rs')]
archive = subprocess.check_output(['git', 'archive', sys.argv[1], *paths])
subprocess.run(['tar', '-xf', '-', '-C', str(baseline)], input=archive, check=True)
scanner = next(p.parent.parent / '.bin/jscpd' for p in Path.home().glob('.npm/_npx/*/node_modules/jscpd/package.json') if json.loads(p.read_text()).get('version') == '4.0.5')
for label, prefix in [('baseline', baseline), ('current', Path('.'))]:
    subprocess.run([str(scanner), *[str(prefix / path) for path in paths], '--format', 'rust', '--max-size', '10mb', '--reporters', 'json', '--output', str(root / ('jscpd-' + label))], check=True)
PY
        ;;
    *) echo 'usage: front_map_pairs_job.sh [focused|scans BASELINE|routine]' >&2; exit 2 ;;
esac
