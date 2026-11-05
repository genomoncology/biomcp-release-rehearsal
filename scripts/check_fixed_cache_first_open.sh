#!/bin/sh
# One focused cache fix proof, then a source-bound installed native wheel campaign.
set -eu
cd "$(dirname "$0")/.."
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
export CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 RUSTUP_TOOLCHAIN=1.95.0
root="$PWD/target/fixed-cache-first-open"
mkdir -p "$root"
case "${1:-focused}" in
focused)
make check-cheap
uvx --offline --from ruff==0.17.2 ruff check --select F401,F841,ERA,BLE,E722,S110 \
    release/smoke.py release/test_checks.py release/test_smoke.py release/test_gate.py scripts/cache_installed_first_open.py
uvx --offline --from vulture==2.14 vulture release/gate.py release/smoke.py scripts/cache_installed_first_open.py --min-confidence 60
python3.13 -m unittest release.test_checks release.test_gate
cargo test --locked -p biomcp-fetch --lib
cargo test --locked -p biomcp-fetch --test cache
cargo build --locked -p biomcp --bin biomcp --test cases --test tools --message-format=json > "$root/debug-build.json"
python3.13 - "$root" <<'PY'
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
root = Path(sys.argv[1])
executables = {row['target']['name']: row['executable'] for line in (root / 'debug-build.json').read_text().splitlines()
               if (row := json.loads(line)).get('reason') == 'compiler-artifact' and row.get('executable')}
installed = root / 'python-installed'
installed.mkdir()
for source, target in [('biomcp', 'biomcp'), ('cases', 'case-runner'), ('tools', 'tool-runner')]:
    shutil.copy2(executables[source], installed / target)
env = dict(os.environ, BIOMCP_INSTALLED_BINARY=str(installed / 'biomcp'),
           BIOMCP_CASE_RUNNER=str(installed / 'case-runner'), BIOMCP_INSTALLED_VERSION='1.0.0')
subprocess.run([sys.executable, '-m', 'unittest',
                'release.test_smoke.InstalledTemporarySpace.test_isolated_installed_commands_and_cleanup'], env=env, check=True)
PY
;;
remaining) ;; # Carry unchanged green inputs; leave the broad cleanup failure red.
*) exit 2 ;;
esac
python3.13 - "$root" <<'PY'
import json
from pathlib import Path
import subprocess
import sys
root = Path(sys.argv[1])
scanner = next((p.parent.parent / '.bin/jscpd' for p in Path.home().glob('.npm/_npx/*/node_modules/jscpd/package.json')
                if json.loads(p.read_text()).get('version') == '4.0.5'), None)
if scanner is None:
    raise SystemExit('cached jscpd 4.0.5 unavailable')
subprocess.run([str(scanner), '--version'], check=True)
product = ['crates/biomcp-fetch/src/cache.rs', 'crates/biomcp-fetch/src/cache']
tests = ['crates/biomcp-fetch/tests/support/cache_process.rs']
python_product = ['release/smoke.py']
python_tests = ['release/test_checks.py', 'release/test_smoke.py', 'release/test_gate.py']
baseline = root / 'baseline'
baseline.mkdir()
archive = subprocess.check_output(['git', 'archive', '015aea0a704b7b6176b6988bc6fae707ac0feb38', *product, *tests, *python_product, *python_tests])
subprocess.run(['tar', '-xf', '-', '-C', str(baseline)], input=archive, check=True)
for kind, language, paths in [('rust-product', 'rust', product), ('rust-tests', 'rust', tests),
                               ('python-product', 'python', python_product), ('python-tests', 'python', python_tests)]:
    for side, prefix in [('baseline', baseline), ('fixed', Path('.'))]:
        selected = [str(prefix / path) for path in paths]
        if side == 'fixed' and kind == 'python-product':
            selected.append('scripts/cache_installed_first_open.py')
        subprocess.run([str(scanner), *selected, '--format', language, '--max-size', '10mb',
                        '--reporters', 'json', '--output', str(root / f'jscpd-{side}-{kind}')], check=True)
PY
source=$(git rev-parse HEAD)
python3.13 release/artifacts.py build --revision "$source" --output "$root/artifact" --profile release
python3.13 -m venv "$root/installed"
"$root/installed/bin/python" -m pip install --no-index --no-deps "$root"/artifact/*.whl
binary_sha=$(python3.13 -c 'import json,sys; print(json.load(open(sys.argv[1]))["binary_sha256"])' "$root/artifact/release.json")
python3.13 scripts/cache_installed_first_open.py "$root/installed/bin/biomcp" "$binary_sha" \
    "$root/results" --source "$source" --artifact "$root/artifact/release.json"
