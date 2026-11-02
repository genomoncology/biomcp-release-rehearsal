#!/bin/sh
# Check portable case names without contacting providers.
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
python3.13 --version
make check-cheap
cargo test --locked -p biomcp --test tools -- --proofs live-source-checks/offline-eligibility --exact
cargo test --locked -p biomcp --test tools -- --proofs schema-discovery/selects-and-lists-cases --exact
