"""At most fifty rounds of the lasting eight-process installed first-open case."""
import argparse
import hashlib
import json
from pathlib import Path
import sqlite3
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from release.smoke import cache_first_opens


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    parser.add_argument('binary_sha256')
    parser.add_argument('output', type=Path)
    parser.add_argument('--artifact', required=True)
    parser.add_argument('--source', default='015aea0a704b7b6176b6988bc6fae707ac0feb38')
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    actual = hashlib.sha256(binary.read_bytes()).hexdigest()
    if actual != args.binary_sha256:
        raise SystemExit('Installed binary digest mismatch; no cases started')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    (output / 'binding.json').write_text(json.dumps({
        'candidate': args.source, 'binary': str(binary), 'binary_sha256': actual,
        'artifact': args.artifact, 'command': ['health', '--offline'],
        'processes_per_round': 8, 'maximum_rounds': 50,
        'timing_limit': 'Native dispatch duration only; successful schema/WAL phase timing is unobservable',
    }, indent=2) + '\n')
    failed = False
    for round_index in range(50):
        root = output / f'round-{round_index:02}'
        record = cache_first_opens(binary, root)
        record['round'] = round_index
        databases = []
        # All CLI processes have exited. These reads cannot affect first-open locking.
        for path in sorted((root / 'cache').rglob('*.sqlite')):
            state = {'path': str(path.relative_to(root))}
            try:
                with sqlite3.connect(path.as_uri() + '?mode=ro', uri=True) as connection:
                    state.update({
                        'journal_mode': connection.execute('PRAGMA journal_mode').fetchone()[0],
                        'owners': connection.execute('SELECT owner,version FROM meta ORDER BY owner').fetchall(),
                        'tables': [row[0] for row in connection.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")],
                    })
            except sqlite3.DatabaseError as error:
                state['post_exit_read_error'] = str(error)
            databases.append(state)
        record['post_exit_databases'] = databases
        (root / 'summary.json').write_text(json.dumps(record, indent=2) + '\n')
        print(json.dumps(record), flush=True)
        if not record['empty_cache_before_release'] or not databases or record['exits'] != [0] * 8:
            durations = [json.loads((root / f'{worker:02}-exec.json').read_text())['duration_ms']
                         for worker in range(8)]
            print(f'round {round_index} duration_ms {durations}', flush=True)
            for worker, status in enumerate(record['exits']):
                if status:
                    for stream in ('stdout', 'stderr'):
                        text = (root / f'{worker:02}.{stream}').read_text(errors='replace')[:600]
                        print(f'worker {worker} exit {status} {stream}: {text}', flush=True)
            failed = True
            break
    raise SystemExit(bool(failed))


if __name__ == '__main__':
    main()
