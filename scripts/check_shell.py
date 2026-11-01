"""Parse every tracked POSIX shell script with the platform's native shell."""
from pathlib import Path
import subprocess


def main():
    paths = subprocess.check_output(['git', 'ls-files', '-z']).split(b'\0')
    status = 0
    for raw in paths:
        if not raw:
            continue
        path = Path(raw.decode('utf-8'))
        if not path.is_file():
            continue
        with path.open('rb') as stream:
            if stream.readline().rstrip(b'\n') != b'#!/bin/sh':
                continue
        result = subprocess.run(['/bin/sh', '-n', str(path)])
        status = status or result.returncode
    return status


if __name__ == '__main__':
    raise SystemExit(main())
