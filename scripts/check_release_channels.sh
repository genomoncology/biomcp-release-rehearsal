#!/bin/sh
# Focused proof-tooling checks; no compilation or real qualification.
set -eu
cd "$(dirname "$0")/.."
case "${1:-focused}" in
  routine)
    exec python3.13 -c 'import json,os; os.execlp("sh", "sh", "-c", json.load(open("sdlc/pm.json"))["ticketsGate"])' ;;
  focused) ;;
  *) exit 2 ;;
esac
ruff check --select F401,F841,ERA,BLE,E722,S110 release/published.py release/smoke.py release/artifacts.py release/test_installer.py release/test_promotion.py scripts/test_windows_vm_lock.py
BIOMCP_TEST_PINNED_INSTALLER=1 BIOMCP_TEST_CLAUDE_INSTALL=1 python3.13 -m unittest release.test_installer release.test_promotion release.test_workflow release.test_windows
python3.13 scripts/test_windows_vm_lock.py
python3.13 scripts/check_shell.py
