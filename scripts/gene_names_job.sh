#!/bin/sh
# Focused HGNC search cases; routine checks use make_goal.sh landing-gate.
set -eu
cd "$(dirname "$0")/.."
gate_python_bin=$(python3.13 -c 'from pathlib import Path; import sys; print(Path(sys.executable).resolve().parent)')
export PATH="$gate_python_bin:$PATH"
export RUSTUP_TOOLCHAIN=1.95.0
cargo build --locked -p biomcp --bin biomcp
cargo test --locked -p biomcp --test cases gene/names-
