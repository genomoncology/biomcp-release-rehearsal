#!/bin/sh
# Instrument cases, but report only the product binary and its child profiles.
set -eu
cd "$(dirname "$0")/.."
root=$(pwd)
ignore='(^|/)(tests|target|generated)/|(^|/)build\.rs$|_tests\.rs$'
threshold=$(awk '
    NR != 1 || $0 !~ /^[0-9]+([.][0-9]+)?$/ || $0 + 0 > 100 { bad = 1 }
    END {
        if (NR != 1 || bad) exit 1
        floor = $0 - 0.2
        printf "%.4f\n", (floor > 0 ? floor : 0)
    }
' coverage.floor) || { echo 'coverage: invalid coverage.floor' >&2; exit 1; }

coverage_env=$(cargo llvm-cov show-env --sh)
eval "$coverage_env"
profile_dir=${LLVM_PROFILE_FILE%/*}
case "$profile_dir" in
    /*) ;;
    *) echo 'coverage: expected an absolute profile directory' >&2; exit 1 ;;
esac
binary="$CARGO_LLVM_COV_TARGET_DIR/debug/biomcp"
llvm_tools="$(rustc --print target-libdir)/../bin"
profdata="$profile_dir/bin.profdata"
complete="$profile_dir/coverage-complete"
summary=$(mktemp)
trap 'rm -f "$summary"' EXIT HUP INT TERM

# Bind reusable profiles to source inputs, including dirty and untracked files.
inputs() {
    python3 - <<'PYINPUTS'
import hashlib
from pathlib import Path
import subprocess

names = subprocess.check_output([
    'git', 'ls-files', '-z', '--cached', '--others', '--exclude-standard', '--',
    'crates', 'tests', 'skills', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml',
    'scripts/coverage.sh', 'sdlc/features.md',
]).split(b'\0')
digest = hashlib.sha256()
for name in sorted(set(names) - {b''}):
    path = Path(name.decode())
    digest.update(name + b'\0')
    digest.update(path.read_bytes() if path.is_file() else b'<missing>')
    digest.update(b'\0')
print(digest.hexdigest())
PYINPUTS
}

selected_source() {
    mode=$1
    # Explicit source selection excludes dependencies. Unlinked library functions
    # have no coverage mapping in this object and never enter the denominator.
    set --
    for crate in "$root"/crates/*; do
        [ ! -d "$crate/src" ] || set -- "$@" "$crate/src"
    done
    python3 - "$mode" "$llvm_tools/llvm-cov" "$binary" "$profdata" "$ignore" "$@" <<'PYREPORT'
from pathlib import Path
import subprocess
import sys

mode, tool, binary, profile, ignore, *directories = sys.argv[1:]
sources = sorted(str(path) for directory in directories for path in Path(directory).rglob('*.rs'))
options = ['--show-line-counts-or-regions'] if mode == 'show' else []
subprocess.run([tool, mode, '--object', binary, '--instr-profile', profile,
                '--ignore-filename-regex', ignore, *options, '--sources', *sources], check=True)
PYREPORT
}

report() {
    selected_source report >"$summary"
    awk '/^TOTAL/ { print "coverage (cases; product binary source): " $0 }' "$summary"
    awk -v threshold="$threshold" '
        /^TOTAL/ { found = 1; value = $10 + 0; if (value < threshold) bad = 1 }
        END {
            if (!found) { print "coverage: missing line totals" > "/dev/stderr"; exit 1 }
            if (bad) { print "coverage: below floor tolerance " threshold > "/dev/stderr"; exit 1 }
        }
    ' "$summary"
}

case "${1:-run}" in
    run)
        cargo llvm-cov clean --workspace
        mkdir -p "$profile_dir"
        rm -f "$complete"
        collected_inputs=$(inputs)
        cargo test --locked -p biomcp --test cases --test ratchets --no-run
        cargo build --locked -p biomcp --bin biomcp
        if ! cargo test --locked -p biomcp --test ratchets; then
            echo 'coverage: structure ratchets failed before case collection' >&2
            exit 1
        fi
        # Keep harness profiles available for diagnosis, but never merge them.
        LLVM_PROFILE_FILE="$profile_dir/harness-%m-%p.profraw"
        export LLVM_PROFILE_FILE
        set --
        if [ -n "${RUST_TEST_THREADS:-}" ]; then
            set -- -- --test-threads "$RUST_TEST_THREADS"
        fi
        if ! cargo test --locked -p biomcp --test cases "$@"; then
            echo 'coverage: cases collection incomplete because its suite failed' >&2
            exit 1
        fi
        python3 - "$llvm_tools/llvm-profdata" "$profile_dir" "$profdata" <<'PYMERGE'
from pathlib import Path
import subprocess
import sys

tool, directory, output = sys.argv[1:]
profiles = sorted(str(path) for path in Path(directory).glob('bin-*.profraw'))
if not profiles:
    raise SystemExit('coverage: cases produced no product binary profiles')
subprocess.run([tool, 'merge', '-sparse', '-o', output, '--input-files=-'],
               input='\n'.join(profiles) + '\n', text=True, check=True)
PYMERGE
        if [ "$(inputs)" != "$collected_inputs" ]; then
            echo 'coverage: inputs changed during collection; run make coverage again' >&2
            exit 1
        fi
        printf '%s\n' "$collected_inputs" >"$complete"
        report
        ;;
    report|unreached)
        if [ ! -f "$complete" ]; then
            echo 'coverage: no complete collection; run make coverage first' >&2
            exit 1
        fi
        if [ "$(cat "$complete")" != "$(inputs)" ]; then
            echo 'coverage: inputs changed since collection; run make coverage again' >&2
            exit 1
        fi
        if [ "$1" = unreached ]; then
            selected_source show
        else
            report
        fi
        ;;
    *) echo 'usage: coverage.sh [run|report|unreached]' >&2; exit 2 ;;
esac
