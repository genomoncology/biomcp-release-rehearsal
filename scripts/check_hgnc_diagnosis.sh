#!/bin/sh
# Check the actual mixed transcript shapes and launcher syntax without a VM.
set -eu
cd "$(dirname "$0")/.."
export PATH="$HOME/.local/bin:$PATH"
python3.13 scripts/diagnose_hgnc_baseline.py --check-parser
sh -n scripts/diagnose_windows_hgnc.sh
sh -n scripts/diagnose_windows_hgnc_m5.sh
