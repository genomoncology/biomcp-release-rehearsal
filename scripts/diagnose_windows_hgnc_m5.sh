#!/bin/sh
# M5 controls the existing Gator VM; all Windows execution remains on that VM.
set -eu
cd "$(dirname "$0")/.."
scratch=$(mktemp -d)
remote=
cleanup() {
    result=$?
    ssh -o BatchMode=yes -o ConnectTimeout=15 gator 'docker stop windows-2022' >/dev/null || true
    if [ -n "$remote" ]; then
        ssh -o BatchMode=yes -o ConnectTimeout=15 gator "rm -rf '$remote'" || true
    fi
    rm -rf "$scratch"
    exit "$result"
}
trap cleanup EXIT
revision=$(git rev-parse HEAD)
short=$(printf '%.12s' "$revision")
ssh -o BatchMode=yes -o ConnectTimeout=15 yellow true
remote=$(ssh -o BatchMode=yes -o ConnectTimeout=15 gator 'mktemp -d /tmp/biomcp-0566-controller.XXXXXX')
ssh -o BatchMode=yes -o ConnectTimeout=15 gator "mkdir '$remote/scripts'"
scp -o BatchMode=yes -o ConnectTimeout=15 scripts/diagnose_windows_hgnc.sh scripts/diagnose_hgnc_baseline.py scripts/wait_windows_ssh.py "gator:$remote/scripts/"
printf '%s\n' 'Controller: M5; remote existing Windows VM on Gator: 2 CPU, 8 GiB; no compilation.'
set +e
ssh -o BatchMode=yes -o ConnectTimeout=15 gator "sh '$remote/scripts/diagnose_windows_hgnc.sh' '$revision' local"
result=$?
set -e
if ! ssh -o BatchMode=yes -o ConnectTimeout=15 gator "test -f '$remote/windows-bounded-$short.tar.gz'"; then
    [ "$result" -ne 0 ] && exit "$result"
    exit 1
fi
scp -o BatchMode=yes -o ConnectTimeout=15 "gator:$remote/windows-bounded-$short.tar.gz" "$scratch/records.tar.gz"
scp -o BatchMode=yes -o ConnectTimeout=15 "$scratch/records.tar.gz" "yellow:/home/ian/workspace/data/biomcp-redux-evidence/0566/windows-bounded-$short.tar.gz"
exit "$result"
