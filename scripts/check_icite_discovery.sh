#!/bin/sh
# Run source-owned citation discovery checks before the routine gate.
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
case "${1:-focused}" in
  focused)
    make fmt lint ratchets
    cargo test --locked -p biomcp --test tools -- --proofs live-source-checks/retained-drift-witnesses
    cargo test --locked -p biomcp --test cases -- article/citations/
    ;;
  routine) make landing-gate ;;
  *) echo 'usage: check_icite_discovery.sh [focused|routine]' >&2; exit 2 ;;
esac
