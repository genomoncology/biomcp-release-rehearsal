#!/bin/sh
# Check the isolated hosted recovery without building product binaries.
set -eu
cd "$(dirname "$0")/.."
export PATH="$HOME/.local/bin:$PATH"
python3.13 -m unittest release.test_section release.test_linux release.test_gate release.test_source_revision release.test_workflow_contracts
python3.13 scripts/check_shell.py
actionlint
ruff check --no-cache --select F401,F841,ERA,BLE,E722,S110 release/section.py release/test_section.py
