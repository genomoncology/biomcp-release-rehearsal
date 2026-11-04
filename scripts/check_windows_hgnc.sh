#!/bin/sh
# Run inside a tracked Gator job while this lane has Windows VM custody.
set -eu
cd "$(dirname "$0")/.."
archive=$(mktemp)
cleanup() {
    result=$?
    rm -f "$archive"
    docker stop windows-2022 >/dev/null || true
    exit "$result"
}
trap cleanup EXIT
docker start windows-2022 >/dev/null
# Wait for SSH boot readiness without changing the VM or authentication.
ready=0
for attempt in $(seq 1 30); do
    if ssh -o BatchMode=yes -o ConnectTimeout=5 -p 2222 Docker@localhost 'cmd /c ver' >/dev/null 2>&1; then
        ready=1
        break
    fi
    sleep 2
done
[ "$ready" -eq 1 ]
revision=$(git rev-parse HEAD)
git archive "$revision" > "$archive"
# Remove only this lane's obsolete full-debug build; the lean target is reused.
ssh -o BatchMode=yes -o ConnectTimeout=15 -p 2222 Docker@localhost 'powershell -NoProfile -Command "if (Test-Path C:\h\biomcp-0566-target) { Remove-Item -Recurse -Force C:\h\biomcp-0566-target }"'
ssh -o BatchMode=yes -o ConnectTimeout=15 -p 2222 Docker@localhost 'powershell -NoProfile -Command "New-Item -ItemType Directory -Force C:\h\biomcp-0566 | Out-Null"'
scp -O -o BatchMode=yes -o ConnectTimeout=15 -P 2222 "$archive" Docker@localhost:C:/h/biomcp-0566/source.tar
ssh -o BatchMode=yes -o ConnectTimeout=15 -p 2222 Docker@localhost 'tar -xf C:\h\biomcp-0566\source.tar -C C:\h\biomcp-0566'
ssh -o BatchMode=yes -o ConnectTimeout=15 -p 2222 Docker@localhost "cmd /d /c C:\\h\\biomcp-0566\\scripts\\check_windows_hgnc.cmd $revision"
