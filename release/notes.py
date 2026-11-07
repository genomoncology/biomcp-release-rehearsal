"""Derive changed schema identities from the candidate and preserved migration history."""
import json
from pathlib import Path
import tarfile

ROOT = Path(__file__).resolve().parents[1]


def schema_changes():
    rows = []
    with tarfile.open(ROOT / 'release/migration-schema-baseline.tar.gz') as archive:
        for member in archive.getmembers():
            if not member.isfile() or not member.name.endswith('.json'):
                continue
            old = json.load(archive.extractfile(member))['$id']
            path = ROOT / 'crates/biomcp-types/schemas' / Path(member.name).name
            if path.exists():
                new = json.loads(path.read_text())['$id']
                if old != new:
                    rows.append(f'| `{old}` | `{new}` |')
    return ('## Changed schema identities\n\nCompared with the preserved pre-release migration schemas.\n\n'
            '| Earlier | This release |\n| --- | --- |\n' + '\n'.join(sorted(rows)))
