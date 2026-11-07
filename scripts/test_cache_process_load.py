"""The load runner ends on the first failure and names the SQLite step."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

SOURCE = Path(__file__).with_name('cache_process_load.py')
spec = importlib.util.spec_from_file_location('cache_process_load', SOURCE)
load = importlib.util.module_from_spec(spec)
spec.loader.exec_module(load)


class FailedRun(unittest.TestCase):
    def run_check(self, script, rounds=3):
        with tempfile.TemporaryDirectory(dir=load.ROOT / 'target' if (load.ROOT / 'target').exists() else load.ROOT) as folder, \
                mock.patch.object(load, 'OUTPUT', Path(folder)), \
                contextlib.redirect_stdout(io.StringIO()) as output:
            argv = [sys.executable, '-c', script]
            passed = load.run_check('python/case', argv, rounds, {}, None)
            logs = len(list(Path(folder).rglob('*.log')))
        return passed, logs, output.getvalue()

    def test_first_failure_is_not_retried_and_names_its_step(self):
        # The product writes these keys in this order (j34178).
        record = json.dumps({'extended_code': 5, 'operation': 'fetch::cache.rs:125', 'primary_code': 5})
        script = f'import os,sys; open(os.environ["BIOMCP_SQLITE_DIAGNOSTICS"],"a").write({record!r}); sys.exit(1)'
        passed, logs, output = self.run_check(script)
        self.assertFalse(passed)
        self.assertEqual(logs, 1)
        self.assertIn('sqlite operation=fetch::cache.rs:125 code=5 extended=5', output)

    def test_every_round_runs_when_all_pass(self):
        passed, logs, _ = self.run_check('pass')
        self.assertTrue(passed)
        self.assertEqual(logs, 3)

    def test_load_follows_booked_cores(self):
        with mock.patch.object(load.subprocess, 'Popen') as start, \
                mock.patch.object(load.os, 'sched_setaffinity', create=True) as pin, \
                mock.patch.object(load.os, 'sched_getaffinity', create=True, return_value={0, 1, 2, 3, 4, 5}):
            burners, pinned = load.start_load(2.0)
        self.assertEqual(len(burners), 3)
        self.assertEqual(start.call_count, 3)
        self.assertEqual(pinned, [0, 1])
        pin.assert_called_once_with(0, [0, 1])

    def test_only_refuses_an_unknown_name(self):
        with self.assertRaises(SystemExit), contextlib.redirect_stderr(io.StringIO()):
            load.main(['--only', 'no-such-check'])


if __name__ == '__main__':
    unittest.main()
