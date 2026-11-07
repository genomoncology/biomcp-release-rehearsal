#!/bin/sh
# Run the configured ticket gate for launchers that hand off to it.
set -eu
cd "$(dirname "$0")/.."
case "${1:-routine}" in
  routine)
    exec python3 -c 'import json, os; os.execlp("sh", "sh", "-c", json.load(open("sdlc/pm.json"))["ticketsGate"])' ;;
  *) echo 'usage: check_nightly_tests.sh [routine]' >&2; exit 2 ;;
esac
