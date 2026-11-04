"""Hosted release contracts preserve candidate checks and truthful summaries."""
from pathlib import Path
import re
import unittest
import subprocess
import sys

from release.workflow import PLATFORMS


class HostedWorkflow(unittest.TestCase):
    def test_security_checks_install_pinned_cargo_deny_first(self):
        workflow = Path('.github/workflows/release.yml').read_text()
        install = 'cargo install cargo-deny --version "${{ steps.tools.outputs.cargo_deny }}" --locked'
        checked = []
        for step in workflow.split('\n      - ')[1:]:
            run = re.search(r'^        run: (.*)$', step, re.M)
            if not run:
                continue
            body = step[run.end():] if run[1] == '|' else run[1]
            commands = '\n'.join(line.strip() for line in body.splitlines()
                                 if line.strip() and not line.lstrip().startswith('#'))
            checks = re.search(r'make audit|make [^\n]*docs-check|(?:release[/.]test_security)', commands)
            if checks:
                checked.append(step)
                self.assertIn(install, commands[:checks.start()].splitlines(),
                              'security checks need an active pinned cargo-deny install first in the same run block')
        self.assertTrue(checked, 'the release workflow must run security checks')

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
        checks = qualify.split('make fmt lint test docs-check', 1)[1]
        self.assertLess(checks.index('cargo clean --profile dev'), checks.index('run: make release-linux'))
        self.assertIn("matrix.platform == 'manylinux_2_28_x86_64'", qualify)
        self.assertIn('--staged-formula staged/biomcp.rb', workflow)
        self.assertIn('needs: [candidate, stage, staged-homebrew]', workflow)
        self.assertNotIn('Ian approved BioMCP', workflow)
        self.assertIn('Rehearsal reached; no channel authentication or publication attempted.', workflow)


if __name__ == '__main__':
    unittest.main()
