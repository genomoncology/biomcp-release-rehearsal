"""Exercise source and artifact custody at the recovery command boundary."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from release import section


class SectionCustody(unittest.TestCase):
    def test_cli_rejects_another_product_before_creating_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.checkout(root / 'product', 'another product')
            tooling_revision = self.checkout(root / 'tooling', 'tooling')
            result = subprocess.run([sys.executable, str(Path(section.__file__).resolve()),
                                     'bind', '--product', str(root / 'product'),
                                     '--tooling', str(root / 'tooling'), '--output', str(root / 'evidence')],
                                    env={**os.environ, 'GITHUB_SHA': tooling_revision,
                                         'PRODUCT_REVISION': 'a' * 40, 'PRODUCT_TREE': 'b' * 40},
                                    capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('checkout revision differs', result.stderr)
            self.assertFalse((root / 'evidence').exists())

    def test_cli_rejects_malformed_revision_and_mismatched_tree_before_build(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            revision = self.checkout(root / 'product', 'selected main')
            driver = self.checkout(root / 'tooling', 'driver')
            for requested, tree, error in (
                    ('main', 'b' * 40, 'full 40-character commit'),
                    (revision, 'b' * 40, 'checkout tree differs')):
                result = subprocess.run(
                    [sys.executable, str(Path(section.__file__).resolve()), 'bind',
                     '--product', str(root / 'product'), '--tooling', str(root / 'tooling'),
                     '--output', str(root / 'evidence')],
                    env={**os.environ, 'GITHUB_SHA': driver,
                         'PRODUCT_REVISION': requested, 'PRODUCT_TREE': tree},
                    capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(error, result.stderr)
                self.assertFalse((root / 'evidence').exists())

    def test_recovery_workflow_is_one_guarded_read_only_section(self):
        workflow = (Path(section.__file__).resolve().parents[1]
                    / '.github/workflows/recover-section.yml').read_text()
        for guard in ("github.repository == 'genomoncology/biomcp-release-rehearsal'",
                      "github.ref_type == 'branch'", "startsWith(github.ref_name, 'rehearsal-0642-')",
                      'inputs.rehearsal == true', 'PRODUCT_REVISION: ${{ inputs.main_revision }}',
                      'ref: ${{ env.PRODUCT_REVISION }}', 'contents: read',
                      'options: [manylinux_2_28_aarch64, macosx_11_0_arm64, win_amd64, manylinux_2_28_x86_64, macosx_11_0_x86_64]'):
            self.assertIn(guard, workflow)
        self.assertEqual(workflow.count('    runs-on:'), 1)
        for forbidden in ('push:', 'pull_request:', 'schedule:', 'contents: write',
                          'packages: write', 'id-token: write', 'secrets.', 'publication.py'):
            self.assertNotIn(forbidden, workflow)

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
            with patch.dict(os.environ, PRODUCT_REVISION=product_revision,
                            PRODUCT_TREE=section.git(product, 'rev-parse', 'HEAD^{tree}'),
                            GITHUB_SHA=tooling_revision, GITHUB_RUN_ID='123', SECTION='win_amd64'):
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
            tooling = root / 'tooling'
            self.checkout(tooling, 'tooling')
            (tooling / 'Cargo.lock').write_text('different pin')
            subprocess.run(['git', '-C', str(tooling), 'add', '.'], check=True)
            subprocess.run(['git', '-C', str(tooling), '-c', 'user.name=Fixture',
                            '-c', 'user.email=fixture@example.org', 'commit', '-qm', 'Change pin'], check=True)
            tooling_revision = section.git(tooling, 'rev-parse', 'HEAD')
            with patch.dict(os.environ, PRODUCT_REVISION=revision,
                            PRODUCT_TREE=section.git(root / 'source', 'rev-parse', 'HEAD^{tree}'),
                            GITHUB_SHA=tooling_revision):
                with self.assertRaisesRegex(ValueError, 'preserve product tool pins'):
                    section.bind(root / 'source', tooling, root / 'evidence')


if __name__ == '__main__':
    unittest.main()
