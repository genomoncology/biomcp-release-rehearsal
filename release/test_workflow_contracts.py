"""Hosted release contracts preserve candidate checks and truthful summaries."""
from pathlib import Path
import unittest


class HostedWorkflow(unittest.TestCase):
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
        self.assertIn('Approval reached; no channel authentication or publication attempted.', workflow)


if __name__ == '__main__':
    unittest.main()
