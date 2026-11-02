#!/bin/sh
# Check installer transport with local TLS fixtures, without a release build.
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
export PATH="$HOME/.local/bin:$PATH"
python3.13 --version
case "${1:-check}" in
    red) python3.13 -m unittest release.test_installer.Installer.test_fixture_ca_reaches_curl_when_backend_ignores_bundle_environment ;;
    check)
        make check-cheap
        ruff check --select F401,F841,ERA,BLE,E722,S110 release/gate.py release/test_installer.py
        python3.13 -m unittest release.test_installer release.test_gate
        ;;
    *) exit 2 ;;
esac
