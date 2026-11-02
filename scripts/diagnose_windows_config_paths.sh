#!/bin/sh
# M5 controls the existing shared VM only after capacity has been granted.
set -eu
cd "$(dirname "$0")/.."
export PATH="$HOME/.local/bin:$PATH"
python3.13 --version
python3.13 scripts/check_shell.py
python3.13 -m py_compile scripts/diagnose_windows_config_paths.py
. scripts/windows_vm_lock.sh
scratch=$(mktemp -d)
remote=
started=0
owned=0
saved=0
cleanup() {
    result=$?
    if [ "$started" -eq 1 ] && windows_vm_holds_lock; then
        if [ "$owned" -eq 1 ] && [ "$saved" -eq 1 ]; then
            ssh -o BatchMode=yes -o ConnectTimeout=15 gator 'ssh -o BatchMode=yes -o ConnectTimeout=15 -p 2222 Docker@localhost "powershell -NoProfile -Command \"Remove-Item -Recurse -Force -ErrorAction Stop C:\h\biomcp-0573; if (Test-Path C:\h\biomcp-0573) { exit 1 }\""' || result=1
        fi
    fi
    windows_vm_release "$started" || result=1
    if [ -n "$remote" ]; then
        ssh -o BatchMode=yes -o ConnectTimeout=15 gator "rm -rf '$remote'" || true
    fi
    rm -rf "$scratch"
    exit "$result"
}
windows_vm_traps
revision=$(git rev-parse HEAD)
short=$(printf '%.12s' "$revision")
git archive "$revision" > "$scratch/source.tar"
remote=$(ssh -o BatchMode=yes -o ConnectTimeout=15 gator 'mktemp -d /tmp/biomcp-0573.XXXXXX')
scp -o BatchMode=yes -o ConnectTimeout=15 "$scratch/source.tar" scripts/wait_windows_ssh.py "gator:$remote/"
windows_vm_claim
running=$(ssh -o BatchMode=yes -o ConnectTimeout=15 gator 'docker inspect --format "{{.State.Running}}" windows-2022')
if [ "$running" != false ]; then
    printf 'Windows VM is already running; leave its consumer untouched\n' >&2
    exit 1
fi
started=1
ssh -o BatchMode=yes -o ConnectTimeout=15 gator 'docker start windows-2022' >/dev/null
ssh -o BatchMode=yes -o ConnectTimeout=15 gator "python3 '$remote/wait_windows_ssh.py'"
ssh -o BatchMode=yes -o ConnectTimeout=15 gator 'ssh -o BatchMode=yes -o ConnectTimeout=15 -p 2222 Docker@localhost "powershell -NoProfile -Command \"if (Test-Path C:\h\biomcp-0573) { exit 1 }; New-Item -ItemType Directory C:\h\biomcp-0573\tmp | Out-Null\""'
owned=1
ssh -o BatchMode=yes -o ConnectTimeout=15 gator "scp -O -o BatchMode=yes -o ConnectTimeout=15 -P 2222 '$remote/source.tar' Docker@localhost:C:/h/biomcp-0573/source.tar"
ssh -o BatchMode=yes -o ConnectTimeout=15 gator 'ssh -o BatchMode=yes -o ConnectTimeout=15 -p 2222 Docker@localhost "tar -xf C:\h\biomcp-0573\source.tar -C C:\h\biomcp-0573"'
set +e
ssh -o BatchMode=yes -o ConnectTimeout=15 gator "ssh -o BatchMode=yes -o ConnectTimeout=15 -p 2222 Docker@localhost 'cmd /d /c C:\\h\\biomcp-0573\\scripts\\diagnose_windows_config_paths.cmd $revision'" > "$scratch/runtime.log" 2>&1
result=$?
set -e
cat "$scratch/runtime.log"
ssh -o BatchMode=yes -o ConnectTimeout=15 gator 'ssh -o BatchMode=yes -o ConnectTimeout=15 -p 2222 Docker@localhost "tar -czf C:\h\biomcp-0573\records.tar.gz -C C:\h\biomcp-0573 records"'
ssh -o BatchMode=yes -o ConnectTimeout=15 gator "scp -O -o BatchMode=yes -o ConnectTimeout=15 -P 2222 Docker@localhost:C:/h/biomcp-0573/records.tar.gz '$remote/records.tar.gz'"
scp -o BatchMode=yes -o ConnectTimeout=15 "gator:$remote/records.tar.gz" "$scratch/records.tar.gz"
(cd "$scratch" && shasum -a 256 source.tar records.tar.gz runtime.log > SHA256SUMS)
ssh -o BatchMode=yes -o ConnectTimeout=15 yellow "mkdir -p /home/ian/workspace/data/biomcp-redux-evidence/0573/$short"
scp -o BatchMode=yes -o ConnectTimeout=15 "$scratch/source.tar" "$scratch/records.tar.gz" "$scratch/runtime.log" "$scratch/SHA256SUMS" "yellow:/home/ian/workspace/data/biomcp-redux-evidence/0573/$short/"
saved=1
exit "$result"
