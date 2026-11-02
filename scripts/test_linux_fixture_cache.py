"""A failed Linux check retains its native SQLite fixture and exit status."""
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class Retention(unittest.TestCase):
    def test_failed_check_retains_native_sqlite_state_and_status(self):
        with tempfile.TemporaryDirectory() as scratch:
            retained = Path(scratch) / 'retained'
            result = subprocess.run(
                ['sh', str(ROOT / 'scripts/linux_fixture_cache.sh'), str(retained),
                 sys.executable, '-c', '''
import os,pathlib,sqlite3
root=pathlib.Path(os.environ['BIOMCP_EVIDENCE_DIR'])
print(root,flush=True)
fixture=root/'fixture-failures'/'failed-case'
fixture.mkdir(parents=True)
db=sqlite3.connect(fixture/'cache.sqlite')
db.execute('PRAGMA journal_mode=WAL')
db.execute('CREATE TABLE marker(value TEXT)')
db.execute("INSERT INTO marker VALUES('retained after failure')")
db.commit()
(fixture/'failure.txt').write_text('fixture command failed')
os._exit(17)
'''], capture_output=True, text=True, check=False,
            )
            self.assertEqual(result.returncode, 17, result.stderr)
            native = Path(result.stdout.strip())
            self.assertEqual(native.parent, Path('/var/tmp'))
            self.assertFalse(native.exists())
            self.assertTrue((retained / 'failed-case/cache.sqlite-wal').is_file())
            self.assertTrue((retained / 'failed-case/cache.sqlite-shm').is_file())
            with sqlite3.connect((retained / 'failed-case/cache.sqlite').as_uri() + '?mode=ro', uri=True) as db:
                self.assertEqual(db.execute('SELECT value FROM marker').fetchone(), ('retained after failure',))
            self.assertEqual((retained / 'failed-case/failure.txt').read_text(), 'fixture command failed')
            self.assertIn('Retained failed fixtures:', result.stderr)


if __name__ == '__main__':
    unittest.main()
