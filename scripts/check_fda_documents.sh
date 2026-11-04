#!/bin/sh
# Build the native FDA document/search changes once, then run exact installed cases.
set -eu
cd "$(dirname "$0")/.."
case "${1:-cases}" in
  prepare)
    make schema-generate
    make docs-generate
    exec make check-cheap ;;
  routine) exec sh scripts/check_nightly_tests.sh routine ;;
  scans) exec python3 scripts/scan_fda_documents.py "${2:?reviewed baseline commit required}" ;;
  live)
    make plain-cli-build
    exec python3 scripts/check_fda_document_live.py ;;
  cases)
    make schema-generate
    make docs-generate
    mkdir -p target
    cargo test --locked -p biomcp --test cases --no-run --message-format=json > target/fda-documents-build.jsonl
    fda_case_runner=$(python3 -c 'import json; rows=[json.loads(line) for line in open("target/fda-documents-build.jsonl")]; print(next(row["executable"] for row in rows if row.get("reason")=="compiler-artifact" and row.get("executable") and row["target"]["name"]=="cases"))')
    fda_product_binary="${CARGO_TARGET_DIR:-target}/debug/biomcp"
    python3 scripts/trials.py --target cases --runner "$fda_case_runner" --executable "$fda_product_binary" --list > target/fda-documents-trials.txt
    python3 scripts/trials.py --target cases --runner "$fda_case_runner" --executable "$fda_product_binary" --installed device-approval-search/diagnostic/device-approvals.yaml fda-document-text/diagnostic/fda-documents.yaml fda-document-text/diagnostic/pdf-over-limit.yaml fda-document-text/diagnostic/pdf-scanned-no-text.yaml diagnostic-pma/diagnostic/pma.yaml diagnostic-cdx/diagnostic/help.yaml
    ;;
  *) echo 'usage: check_fda_documents.sh [prepare|cases|routine|live|scans BASELINE]' >&2; exit 2 ;;
esac
