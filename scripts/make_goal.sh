#!/bin/sh
# Run one Make goal with the configured ticket gate's tool setup.
set -eu
cd "$(dirname "$0")/.."
if [ "$#" -ne 1 ]; then
    echo 'usage: make_goal.sh GOAL' >&2
    exit 2
fi
exec python3 - "$1" <<'PYTHON'
import json
import os
import shlex
import sys

goal = sys.argv[1]
if not goal or goal.startswith("-") or "=" in goal:
    raise SystemExit("make_goal.sh: expected one Make goal, not an option or assignment")
with open("sdlc/pm.json") as config:
    command = json.load(config).get("ticketsGate")
marker = "make landing-gate"
if not isinstance(command, str) or command.count(marker) != 1 or not command.endswith(marker):
    raise SystemExit("make_goal.sh: unsupported ticketsGate; expected one trailing 'make landing-gate'")
os.execlp("sh", "sh", "-c", command[:-len("landing-gate")] + shlex.quote(goal))
PYTHON
