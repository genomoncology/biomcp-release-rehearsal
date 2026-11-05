"""Outside-in checks for source binding and owned smoke-process cleanup."""
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import threading
import unittest
from unittest.mock import patch

from scripts.installed_smoke_section import bind_fixtures, cleanup, observe


class InstalledSmokeTooling(unittest.TestCase):
    def test_modified_runtime_fixture_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'tests').mkdir()
            fixture = root / 'tests/case.yaml'
            fixture.write_text('original')
            subprocess.run(['git', 'init', '--quiet', str(root)], check=True)
            subprocess.run(['git', '-C', str(root), 'add', 'tests'], check=True)
            subprocess.run(['git', '-C', str(root), '-c', 'user.name=Fixture', '-c',
                            'user.email=fixture@example.invalid', 'commit', '--quiet', '-m', 'Fixture'], check=True)
            fixture.write_text('modified')
            previous = Path.cwd()
            try:
                os.chdir(root)
                with self.assertRaisesRegex(ValueError, 'runtime fixture differs from candidate'):
                    bind_fixtures('HEAD', root / 'proof', root)
            finally:
                os.chdir(previous)

    def test_observer_error_is_recorded_and_stops_owned_child(self):
        with tempfile.TemporaryDirectory() as directory:
            child = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(30)'], start_new_session=True)
            errors = []
            try:
                with patch('scripts.installed_smoke_section.identities', side_effect=PermissionError):
                    observe(child, threading.Event(), errors, Path(directory) / 'processes.jsonl')
                self.assertNotEqual(child.wait(timeout=5), 0)
                self.assertEqual(errors, ['PermissionError'])
            finally:
                cleanup(child.pid)
                child.wait(timeout=5)

    def test_cleanup_kills_term_resistant_owned_descendant(self):
        script = """
import os, signal, subprocess, sys
signal.signal(signal.SIGTERM, signal.SIG_IGN)
child = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(30)'])
print(child.pid, flush=True)
child.wait()
"""
        child = subprocess.Popen([sys.executable, '-c', script], stdout=subprocess.PIPE,
                                 text=True, start_new_session=True)
        try:
            descendant = int(child.stdout.readline())
            cleanup(child.pid)
            child.stdout.read()  # EOF waits for every inherited stdout owner to exit.
            self.assertEqual(child.wait(timeout=5), -signal.SIGKILL)
            status = Path('/proc') / str(descendant) / 'status'
            try:
                state = next(line for line in status.read_text().splitlines() if line.startswith('State:'))
            except FileNotFoundError:
                pass
            else:
                self.assertIn(state.split()[1], {'X', 'Z'})
        finally:
            cleanup(child.pid)
            child.wait(timeout=5)
            child.stdout.close()


if __name__ == '__main__':
    unittest.main(defaultTest=InstalledSmokeTooling.__name__)
