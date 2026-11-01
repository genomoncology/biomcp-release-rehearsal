#!/bin/sh
# Exercise the release notice graph and schema proof without a release build.
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
export PATH="$HOME/.local/bin:$PATH"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}"
case "${1:-focused}" in
  diagnose)
    cargo --version
    python3.13 --version
    for crate in borrow-or-share@0.2.4 foldhash@0.2.0; do
      cargo tree --locked --workspace --target all -i "$crate"
      cargo tree --locked --workspace --target all --edges normal,build -i "$crate"
    done
    cargo deny --version
    cargo deny --locked check licenses
    ;;
  policy)
    ruff --version
    ruff check --select F401,F841,ERA,BLE,E722,S110 release/notices.py release/test_licenses.py scripts/test_schema_release.py
    python3.13 - <<'PYTHON'
import json
import subprocess

for workspace in ([], ['--workspace']):
    command = ['cargo', 'deny', '--locked', *workspace]
    result = subprocess.run(command + ['list', '--format', 'json', '--layout', 'crate'],
                            capture_output=True, text=True, check=True)
    data = json.loads(result.stdout)
    print('cargo-deny workspace:', bool(workspace), 'crate inventory:', flush=True)
    suspects = {name: entry for name, entry in data.items()
                if name.split()[0] in ('borrow-or-share', 'foldhash', 'jsonschema')}
    print('total:', len(data), 'suspects:', json.dumps(suspects), flush=True)
    result = subprocess.run(command + ['check', 'licenses'], check=False)
    print('licenses exit:', result.returncode, flush=True)
PYTHON
    ;;
  fixture)
    python3.13 -m unittest release.test_licenses.Licensing.test_generation_excludes_dependencies_that_do_not_ship
    ;;
  notices)
    python3.13 - <<'PYTHON'
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
from release.notices import configuration

with tempfile.TemporaryDirectory(prefix='biomcp-notices-check-') as folder:
    root = Path(folder)
    settings = root / 'about.toml'
    settings.write_text(configuration())
    for target in ('x86_64-unknown-linux-gnu', 'aarch64-unknown-linux-gnu',
                   'x86_64-apple-darwin', 'aarch64-apple-darwin', 'x86_64-pc-windows-msvc'):
        output = root / (target + '.json')
        subprocess.run(['cargo', 'about', 'generate', '--locked', '--workspace', '--fail',
                        '--target', target, '--config', str(settings), '--format', 'json',
                        '--output-file', str(output)], check=True)
        data = json.loads(output.read_text())
        names = {item['crate']['name'] for license in data['licenses'] for item in license['used_by']}
        assert names and not names.intersection({'borrow-or-share', 'foldhash', 'jsonschema'}), names
        print(target, len(names), 'shipping crates', hashlib.sha256(output.read_bytes()).hexdigest(), flush=True)
PYTHON
    cargo deny --locked check licenses
    ;;
  schema-reuse)
    cargo --version
    python3.13 --version
    ruff check --select F401,F841,ERA,BLE,E722,S110 scripts/test_schema_release.py
    mkdir -p "$CARGO_TARGET_DIR"
    du -sk "$CARGO_TARGET_DIR"
    df -k . "${TMPDIR:-/tmp}"
    trap 'df -k . "${TMPDIR:-/tmp}"; du -sk "$CARGO_TARGET_DIR"' EXIT
    python3.13 scripts/test_schema_release.py
    cargo test --locked -p biomcp-types --features schema --test schemas
    ;;
  focused)
    python3.13 -m unittest release.test_licenses release.test_publication
    cargo test --locked -p biomcp-types --features schema --test schemas checked_in_schemas_match_types -- --exact
    du -sk "$CARGO_TARGET_DIR"
    df -k . "${TMPDIR:-/tmp}"
    trap 'df -k . "${TMPDIR:-/tmp}"; du -sk "$CARGO_TARGET_DIR"' EXIT
    python3.13 scripts/test_schema_release.py
    ;;
  *) exit 2 ;;
esac
