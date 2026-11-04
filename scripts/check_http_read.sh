#!/bin/sh
# Test only the runner's interrupted-read repair and the affected installed case.
set -eu
cd "$(dirname "$0")/.."
case "${1:-focused}" in
  routine)
    exec python3.13 -c 'import json,os; os.execlp("sh", "sh", "-c", json.load(open("sdlc/pm.json"))["ticketsGate"])' ;;
  focused)
    export RUSTUP_TOOLCHAIN="$(python3.13 release/toolchain.py get rust)"
    mkdir -p target/http-read
    cargo fmt --all -- --check
    cargo clippy --locked -p biomcp --test cases --test tools -- -D warnings
    cargo test --locked -p biomcp --test tools --no-run --message-format=json > target/http-read/tools.jsonl
    python3.13 - <<'PYTHON'
import json, subprocess
from pathlib import Path
rows = [json.loads(line) for line in Path('target/http-read/tools.jsonl').read_text().splitlines()]
runner, = [row['executable'] for row in rows if row.get('reason') == 'compiler-artifact' and row.get('target', {}).get('name') == 'tools' and row.get('executable')]
for proof in ['mcp-http/read-retries-only-interrupted', 'mcp-http/early-close-is-rejected']:
    subprocess.run([runner, '--proofs', proof, '--exact'], check=True)
PYTHON
    ;;
  case-only)
    [ "$#" -eq 6 ] || { echo 'case-only IMAGE RUNNER FIXTURES BINARY EXPECTED_SHA' >&2; exit 2; }
    shift
    python3.13 - "$@" <<'PYTHON'
import hashlib, json, os, subprocess, sys
from pathlib import Path
image, runner, fixtures, binary, expected = sys.argv[1:]
if Path(image).is_file():
    subprocess.run(['docker', 'load', '-i', image], check=True)
    image = 'biomcp-candidate:amd64'
elif '@sha256:' not in image:
    raise ValueError('case-only requires a pinned image or saved runtime image')
common = ['docker', 'run', '--rm', '--platform', 'linux/amd64', '--memory', '2g', '--cpus', '2']
if os.environ.get('JOB_LABEL_FILE'):
    common += ['--label-file', os.environ['JOB_LABEL_FILE']]
common += ['-v', str(Path(runner).resolve()) + ':/proof/case-runner:ro',
           '-v', str(Path(fixtures).resolve()) + ':/proof/fixtures:ro']
selected = '/usr/local/bin/biomcp'
if binary != '-':
    common += ['-v', str(Path(binary).resolve()) + ':/proof/biomcp:ro']
    selected = '/proof/biomcp'
print(json.dumps({'runner_sha256': hashlib.sha256(Path(runner).read_bytes()).hexdigest(),
                  'expected_binary_sha256': expected, 'image': image}), flush=True)
command = common + ['-e', 'BIOMCP_TEST_EXECUTABLE=' + selected,
                    '-e', 'BIOMCP_TEST_FIXTURES=/tmp/check/fixtures',
                    '-e', 'EXPECTED_BINARY_SHA256=' + expected,
                    '--entrypoint', '/bin/sh', image, '-ec',
                    'actual=$(sha256sum "$BIOMCP_TEST_EXECUTABLE"); '
                    'test "${actual%% *}" = "$EXPECTED_BINARY_SHA256"; '
                    'mkdir /tmp/check; cp /proof/case-runner /tmp/check/case-runner; '
                    'cp -R /proof/fixtures /tmp/check/fixtures; '
                    'exec /tmp/check/case-runner mcp-http/framing-resource-deadlines-and-shutdown '
                    '--exact --test-threads 1']
subprocess.run(command, check=True)
PYTHON
    ;;
  container)
    [ "$#" -eq 4 ] || { echo 'container ARTIFACTS CACHE REVISION' >&2; exit 2; }
    original_artifacts=$2
    cache=$3
    revision=$4
    python3.13 -c 'from pathlib import Path; import sys; from release.artifacts import candidate_record; print(candidate_record(Path(sys.argv[1]), sys.argv[2]))' "$original_artifacts" "$revision"
    artifacts="$PWD/target/http-read-linux/candidate"
    mkdir -p "$artifacts"
    cp "$original_artifacts"/*.whl "$original_artifacts/release.json" \
      "$original_artifacts/biomcp-linux-x86_64.tar.gz" \
      "$original_artifacts/biomcp-linux-x86_64.tar.gz.sha256" "$artifacts/"
    image=$(python3.13 release/toolchain.py image manylinux-x86_64)
    mkdir -p target/http-read-linux/build
    if [ "$cache" != - ]; then cp -a "$cache/." target/http-read-linux/build/; fi
    docker run --rm ${JOB_LABEL_FILE:+--label-file "$JOB_LABEL_FILE"} \
      --platform linux/amd64 --memory 8g --cpus 2 \
      -v "$PWD:/source" -v "$artifacts:/artifacts:ro" -w /source \
      -e HOME=/source/target/http-read-linux/home \
      -e CARGO_TARGET_DIR=/source/target/http-read-linux/build \
      -e CARGO_BUILD_JOBS=2 -e RUST_TEST_THREADS=2 \
      -e "BIOMCP_BUILD_REVISION=$revision" \
      "$image" /bin/bash scripts/linux_fixture_cache.sh /source/target/http-read-linux/evidence \
      /bin/bash scripts/check_http_read.sh inside
    ;;
  inside)
    python_series=$(python3 release/toolchain.py get python | tr -d .)
    export PATH="$HOME/.cargo/bin:/opt/python/cp$python_series-cp$python_series/bin:$PATH"
    mkdir -p "$HOME"
    python release/toolchain.py install-rust
    cargo test --locked -p biomcp --test cases --test tools --no-run --message-format=json > target/http-read-linux/runner.jsonl
    mkdir -p target/http-read-linux/installed
    tar -xzf /artifacts/biomcp-linux-x86_64.tar.gz -C target/http-read-linux/installed
    python - <<'PYTHON'
import hashlib, json, os, subprocess
from pathlib import Path
from release.artifacts import candidate_record
rows = [json.loads(line) for line in Path('target/http-read-linux/runner.jsonl').read_text().splitlines()]
executables = {row['target']['name']: row['executable'] for row in rows if row.get('reason') == 'compiler-artifact' and row.get('executable')}
runner = executables['cases']
for proof in ['mcp-http/read-retries-only-interrupted', 'mcp-http/early-close-is-rejected']:
    subprocess.run([executables['tools'], '--proofs', proof, '--exact'], check=True)
record = candidate_record(Path('/artifacts'), os.environ['BIOMCP_BUILD_REVISION'])
binary = Path('target/http-read-linux/installed/biomcp').resolve()
assert hashlib.sha256(binary.read_bytes()).hexdigest() == record['binary_sha256']
print(json.dumps({'archive_sha256': hashlib.sha256(Path('/artifacts/biomcp-linux-x86_64.tar.gz').read_bytes()).hexdigest(), 'candidate': record}), flush=True)
env = {**os.environ, 'BIOMCP_TEST_EXECUTABLE': str(Path('target/http-read-linux/installed/biomcp').resolve()),
       'BIOMCP_TEST_FIXTURES': str(Path('tests').resolve())}
subprocess.run([runner, 'mcp-http/framing-resource-deadlines-and-shutdown', '--exact', '--test-threads', '1'], env=env, check=True)
PYTHON
    ;;
  *) echo 'usage: check_http_read.sh [focused|routine|container ARTIFACTS CACHE REVISION|case-only IMAGE RUNNER FIXTURES BINARY EXPECTED_SHA]' >&2; exit 2 ;;
esac
