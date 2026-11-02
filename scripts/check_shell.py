"""Parse tracked POSIX shell scripts and keep focused prechecks complete."""
from pathlib import Path
import re
import shlex
import subprocess
import sys


def main():
    paths = subprocess.check_output(['git', 'ls-files', '-z']).split(b'\0')
    recipe = subprocess.check_output(
        ['make', '--no-print-directory', '-n', 'check-cheap'], text=True).splitlines()[0]
    cheap = {word for word in shlex.split(recipe)[1:]
             if not word.startswith('-') and '=' not in word}
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
        if not result.returncode and path.parent == Path('scripts') and path.match('check_*.sh'):
            for line, command in enumerate(path.read_text().splitlines(), 1):
                match = re.match(r'^\s*(?:[\w|-]+\)\s*)?make\s+([^;]+)', command)
                if match:
                    goals = set(shlex.split(match[1], comments=True))
                    if goals & cheap and 'check-cheap' not in goals:
                        print(f'{path}:{line}: use make check-cheap instead of partial prechecks',
                              file=sys.stderr)
                        status = 1
    return status


if __name__ == '__main__':
    raise SystemExit(main())
