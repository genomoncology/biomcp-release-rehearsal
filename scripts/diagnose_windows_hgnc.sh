#!/bin/sh
# One no-compile job, retaining only named output records before releasing the VM.
set -eu
cd "$(dirname "$0")/.."
scratch=$(mktemp -d)
cleanup() {
    result=$?
    rm -rf "$scratch"
    docker stop windows-2022 >/dev/null || true
    exit "$result"
}
trap cleanup EXIT
revision=${1:-$(git rev-parse HEAD)}
destination=${2:-yellow}
if [ "$destination" = yellow ]; then
    ssh -o BatchMode=yes -o ConnectTimeout=15 yellow true
fi
docker start windows-2022 >/dev/null
python3 scripts/wait_windows_ssh.py
scp -O -o BatchMode=yes -o ConnectTimeout=15 -P 2222 scripts/diagnose_hgnc_baseline.py Docker@localhost:C:/h/biomcp-0566/diagnose_hgnc_baseline.py
set +e
ssh -o BatchMode=yes -o ConnectTimeout=15 -p 2222 Docker@localhost "C:\\Users\\Docker\\AppData\\Roaming\\uv\\python\\cpython-3.13.13-windows-x86_64-none\\python.exe C:\\h\\biomcp-0566\\diagnose_hgnc_baseline.py $revision" > "$scratch/runtime.log" 2>&1
result=$?
set -e
cat "$scratch/runtime.log"
short=$(printf '%.12s' "$revision")
if ! rg -q '^HGNC_RECORDS_READY ' "$scratch/runtime.log"; then
    [ "$result" -ne 0 ] && exit "$result"
    exit 1
fi
scp -O -o BatchMode=yes -o ConnectTimeout=15 -P 2222 "Docker@localhost:C:/h/biomcp-0566/hgnc-bounded-output-$short.tar.gz" "$scratch/records.tar.gz"
if [ "$destination" = local ]; then
    cp "$scratch/records.tar.gz" "windows-bounded-$short.tar.gz"
else
    scp -o BatchMode=yes -o ConnectTimeout=15 "$scratch/records.tar.gz" "yellow:/home/ian/workspace/data/biomcp-redux-evidence/0566/windows-bounded-$short.tar.gz"
fi
exit "$result"
