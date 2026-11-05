"""Carry a cmd payload through Windows OpenSSH's default PowerShell unchanged."""
import base64
import sys


def remote_command(payload):
    # ProcessStartInfo avoids PowerShell's native-argument quote rewriting. cmd's
    # /s removes just the outer quotes; quotes around individual paths survive.
    arguments = ('/d /s /c "' + payload + '"').replace("'", "''")
    script = (
        "$info = New-Object System.Diagnostics.ProcessStartInfo; "
        "$info.FileName = 'cmd.exe'; "
        f"$info.Arguments = '{arguments}'; "
        "$info.UseShellExecute = $false; "
        "$process = [System.Diagnostics.Process]::Start($info); "
        "$process.WaitForExit(); exit $process.ExitCode"
    )
    encoded = base64.b64encode(script.encode('utf-16le')).decode('ascii')
    # The default outer PowerShell otherwise maps any native failure to 1.
    return ('powershell.exe -NoProfile -NonInteractive -EncodedCommand ' + encoded
            + '; exit $LASTEXITCODE')


if __name__ == '__main__':
    print(remote_command(sys.argv[1]))
