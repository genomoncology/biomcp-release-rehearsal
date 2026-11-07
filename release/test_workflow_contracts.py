"""Hosted release contracts preserve candidate checks and truthful summaries."""
from pathlib import Path
import re
import unittest
import subprocess
import sys

from release.workflow import PLATFORMS


class HostedWorkflow(unittest.TestCase):
    def test_native_gate_names_actual_tooling_after_retaining_the_producer(self):
        workflow = Path('.github/workflows/release.yml').read_text()
        qualify = workflow.split('\n  qualify:\n')[1].split('\n  stage:\n')[0]
        producer = qualify.index('python release/artifacts.py build --revision "$GITHUB_SHA" --output dist')
        retained = qualify.index('name: producer-${{ matrix.platform }}')
        gate = qualify.index('python release/gate.py --artifacts dist')
        self.assertLess(producer, retained)
        self.assertLess(retained, gate)
        step, = [step for step in qualify.split('\n      - ')
                 if 'python release/gate.py --artifacts dist' in step]
        self.assertIn('BIOMCP_BUILD_REVISION: ${{ github.sha }}', step)
        self.assertNotIn('BIOMCP_BUILD_REVISION: ${{ github.sha }}', qualify[:retained])
        linux = Path('release/linux.sh').read_text()
        self.assertIn('BIOMCP_BUILD_REVISION="$BIOMCP_TOOLING_REVISION" python release/gate.py', linux)
        self.assertIn('export BIOMCP_EVIDENCE_DIR=/output/fixture-evidence', linux)

    def test_selected_main_section_qualifies_only_its_product_sources(self):
        workflow = Path('.github/workflows/recover-section.yml').read_text()
        self.assertIn('PRODUCT_REVISION: ${{ inputs.main_revision }}', workflow)
        self.assertIn('PRODUCT_TREE: ${{ inputs.main_tree }}', workflow)
        self.assertNotIn('1bb97e64', workflow)
        gate, = [step for step in workflow.split('\n      - ')
                 if 'python release/gate.py --artifacts' in step]
        self.assertIn('working-directory: product', gate)
        self.assertIn('BIOMCP_BUILD_REVISION: ${{ env.PRODUCT_REVISION }}', gate)
        self.assertIn('--artifacts dist', gate)
        linux, = [step for step in workflow.split('\n      - ')
                  if 'sh release/linux.sh' in step]
        self.assertIn('BIOMCP_TOOLING_REVISION: ${{ env.PRODUCT_REVISION }}', linux)
        self.assertIn('BIOMCP_RELEASE_TOOLING_SOURCE="$GITHUB_WORKSPACE/product"', linux)
        self.assertLess(workflow.index('name: producer-${{ inputs.section }}'),
                        workflow.index('python release/gate.py --artifacts'))
        self.assertNotIn('original_run', Path('release/section.py').read_text())

    def test_native_sections_keep_room_above_the_slowest_platform(self):
        # Intel Mac qualified at 119m44s, and all five platforms share these jobs.
        qualify = Path('.github/workflows/release.yml').read_text().split('\n  qualify:\n')[1].split('\n  stage:\n')[0]
        recover = Path('.github/workflows/recover-section.yml').read_text()
        self.assertEqual(re.findall(r'timeout-minutes: (\d+)', qualify), ['180'])
        self.assertEqual(re.findall(r'^    timeout-minutes: (\d+)', recover, re.M), ['180'])

    def test_staged_wheels_and_installer_are_checked_before_any_channel(self):
        workflow = Path('.github/workflows/release.yml').read_text()
        job = workflow.split('\n  staged-payload:\n')[1].split('\n  publish-github:\n')[0]
        self.assertIn('python -m twine check --strict staged/*.whl', job)
        self.assertIn('published.py installer-bytes', job)
        # Third-party install steps stay out of the job that can sign.
        self.assertNotIn('id-token', job)
        for name in ['github', 'ghcr', 'pypi', 'homebrew', 'installer', 'registry']:
            job = workflow.split('\n  publish-' + name + ':\n', 1)[1]
            self.assertIn('staged-payload', re.search(r'^    needs: (.*)$', job, re.M)[1])

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

    def test_every_python_check_follows_setup_python_under_the_same_condition(self):
        for path in sorted(Path('.github/workflows').glob('*.yml')):
            jobs = re.split(r'\n  (?=[a-z][a-z-]*:\n)', path.read_text().split('\njobs:\n')[1])
            for job in jobs:
                steps = job.split('\n      - ')
                for i, step in enumerate(steps):
                    if 'toolchain.py python-check' not in step:
                        continue
                    setups = [s for s in steps[:i] if s.startswith('uses: actions/setup-python@')]
                    self.assertTrue(setups, f'{path} job {job.splitlines()[0]} runs python-check without setup-python')
                    condition = lambda text: re.findall(r'^\s+if: (.*)$', text, re.M)
                    self.assertEqual(condition(setups[-1]), condition(step),
                                     f'{path} job {job.splitlines()[0]} setup-python condition differs from python-check')

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
        self.assertIn('needs: [candidate, stage, staged-homebrew, staged-payload]', workflow)
        self.assertNotIn('Ian approved BioMCP', workflow)
        self.assertIn('Rehearsal reached; no channel authentication or publication attempted.', workflow)


if __name__ == '__main__':
    unittest.main()
