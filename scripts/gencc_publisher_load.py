"""Repeat both first-publisher cases under CPU contention; retain phase output."""
import json
import math
import os
from pathlib import Path
import subprocess
import sys

ROUNDS = 10
LOAD_PER_CORE = 1.5


def main():
    executable = next(row['executable'] for line in Path(sys.argv[1]).read_text().splitlines()
                      if (row := json.loads(line)).get('reason') == 'compiler-artifact'
                      and row['target']['name'] == 'cases' and row.get('executable'))
    root = Path('target/gencc-publisher-load')
    root.mkdir(parents=True, exist_ok=True)
    cores = float(os.environ.get('PM_JOB_CORES', os.cpu_count() or 1))
    count = math.ceil(cores * LOAD_PER_CORE)
    print(json.dumps({'rounds': ROUNDS, 'burners': count, 'cores': cores,
                      'host_load': os.getloadavg()}), flush=True)
    burners = [subprocess.Popen([sys.executable, '-c', 'while True: pass']) for _ in range(count)]
    try:
        for index in range(ROUNDS):
            path = root / f'{index:02}.log'
            with path.open('wb') as output:
                result = subprocess.run([executable, 'gene-sections/gencc-first-publisher',
                                         '--test-threads', '2', '--nocapture'],
                                        stdout=output, stderr=subprocess.STDOUT)
            print(f'round={index + 1}/{ROUNDS} exit={result.returncode} log={path}', flush=True)
            print(path.read_text(), flush=True)
            if result.returncode:
                return result.returncode
    finally:
        for child in burners:
            child.terminate()
        for child in burners:
            child.wait()
    return 0


if __name__ == '__main__':
    sys.exit(main())
