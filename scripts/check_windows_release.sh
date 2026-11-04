#!/bin/sh
# Run on the VM host in the current qualification's tracked pm job.
set -eu
cd "$(dirname "$0")/.."
[ "$#" -eq 3 ] || { echo 'Usage: check_windows_release.sh TICKET REVISION ARTIFACTS' >&2; exit 2; }
ticket=$1
revision=$2
artifacts=$3
case "$ticket" in [0-9][0-9][0-9][0-9]) ;; *) exit 2 ;; esac
case "$revision" in *[!0-9a-f]*|'') exit 2 ;; esac
[ "${#revision}" -eq 40 ] || exit 2
. scripts/windows_vm_lock.sh
# This wrapper runs on Gator, which already owns the VM and lock directory.
windows_vm_ssh() { sh -c "$1"; }
scratch=$(mktemp -d)
started=0
owned=0
remote=
cleanup() {
    result=$?
    if [ "$owned" -eq 1 ] && windows_vm_holds_lock; then
        ssh -o BatchMode=yes -p 2222 Docker@localhost "cmd /d /c rmdir /s /q $remote" || result=1
    fi
    windows_vm_release "$started" || result=1
    rm -rf "$scratch"
    exit "$result"
}
windows_vm_traps
windows_vm_claim "$ticket"
running=$(docker inspect --format '{{.State.Running}}' windows-2022)
[ "$running" = false ] || { echo 'Windows VM is already in use' >&2; exit 1; }
started=1
docker start windows-2022 >/dev/null
python3 scripts/wait_windows_ssh.py
remote="C:\\h\\biomcp-$ticket-$windows_job_id"
# Refuse a pre-existing folder; cleanup removes only one created by this job.
ssh -o BatchMode=yes -p 2222 Docker@localhost "cmd /d /c if exist $remote (exit /b 1) else (mkdir $remote)"
owned=1
git archive HEAD > "$scratch/source.tar"
tar -cf "$scratch/artifacts.tar" -C "$artifacts" .
scp -O -o BatchMode=yes -P 2222 "$scratch/source.tar" "$scratch/artifacts.tar" "Docker@localhost:C:/h/biomcp-$ticket-$windows_job_id/"
ssh -o BatchMode=yes -p 2222 Docker@localhost "cmd /d /c tar -xf $remote\\source.tar -C $remote && mkdir $remote\\candidate && tar -xf $remote\\artifacts.tar -C $remote\\candidate"
ssh -o BatchMode=yes -p 2222 Docker@localhost "cmd /d /c $remote\\scripts\\check_windows_release.cmd $remote $revision"
