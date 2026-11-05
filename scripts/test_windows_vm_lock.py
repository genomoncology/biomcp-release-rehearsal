"""Exercise the VM lock commands against a temporary home and fake Docker, never SSH."""
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
import unittest

HELPER = Path(__file__).with_name("windows_vm_lock.sh").resolve()


class WindowsVmLock(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        self.lock = self.root / ".windows-vm-lock"
        labels = self.root / "t/j987/container-labels"
        labels.parent.mkdir(parents=True)
        labels.write_text("pm.repo=biomcp-redux\npm.ticket=0573\n")
        binary = self.root / "bin"
        binary.mkdir()
        docker = binary / "docker"
        docker.write_text('#!/bin/sh\nprintf "%s\\n" "$*" >> "$MOCK_LOG"\n'
                          'case "$1" in stop) exit "${MOCK_STOP_CODE:-0}" ;; '
                          'inspect) printf "false\\n" ;; esac\n')
        docker.chmod(0o755)
        self.log = self.root / "commands"
        self.env = {**os.environ, "HOME": str(self.root), "JOB_LABEL_FILE": str(labels),
                    "MOCK_LOG": str(self.log), "PATH": str(binary) + os.pathsep + os.environ["PATH"]}

    def run_shell(self, body, **environment):
        script = f'. {shlex.quote(str(HELPER))}\n'
        script += 'windows_vm_ssh() { /bin/sh -c "$1"; }\n'
        return subprocess.run(["/bin/sh", "-eu", "-c", script + body],
                              env={**self.env, **environment}, capture_output=True, text=True)

    def test_failure_stops_owned_vm_before_releasing_lock(self):
        result = self.run_shell('''
started=0
trap 'result=$?; windows_vm_release "$started" || result=1; exit "$result"' EXIT
windows_vm_claim
test "$(cat ~/.windows-vm-lock/owner)" = 'biomcp-redux j987'
started=1
exit 9
''')
        self.assertEqual(result.returncode, 9, result.stderr)
        self.assertFalse(self.lock.exists())
        self.assertEqual(self.log.read_text().splitlines(),
                         ["stop windows-2022", "inspect --format {{.State.Running}} windows-2022"])

    def test_existing_owner_waits_until_release(self):
        self.lock.mkdir()
        (self.lock / "owner").write_text("thinkthen j321\n")
        result = self.run_shell('''
windows_vm_wait() { mv ~/.windows-vm-lock ~/.released-owner; }
windows_vm_claim
test "$(cat ~/.windows-vm-lock/owner)" = 'biomcp-redux j987'
windows_vm_release 0
''')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("lock held", result.stdout)
        self.assertEqual((self.root / ".released-owner/owner").read_text(), "thinkthen j321\n")
        self.assertFalse(self.lock.exists())
        self.assertFalse(self.log.exists())

    def test_foreign_owner_is_never_stopped_or_released(self):
        result = self.run_shell('''
windows_vm_claim
printf 'thinkthen j321\\n' > ~/.windows-vm-lock/owner
windows_vm_release 1
''')
        self.assertEqual(result.returncode, 1)
        self.assertEqual((self.lock / "owner").read_text(), "thinkthen j321\n")
        self.assertFalse(self.log.exists())

    def test_current_ticket_must_match_expected_ticket_before_claim(self):
        labels = Path(self.env['JOB_LABEL_FILE'])
        labels.write_text('pm.repo=biomcp-redux\npm.ticket=0293\n')
        result = self.run_shell('windows_vm_claim 0621\n')
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.lock.exists())
        result = self.run_shell('windows_vm_claim 0293\nwindows_vm_release 0\n')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(self.lock.exists())

    def test_missing_actual_job_id_fails_before_claim(self):
        result = self.run_shell("windows_vm_claim\n", JOB_LABEL_FILE="")
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.lock.exists())
        self.assertFalse(self.log.exists())

    def test_signals_exit_through_owned_cleanup(self):
        for signal, code in [("HUP", 129), ("INT", 130), ("TERM", 143)]:
            with self.subTest(signal=signal):
                self.log.unlink(missing_ok=True)
                result = self.run_shell("""
started=0
cleanup() {
    result=$?
    windows_vm_release "$started" || result=1
    exit "$result"
}
windows_vm_traps
windows_vm_claim
started=1
kill -s """ + signal + " $$\n")
                self.assertEqual(result.returncode, code, result.stderr)
                self.assertFalse(self.lock.exists())
                self.assertTrue(self.log.read_text().startswith("stop windows-2022\n"))

    def test_stop_failure_still_releases_own_lock_and_reports_failure(self):
        result = self.run_shell("windows_vm_claim\nwindows_vm_release 1\n", MOCK_STOP_CODE="1")
        self.assertEqual(result.returncode, 1)
        self.assertFalse(self.lock.exists())
        self.assertIn("stop windows-2022", self.log.read_text())


if __name__ == "__main__":
    unittest.main()
