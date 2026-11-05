"""Exercise source and artifact custody at the recovery command boundary."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from release import section


class SectionCustody(unittest.TestCase):
    def checkout(self, root, text):
        root.mkdir()
        subprocess.run(['git', 'init', '-q', str(root)], check=True)
        for name in ('Cargo.lock', 'rust-toolchain.toml', 'release/toolchain.json', 'source'):
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text if name == 'source' else 'pin')
        subprocess.run(['git', '-C', str(root), 'add', '.'], check=True)
        subprocess.run(['git', '-C', str(root), '-c', 'user.name=Fixture',
                        '-c', 'user.email=fixture@example.org', 'commit', '-qm', 'Create source'], check=True)
        return section.git(root, 'rev-parse', 'HEAD')

    def test_product_and_tooling_are_separate_and_artifacts_keep_their_hashes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            product, tooling, output = (root / name for name in ('product', 'tooling', 'evidence'))
            product_revision = self.checkout(product, 'frozen product')
            tooling_revision = self.checkout(tooling, 'recovery tooling')
            with patch.object(section, 'PRODUCT', product_revision), patch.dict(
                    os.environ, GITHUB_SHA=tooling_revision, GITHUB_RUN_ID='123', SECTION='win_amd64'):
                section.bind(product, tooling, output)
                sources = json.loads((output / 'sources.json').read_text())
                self.assertEqual(sources['product']['revision'], product_revision)
                self.assertEqual(sources['tooling']['revision'], tooling_revision)
                self.assertNotEqual(sources['product']['tracked_bytes_sha256'],
                                    sources['tooling']['tracked_bytes_sha256'])
                (product / 'dist').mkdir()
                (product / 'dist/candidate.whl').write_bytes(b'producer bytes')
                section.retain(product, tooling, output)
                retained = json.loads((output / 'artifacts.json').read_text())
                self.assertEqual(retained['product'], sources['product'])
                self.assertEqual(retained['tooling'], sources['tooling'])
                self.assertEqual(retained['sha256']['dist/candidate.whl'],
                                 section.hashlib.sha256(b'producer bytes').hexdigest())
                (product / 'source').write_text('changed product')
                with self.assertRaisesRegex(ValueError, 'tracked checkout bytes'):
                    section.retain(product, tooling, output)

    def test_wrong_revision_and_changed_pins_fail_before_build(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            revision = self.checkout(root / 'source', 'source')
            with self.assertRaisesRegex(ValueError, 'checkout revision'):
                section.identity(root / 'source', 'a' * 40)
            self.assertEqual(section.identity(root / 'source', revision)['revision'], revision)


if __name__ == '__main__':
    unittest.main()
