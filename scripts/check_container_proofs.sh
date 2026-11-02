#!/bin/sh
# Check the declared runtime proof selection without rebuilding release artifacts.
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
export PATH="$HOME/.local/bin:$PATH"
cargo fmt --all -- --check
ruff check --select F401,F841,ERA,BLE,E722,S110 release/container_check.py release/smoke.py release/test_linux.py
python3.13 scripts/check_shell.py
python3.13 scripts/check_docs.py
if command -v cargo-shear >/dev/null 2>&1; then cargo shear; else echo 'cargo-shear unavailable'; fi
if command -v jscpd >/dev/null 2>&1; then
  jscpd --reporters console crates/biomcp/tests/runner/proof_requirements.rs release/container_check.py release/smoke.py release/test_linux.py
else
  echo 'jscpd unavailable'
fi
python3.13 -m unittest release.test_linux release.test_checks release.test_gate
cargo test --locked -p biomcp --test tools -- --proofs schema-discovery/repository-python-proof-exclusions --exact
