"""Public package descriptions and staged release notes preserve migration guidance."""
from email.parser import Parser
from pathlib import Path
import json
import shutil
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile

from release.artifacts import wheel
from release.readme import description
from release.test_licenses import write_payloads
from release.workflow import release_notes


class PublicDocs(unittest.TestCase):
    def test_built_wheel_carries_readme_and_absolute_links(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_payloads(root)
            binary = root / 'biomcp'
            binary.write_bytes(b'candidate')
            artifact = wheel(root, binary, {'version': '1.0.0', 'platform': 'linux_x86_64',
                                            'distribution': 'biomcp-cli'})
            with zipfile.ZipFile(artifact) as archive:
                metadata = Parser().parsestr(archive.read('biomcp_cli-1.0.0.dist-info/METADATA').decode())
            self.assertEqual(metadata['Description-Content-Type'], 'text/markdown')
            self.assertIn(description(), metadata.get_payload())
            self.assertIn('# BioMCP\n', metadata.get_payload())
            self.assertNotIn('](docs/', metadata.get_payload())
            self.assertNotIn('](sdlc/', metadata.get_payload())
            self.assertIn('https://github.com/genomoncology/biomcp/blob/main/docs/recipes.md', metadata.get_payload())

    def test_staged_notes_include_migration_and_candidate_schema_ids(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source_root = Path(__file__).resolve().parents[1]
            source = root / 'source'
            schemas = source / 'crates/biomcp-types/schemas'
            shutil.copytree(source_root / 'crates/biomcp-types/schemas', schemas)
            (source / 'release').mkdir()
            for name in ('schema-baseline.tar.gz', 'migration-schema-baseline.tar.gz'):
                shutil.copyfile(source_root / 'release' / name, source / 'release' / name)
            with patch('release.notes.ROOT', source):
                release_notes(root, '1.0.0', 'a' * 40)
                before_refresh = (root / 'RELEASE-NOTES.md').read_text()
                with tarfile.open(source / 'release/schema-baseline.tar.gz', 'w:gz') as archive:
                    for schema in sorted(schemas.glob('*.json')):
                        archive.add(schema, arcname=schema.name)
                release_notes(root, '1.0.0', 'a' * 40)
            notes = (root / 'RELEASE-NOTES.md').read_text()
            self.assertEqual(notes, before_refresh)
            for name in ('adverse-event-faers-card', 'adverse-event-maude-card',
                         'adverse-event-drugs', 'adverse-event-reactions',
                         'adverse-event-devices', 'adverse-event-narrative',
                         'adverse-event-identifiers'):
                current_id = json.loads((schemas / (name + '.json')).read_text())['$id']
                self.assertIn(' | `' + current_id + '` |', notes)
            self.assertIn(Path('release/notes.md').read_text(), notes)
            self.assertIn('FHIR patient reading is planned for 1.1', notes)
            self.assertIn('## Changed schema identities', notes)
            self.assertIn('biomcp.adverse-event-faers-card/', notes)
            self.assertIn('gh attestation verify ARTIFACT', notes)
