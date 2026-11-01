#!/bin/sh
# Reuse the existing NCI cases and transcript comparator for this shared flow.
set -eu
case "${1:-focused}" in
qualify)
    make fmt lint ratchets
    sh "$0" focused
    sh "$0" cases
    sh "$0" diagnostic "$2" "$3"
    ;;
focused)
    cargo test --locked -p biomcp --test cases organization
    cargo test --locked -p biomcp --test cases biomarker
    ;;
cases)
    sh scripts/check_source_snapshots.sh cases
    ;;
diagnostic)
    reference=$(cd "$(dirname "$2")" && pwd)/$(basename "$2")
    destination=$3
    mkdir -p "$destination/reference" "$destination/current" "${TMPDIR:?}/nci-reference-profiles"
    destination=$(cd "$destination" && pwd)
    cp "$(dirname "$reference")/source-job.json" "$destination/reference-source.json"
    export LLVM_PROFILE_FILE="$TMPDIR/nci-reference-profiles/reference-%m-%p.profraw"
    python3 - "$reference" "$destination" <<'PY'
import hashlib, json, os, pathlib, subprocess, sys
reference, destination = map(pathlib.Path, sys.argv[1:])
source = json.loads((destination / 'reference-source.json').read_text())
assert hashlib.sha256(reference.read_bytes()).hexdigest() == source['sha256']
versions = subprocess.check_output([str(reference), 'version'], text=True)
(destination / 'reference-version.json').write_text(versions)
(destination / 'diagnostic-source.json').write_text(json.dumps({
    'source': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
    'reference_profile': 'coverage-instrumented parent CLI from the retained gate',
    'current_profile': 'default Cargo dev profile without coverage instrumentation',
    'profile_output': os.environ['LLVM_PROFILE_FILE'],
    'families': ['organization', 'biomarker'],
}, indent=2) + '\n')
PY
    for family in organization biomarker; do
        BIOMCP_TEST_EXECUTABLE="$reference" BIOMCP_TRANSCRIPT_DIR="$destination/reference" \
            cargo test --locked -p biomcp --test cases "$family"
        BIOMCP_TRANSCRIPT_DIR="$destination/current" \
            cargo test --locked -p biomcp --test cases "$family"
    done
    python3 scripts/reply_diff.py "$destination/reference" "$destination/current"
    ;;
*) echo 'expected focused, cases, diagnostic or qualify REFERENCE DESTINATION' >&2; exit 2 ;;
esac
