"""Fixed-count CPU-loaded cache-process runs, preserving each child's phase output."""
import concurrent.futures
import json
import os
from pathlib import Path
import subprocess
import sys
import time


def main():
    executable = next(
        row['executable']
        for line in Path(sys.argv[1]).read_text().splitlines()
        if (row := json.loads(line)).get('reason') == 'compiler-artifact'
        and row['target']['name'] == 'cache' and row.get('executable')
    )
    output = Path('target/cache-process-load/results')
    output.mkdir(parents=True, exist_ok=True)
    print(f'load: 2 CPU quota, 2 CPU burners, 4 concurrent cases, 3 waves; host load {os.getloadavg()}', flush=True)
    burners = [subprocess.Popen([sys.executable, '-c', 'while True: pass']) for _ in range(2)]
    failures = 0
    try:
        def run(index):
            started = time.monotonic()
            result = subprocess.run(
                [executable, '--exact', 'cache_process::concurrent_first_opens_and_distinct_writes_survive_into_a_new_process', '--nocapture'],
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=90,
            )
            path = output / f'{index:02}.log'
            path.write_bytes(result.stdout)
            print(f'case {index}: exit={result.returncode} elapsed={time.monotonic()-started:.3f}s log={path}', flush=True)
            print(result.stdout.decode(errors='replace'), flush=True)
            return result.returncode != 0
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            for wave in range(3):
                failures += sum(pool.map(run, range(wave * 4, wave * 4 + 4)))
    finally:
        for process in burners:
            process.terminate()
        for process in burners:
            process.wait()
    print(f'fixed-count results: {12-failures}/12 passed', flush=True)
    return bool(failures)


if __name__ == '__main__':
    sys.exit(main())
