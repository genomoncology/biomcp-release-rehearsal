#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
export PATH="$HOME/.local/bin:$PATH"
python3.13 --version
case "${1:-focused}" in
  red)
    python3.13 -m unittest release.test_artifacts.ReleaseCommands.test_build_keeps_notice_bytes_with_windows_text_newlines
    ;;
  focused)
    ruff check --select F401,F841,ERA,BLE,E722,S110 release/notices.py release/test_artifacts.py
    make check-cheap
    python3.13 -m unittest release.test_artifacts.ReleaseCommands release.test_licenses
    ;;
  *) exit 2 ;;
esac
