"""Hosted release contracts preserve candidate checks and truthful summaries."""
from pathlib import Path
import unittest
import subprocess
import sys

from release.workflow import PLATFORMS


class HostedWorkflow(unittest.TestCase):
    def test_release_uses_supported_apple_arm_runner(self):
        workflow = Path('.github/workflows/release.yml').read_text()
        self.assertNotIn('macos-14', workflow)
        arm, = [row for row in PLATFORMS if row['platform'] == 'macosx_11_0_arm64']
        self.assertEqual(arm['os'], 'macos-15')
        self.assertIn('os: macos-15\n            platform: macosx_11_0_arm64', workflow)
        self.assertIn('channel: homebrew\n            os: macos-15\n', workflow)

    def test_release_has_no_environment_inspection_or_manual_pause(self):
        workflow = Path('.github/workflows/release.yml').read_text()
        self.assertNotIn('release/workflow.py protection', workflow)
        self.assertNotIn('/environments/', workflow)
        self.assertIn('      rehearsal:\n', workflow)
        result = subprocess.run([sys.executable, 'release/workflow.py', '--help'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn('protection', result.stdout)

    def test_native_bootstrap_quality_and_staged_formula_are_exercised(self):
        workflow = Path('.github/workflows/release.yml').read_text()
        qualify = workflow.split('\n  qualify:\n')[1].split('\n  stage:\n')[0]
        bootstrap = qualify.split('uses: actions/setup-python@')[0]
        self.assertIn("runner.os == 'Windows' && 'python' || 'python3'", bootstrap)
        self.assertIn('make fmt lint test', qualify)
        self.assertIn("matrix.platform == 'manylinux_2_28_x86_64'", qualify)
        self.assertIn('--staged-formula staged/biomcp.rb', workflow)
        self.assertIn('needs: [candidate, stage, staged-homebrew]', workflow)
        self.assertNotIn('Ian approved BioMCP', workflow)
        self.assertIn('Rehearsal reached; no channel authentication or publication attempted.', workflow)


if __name__ == '__main__':
    unittest.main()
