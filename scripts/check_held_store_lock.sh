#!/bin/sh
# Diagnose the installed macOS store-lock case after the ordinary cheap checks.
set -eu
cd "$(dirname "$0")/.."
export PATH="$HOME/.local/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.95.0
export CARGO_BUILD_JOBS=2
export RUST_TEST_THREADS=2
python3.13 --version
make check-cheap
cargo test --locked -p biomcp-fetch --lib cache::contention
cargo test --locked -p biomcp-fetch --test bundle \
    download::bundle_publication_finishes_while_another_owner_holds_the_reply_connection -- --exact
cargo test --locked -p biomcp-fetch --test bundle \
    download::replay_external_writer_stays_external_while_an_admission_owner_does_no_sql -- --exact
cargo test --locked -p biomcp-query --lib
cargo test --locked -p biomcp-query --test article_search
python3.13 - <<'PYTHON'
from pathlib import Path
import os
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, str(Path('release').resolve()))
from artifacts import cargo
from gate import stage_fixtures

binary = Path(cargo('dev')['executable'])
runner = Path(cargo('dev', tests='cases')['executable'])
root = Path(tempfile.mkdtemp(prefix='installed-store-lock-', dir='target')).resolve()
stage_fixtures(Path.cwd(), root / 'fixtures')
shutil.copy2(binary, root / 'biomcp')
shutil.copy2(runner, root / 'case-runner')
environment = {**os.environ, 'BIOMCP_TEST_EXECUTABLE': str(root / 'biomcp'),
               'BIOMCP_TEST_FIXTURES': str(root / 'fixtures')}
print('installed store-lock fixture:', root, flush=True)
result = subprocess.run([str(root / 'case-runner'), '--exact',
                         'pathway-wikipathways/held-store-lock-names-local-failure',
                         '--test-threads', '2'], cwd=root, env=environment, check=False)
if result.returncode == 0:
    shutil.rmtree(root)
raise SystemExit(result.returncode)
PYTHON
