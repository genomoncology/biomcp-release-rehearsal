#!/bin/sh
# pm's runner sets JOB_LABEL_FILE to <temporary root>/j<actual job ID>/container-labels.
windows_lock_owned=0
windows_lock_owner=
windows_vm_ssh() { ssh -o BatchMode=yes -o ConnectTimeout=15 gator "$@"; }
windows_vm_wait() { sleep 2; }
windows_vm_traps() {
    trap cleanup EXIT
    trap 'exit 129' HUP
    trap 'exit 130' INT
    trap 'exit 143' TERM
}
windows_vm_claim() {
    windows_expected_ticket=${1:-0573}
    case "$windows_expected_ticket" in [0-9][0-9][0-9][0-9]) ;; *) return 1 ;; esac
    windows_job_id=$(basename "$(dirname "${JOB_LABEL_FILE:-missing}")")
    case "${windows_job_id#j}" in ''|*[!0-9]*) return 1 ;; esac
    case "$windows_job_id" in j*) ;; *) return 1 ;; esac
    # Validate only our own label fields; never print or copy the control file.
    [ -f "${JOB_LABEL_FILE:-}" ] &&
        grep -qx 'pm.repo=biomcp-redux' "$JOB_LABEL_FILE" &&
        grep -qx "pm.ticket=$windows_expected_ticket" "$JOB_LABEL_FILE" || return 1
    windows_lock_owner="biomcp-redux $windows_job_id"
    while :; do
        if windows_vm_ssh "if mkdir ~/.windows-vm-lock 2>/dev/null; then
            if printf '%s\\n' '$windows_lock_owner' > ~/.windows-vm-lock/owner; then exit 0; fi
            rm -f ~/.windows-vm-lock/owner
            rmdir ~/.windows-vm-lock
            exit 2
        else exit 75; fi"; then
            windows_lock_owned=1
            return 0
        else
            windows_claim_result=$?
            [ "$windows_claim_result" -eq 75 ] || return "$windows_claim_result"
            printf 'Windows VM lock held; wait for its owner\n'
            windows_vm_wait
        fi
    done
}
windows_vm_holds_lock() {
    [ "$windows_lock_owned" -eq 1 ] &&
        windows_vm_ssh "test \"\$(cat ~/.windows-vm-lock/owner)\" = '$windows_lock_owner'"
}
windows_vm_release() {
    [ "$windows_lock_owned" -eq 1 ] || return 0
    # A changed owner never authorizes stopping its VM or releasing its lock.
    windows_vm_holds_lock || return 1
    windows_release_result=0
    if [ "$1" -eq 1 ]; then
        windows_vm_ssh 'docker stop windows-2022' >/dev/null || windows_release_result=1
        windows_stopped=$(windows_vm_ssh 'docker inspect --format "{{.State.Running}}" windows-2022') || windows_stopped=unknown
        printf 'Windows VM running after cleanup: %s\n' "$windows_stopped"
        [ "$windows_stopped" = false ] || windows_release_result=1
    fi
    windows_vm_ssh "test \"\$(cat ~/.windows-vm-lock/owner)\" = '$windows_lock_owner' && rm -r ~/.windows-vm-lock" || windows_release_result=1
    windows_lock_owned=0
    return "$windows_release_result"
}
