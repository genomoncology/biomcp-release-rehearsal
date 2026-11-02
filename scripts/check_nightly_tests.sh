#!/bin/sh
# Check the offline nightly wrapper and its source-owned discovery fixtures.
set -eu
cd "$(dirname "$0")/.."
cargo fmt --all -- --check
ruff check --select F401,F841,ERA,BLE,E722,S110 scripts/nightly_live_check.py scripts/nightly_recommendation_cases.py scripts/test_nightly_live_check.py
python3 scripts/check_docs.py
python3 scripts/test_nightly_live_check.py
python3 -m unittest scripts.test_nightly_live_check.NightlyCommand.test_complete_recommendation_replacements_follow_exact_discovery
