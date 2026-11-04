#!/bin/sh
# Keep SQLite WAL fixtures on the container's filesystem, then retain failures.
set -eu
retained=$1
shift
fixture_root=$(mktemp -d /var/tmp/biomcp-fixtures.XXXXXX)
finish() {
    status=$?
    trap - EXIT
    if [ "$status" -ne 0 ] && [ -d "$fixture_root/fixture-failures" ]; then
        if mkdir -p "$retained" && cp -a "$fixture_root/fixture-failures/." "$retained/"; then
            echo "Retained failed fixtures: $retained" >&2
        else
            echo "Failed to retain fixtures in $retained" >&2
            status=1
        fi
    fi
    rm -rf "$fixture_root"
    exit "$status"
}
trap finish EXIT
# Docker Desktop host binds lost committed rows in the loaded WAL process test;
# the same binary passed on native storage (cache process deadline diagnosis).
export BIOMCP_EVIDENCE_DIR="$fixture_root"
"$@"
