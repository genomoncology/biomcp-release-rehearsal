"""Compare unfiltered drug and variant replies with the retained public rehearsal executable."""
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from release.smoke import environment


def invoke(binary, words):
    with tempfile.TemporaryDirectory(prefix='civic-evidence-compatibility-') as directory:
        root = Path(directory)
        installed = root / 'biomcp'
        shutil.copy2(binary, installed)
        return subprocess.run([str(installed), *words], cwd=root, env=environment(root),
                              capture_output=True, timeout=60)


def main():
    baseline, candidate = (Path(value).resolve(strict=True) for value in sys.argv[1:])
    commands = [
        ['get', 'drug', '5', 'civic', '--source', 'civic', '--civic-kind', 'evidence',
         '--section-limit', '2', '--replay', str(ROOT / 'tests/replies/drug/civic-evidence-streams')],
        ['get', 'variant', 'CA123643', 'civic', '--disable-sources', 'myvariant',
         '--replay', str(ROOT / 'tests/replies/variant/native-civic')],
    ]
    for words in commands:
        before, after = (invoke(binary, words) for binary in (baseline, candidate))
        if before.returncode or after.returncode or before.stdout != after.stdout:
            raise ValueError('unfiltered ' + words[1] + ' output differs from retained baseline: '
                             + before.stdout.decode()[:400] + ' / ' + after.stdout.decode()[:400])
        print('Unfiltered ' + words[1] + ' stdout byte-identical: ' + str(len(after.stdout)) + ' bytes')
    refused = invoke(baseline, commands[0] + ['--evidence-level', 'A'])
    if refused.returncode != 2 or b'evidence-level' not in refused.stdout:
        raise ValueError('baseline did not reject the previously unsupported evidence-level filter')
    print('Baseline red witness: evidence-level refused before source access')


if __name__ == '__main__':
    main()
