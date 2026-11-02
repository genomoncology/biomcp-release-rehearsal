"""Validate the reviewed public fixture digests; keep all key bytes out of Git."""
import json
import hashlib
from itertools import islice
from pathlib import Path
import re

POLICY = Path(__file__).with_name('public-fixtures.json')
UPSTREAM = 'https://raw.githubusercontent.com/gnutls/gnutls/3.7.9/lib/crypto-selftests-pk.c'


def reviewed():
    policy = json.loads(POLICY.read_text())
    if (set(policy) != {'upstream', 'tag', 'file', 'fixtures'} or policy['upstream'] != UPSTREAM
            or policy['tag'] != '3.7.9' or policy['file'] != 'lib/crypto-selftests-pk.c'
            or len(policy['fixtures']) != 10):
        raise ValueError('invalid reviewed public fixture policy')
    symbols, digests = set(), set()
    for row in policy['fixtures']:
        if (set(row) != {'symbol', 'sha256'} or not re.fullmatch('[a-zA-Z0-9_]+', row['symbol'])
                or row['symbol'] in symbols or not re.fullmatch('[a-f0-9]{64}', row['sha256'])
                or row['sha256'] in digests):
            raise ValueError('invalid reviewed public fixture entry')
        symbols.add(row['symbol'])
        digests.add(row['sha256'])
    return digests


# The pinned Gitleaks 8.30.1 private-key match, used only to validate that
# coordinates recover a complete match rather than neighboring source bytes.
PRIVATE_KEY = re.compile(rb'(?i)-----BEGIN[ A-Z0-9_-]{0,100}PRIVATE KEY(?: BLOCK)?-----[\s\S-]{64,}?KEY(?: BLOCK)?-----')


def recognized(path, coordinates, digests):
    first, last, column, end = coordinates
    if first < 1 or last < first or column < 1 or end < 1:
        return False
    with path.open('rb') as source:
        lines = list(islice(source, first - 1, last))
    if len(lines) != last - first + 1:
        return False
    candidates = set()
    # Gitleaks byte columns use the previous newline's index. The first line
    # of a scanner fragment instead starts at zero. Both exact conventions
    # are checked; ambiguous spans remain fatal, never partially recognized.
    for start in {column - 1, column - 2}:
        for stop in {end, end - 1}:
            if start < 0 or stop < 0:
                continue
            if first == last:
                value = lines[0][start:stop]
            else:
                value = lines[0][start:] + b''.join(lines[1:-1]) + lines[-1][:stop]
            if PRIVATE_KEY.fullmatch(value):
                candidates.add(value)
    return (len(candidates) == 1
            and hashlib.sha256(next(iter(candidates))).hexdigest() in digests)
