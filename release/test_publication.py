"""Publication contracts keep channel credentials behind approval and check installs."""
import json
import hashlib
import io
import os
import re
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch


class Publication(unittest.TestCase):
    def test_every_channel_waits_for_approval_and_dispatch_cannot_publish(self):
        workflow = Path('.github/workflows/release.yml').read_text()
        for name in ['github', 'ghcr', 'pypi', 'homebrew', 'installer', 'registry', 'latest']:
            job = workflow.split('\n  publish-' + name + ':\n', 1)[1]
            job = re.split(r'\n  [a-z-]+:\n', job, maxsplit=1)[0]
            self.assertIn('environment: release-publication', job)
            for step in job.split('      - name: ')[1:]:
                if 'Report the approval rehearsal boundary' not in step:
                    self.assertIn("if: github.event_name == 'push' && github.repository == 'genomoncology/biomcp'", step)
        self.assertIn('attestations: true', workflow)
        self.assertIn('subject-digest: ${{ steps.image.outputs.digest }}', workflow)
        self.assertIn('push-to-registry: true', workflow)
        self.assertIn('make fmt lint test docs-check', workflow)
        post = workflow.split('\n  post-publish:\n')[1]
        for channel in ['pypi', 'ghcr', 'homebrew', 'installer', 'registry']:
            self.assertIn('channel: ' + channel, post)
            self.assertIn('publish-' + channel, post)

    def test_publication_command_refuses_unapproved_repository_event_or_source(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'candidate.json').write_text(json.dumps({'version': '1.0.0', 'revision': 'a' * 40}))
            base = {**os.environ, 'GITHUB_REPOSITORY': 'genomoncology/biomcp',
                    'GITHUB_EVENT_NAME': 'push', 'GITHUB_REF': 'refs/tags/v1.0.0', 'GITHUB_SHA': 'a' * 40}
            for name, value in [('GITHUB_REPOSITORY', 'example/scratch'), ('GITHUB_EVENT_NAME', 'workflow_dispatch'),
                                ('GITHUB_REF', 'refs/heads/main'), ('GITHUB_SHA', 'b' * 40)]:
                result = subprocess.run([sys.executable, 'release/publication.py', 'wheels', '--staged', str(root)],
                                        env={**base, name: value}, capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('publication identity', result.stderr)

    def test_publisher_install_verifies_checksum_and_signature_before_writing_executable(self):
        from release.toolchain import install_mcp_publisher
        archive = io.BytesIO()
        with tarfile.open(fileobj=archive, mode='w:gz') as package:
            entry = tarfile.TarInfo('mcp-publisher')
            entry.size = 9
            package.addfile(entry, io.BytesIO(b'publisher'))
        data = archive.getvalue()
        pins = {'mcp-publisher': {'version': 'v1.8.1', 'archives': {
            'linux_amd64': hashlib.sha256(data).hexdigest()}}}
        for defect in ['checksum', 'signature', 'none']:
            with self.subTest(defect=defect), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                def download(url, timeout):
                    return io.BytesIO(b'wrong' if defect == 'checksum' else data)
                def verify(command, check):
                    self.assertEqual(command[:2], ['cosign', 'verify-blob'])
                    self.assertIn('--certificate-identity', command)
                    self.assertIn('https://github.com/modelcontextprotocol/registry/.github/workflows/release.yml@refs/tags/v1.8.1', command)
                    if defect == 'signature':
                        raise subprocess.CalledProcessError(1, command)
                with patch('release.toolchain.manifest', return_value=pins), patch('release.toolchain.platform.system', return_value='Linux'), patch('release.toolchain.platform.machine', return_value='x86_64'), patch('release.toolchain.Path.home', return_value=root), patch('release.toolchain.urllib.request.urlopen', side_effect=download), patch('release.toolchain.subprocess.run', side_effect=verify) as verifier, patch.dict(os.environ, {'GITHUB_PATH': str(root / 'path')}):
                    if defect == 'none':
                        install_mcp_publisher()
                        self.assertEqual((root / '.local/bin/mcp-publisher').read_bytes(), b'publisher')
                        self.assertEqual((root / 'path').read_text(), str(root / '.local/bin') + '\n')
                    else:
                        with self.assertRaises((ValueError, subprocess.CalledProcessError)):
                            install_mcp_publisher()
                        self.assertFalse((root / '.local/bin/mcp-publisher').exists())
                    self.assertEqual(verifier.call_count, 0 if defect == 'checksum' else 1)

    def test_version_checker_rejects_successful_wrong_version(self):
        from release.published import check_version
        for output in ['biomcp 0.9.3\n', 'biomcp 1.0.0\nextra\n', '']:
            with self.assertRaises(ValueError):
                check_version(output, '1.0.0')
        check_version('biomcp 1.0.0\n', '1.0.0')


if __name__ == '__main__':
    unittest.main()
