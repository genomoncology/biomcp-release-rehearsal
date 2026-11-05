#!/bin/sh
# Check the isolated hosted recovery without building product binaries.
set -eu
cd "$(dirname "$0")/.."
export PATH="$HOME/.local/bin:$PATH"
if [ "${1:-}" = metadata ]; then
  sh -n "$0"
  python3.13 -c 'import json; from pathlib import Path; config=json.loads(Path("sdlc/pm.json").read_text()); assert "mailroom" not in config; print("sdlc/pm.json: valid JSON; unused mailroom entry absent")'
  exit 0
fi
python3.13 --version
actionlint -version
ruff --version
python3.13 -m unittest release.test_section release.test_linux release.test_gate release.test_source_revision release.test_workflow_contracts
python3.13 scripts/check_shell.py
actionlint
ruff check --no-cache --select F401,F841,ERA,BLE,E722,S110 release/section.py release/test_section.py
uvx --offline --from vulture==2.14 vulture --version
uvx --offline --from vulture==2.14 vulture release/section.py --min-confidence 60
npm exec --offline --yes --package=jscpd@4.0.5 -- jscpd --version
npm exec --offline --yes --package=jscpd@4.0.5 -- jscpd --format python --reporters console,json \
  --output target/0642-jscpd release/section.py release/test_section.py
