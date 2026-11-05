#!/bin/sh
# One focused native diagnosis: six sequential fresh-cache cases, no CPU burners.
set -eu
cd "$(dirname "$0")/.."
export CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CACHE_OPEN_DIAGNOSTICS=1
mkdir -p target/cache-first-open
cargo test --locked -p biomcp-fetch --test cache --no-run --message-format=json > target/cache-first-open/build.json
python3 - <<'PY'
import json
import os
from pathlib import Path
import subprocess
import time

root = Path('target/cache-first-open')
executable = next(
    row['executable'] for line in (root / 'build.json').read_text().splitlines()
    if (row := json.loads(line)).get('reason') == 'compiler-artifact'
    and row['target']['name'] == 'cache' and row.get('executable')
)
failures = 0
for index in range(6):
    started = time.monotonic()
    load = os.getloadavg()
    with (root / f'{index:02}.log').open('wb') as log:
        try:
            result = subprocess.run(
                [executable, '--exact', 'cache_process::concurrent_first_opens_and_distinct_writes_survive_into_a_new_process', '--nocapture'],
                stdout=log, stderr=subprocess.STDOUT, timeout=90,
            )
            status = result.returncode
        except subprocess.TimeoutExpired:
            status = 'watchdog'
    record = {'case': index, 'status': status, 'elapsed_s': time.monotonic() - started,
              'host_load_start': load, 'parallel_cases': 1, 'cpu_burners': 0}
    with (root / 'runs.jsonl').open('a') as summary:
        summary.write(json.dumps(record) + '\n')
    print(json.dumps(record), flush=True)
    failures += status != 0
raise SystemExit(bool(failures))
PY
