#!/bin/sh
# Focused transport regression; the VM mode books no product build or gate.
set -eu
cd "$(dirname "$0")/.."
case "${1:-local}" in
    local)
        sh scripts/make_goal.sh shell-check
        uvx --offline --from ruff==0.17.2 ruff check --select F401,F841,ERA,BLE,E722,S110 \
            scripts/windows_remote.py scripts/test_windows_remote.py
        uvx --offline --from vulture==2.14 vulture scripts/windows_remote.py --min-confidence 60
        python3 scripts/test_windows_vm_lock.py
        python3 scripts/test_windows_remote.py
        ;;
    vm)
        . scripts/windows_vm_lock.sh
        windows_vm_ssh() { sh -c "$1"; }
        started=0
        cleanup() {
            result=$?
            windows_vm_release "$started" || result=1
            exit "$result"
        }
        windows_vm_traps
        windows_vm_claim 0634
        running=$(docker inspect --format '{{.State.Running}}' windows-2022)
        [ "$running" = false ] || { echo 'Windows VM is already in use' >&2; exit 1; }
        started=1
        docker start windows-2022 >/dev/null
        python3 scripts/wait_windows_ssh.py
        WINDOWS_REMOTE_TEST_HOST=Docker@localhost WINDOWS_REMOTE_TEST_JOB="$windows_job_id" \
            python3 scripts/test_windows_remote.py -v
        ;;
    *) echo 'Usage: check_windows_remote.sh [local|vm]' >&2; exit 2 ;;
esac
