#!/bin/sh
set -eu
case "${1:-focused}" in
cheap) make fmt lint ratchets ;;
focused)
    cargo test --locked -p biomcp-fetch --lib
    cargo test --locked -p biomcp-fetch --test transport
    cargo test --locked -p biomcp-fetch --test grpc_worker
    ;;
*) exit 2 ;;
esac
