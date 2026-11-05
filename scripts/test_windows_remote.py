"""Check Windows command transport; opt into real default-PowerShell SSH proof."""
import base64
import os
from pathlib import Path
import subprocess
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
from windows_remote import remote_command


class WindowsRemote(unittest.TestCase):
    def test_transport_encodes_payload_and_propagates_native_exit(self):
        payload = 'if exist "C:\\h\\a (b)" (exit /b 1) else (mkdir "C:\\h\\a (b)")'
        command = remote_command(payload)
        prefix = 'powershell.exe -NoProfile -NonInteractive -EncodedCommand '
        self.assertTrue(command.startswith(prefix))
        encoded, suffix = command[len(prefix):].split(';', 1)
        self.assertEqual(suffix, ' exit $LASTEXITCODE')
        script = base64.b64decode(encoded, validate=True).decode('utf-16le')
        self.assertIn("/d /s /c \"" + payload + '\"', script)
        self.assertIn('exit $process.ExitCode', script)

    @unittest.skipUnless(os.environ.get('WINDOWS_REMOTE_TEST_HOST'), 'requires owned Windows VM')
    def test_default_powershell_preserves_commands_and_exit_codes(self):
        host = os.environ['WINDOWS_REMOTE_TEST_HOST']
        path = r'C:\h\remote proof (quoting)-' + os.environ['WINDOWS_REMOTE_TEST_JOB']

        def run(payload):
            return subprocess.run(['ssh', '-o', 'BatchMode=yes', '-p', '2222', host,
                                   remote_command(payload)], capture_output=True, text=True)

        create = f'if exist "{path}" (exit /b 19) else (mkdir "{path}")'
        result = run(create)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        try:
            result = run(create)
            self.assertEqual(result.returncode, 19, result.stdout + result.stderr)
            result = run(f'cd /d "{path}" && echo proof>marker && type marker')
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(result.stdout.strip(), 'proof')
            self.assertEqual(run('exit /b 23').returncode, 23)
        finally:
            result = run(f'rmdir /s /q "{path}"')
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(run(f'if exist "{path}" (exit /b 1)').returncode, 0)


if __name__ == '__main__':
    unittest.main()
