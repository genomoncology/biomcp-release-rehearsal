"""Latest pointers move only after every install passes and version ordering admits it."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class Promotion(unittest.TestCase):
    def test_workflow_requires_every_install_before_promotion(self):
        text = Path('.github/workflows/release.yml').read_text()
        job = text.split('\n  publish-latest:\n')[1]
        self.assertIn('post-publish]', job)
        self.assertIn('if: success() &&', job)
        self.assertIn('environment: release-publication', job)
        self.assertIn('group: release-latest', job)
        self.assertIn('cancel-in-progress: false', job)
        self.assertIn('DIGEST: ${{ needs.publish-ghcr.outputs.digest }}', job)
        self.assertIn('publication.py promote', job)
        self.assertNotIn('always()', job)

    def test_promotion_command_queries_every_release_before_moving_either_pointer(self):
        current = {'tag_name': 'v1.0.0', 'draft': False, 'prerelease': False}
        older = {'tag_name': 'v0.9.3', 'draft': False, 'prerelease': False}
        newer = {'tag_name': 'v1.0.1', 'draft': False, 'prerelease': False}
        cases = [
            ('newest', [[current], [older]], False, True),
            ('newer-on-later-page', [[current, older], [newer]], False, False),
            ('draft-and-prerelease', [[current, older, dict(newer, draft=True), dict(newer, prerelease=True)]], False, True),
            ('query-fails', [[current, older]], True, False),
            ('container-push-fails', [[current, older]], False, False),
            ('github-update-fails', [[current, older]], False, False),
            ('candidate-missing', [[older]], False, False),
            ('candidate-prerelease', [[dict(current, prerelease=True), older]], False, False),
        ]
        for name, pages, query_fails, promoted in cases:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                staged = root / 'staged'
                staged.mkdir()
                (staged / 'candidate.json').write_text(json.dumps({'version': '1.0.0', 'revision': 'a' * 40}))
                tools = root / 'tools'
                tools.mkdir()
                program = '''import json, os, sys
from pathlib import Path
name = Path(sys.argv[0]).name
with Path(os.environ['CALLS']).open('a') as output:
    output.write(json.dumps([name, *sys.argv[1:]]) + '\\n')
if name == 'gh' and sys.argv[1] == 'api':
    if os.environ['QUERY_FAILS'] == 'yes':
        sys.exit(7)
    print(os.environ['PAGES'])
if name == 'docker' and os.environ['DOCKER_FAILS'] == 'yes':
    sys.exit(8)
if name == 'gh' and sys.argv[1:3] == ['release', 'edit'] and os.environ['GITHUB_FAILS'] == 'yes':
    sys.exit(9)
'''
                for tool in ['gh', 'docker']:
                    path = tools / tool
                    path.write_text('#!' + sys.executable + '\n' + program)
                    path.chmod(0o700)
                calls = root / 'calls'
                env = {**os.environ, 'PATH': str(tools) + os.pathsep + os.defpath,
                       'GITHUB_REPOSITORY': 'genomoncology/biomcp', 'GITHUB_EVENT_NAME': 'push',
                       'GITHUB_REF': 'refs/tags/v1.0.0', 'GITHUB_SHA': 'a' * 40,
                       'CALLS': str(calls), 'PAGES': json.dumps(pages), 'QUERY_FAILS': 'yes' if query_fails else 'no',
                       'DOCKER_FAILS': 'yes' if name == 'container-push-fails' else 'no',
                       'GITHUB_FAILS': 'yes' if name == 'github-update-fails' else 'no'}
                result = subprocess.run([sys.executable, 'release/publication.py', 'promote', '--staged', str(staged),
                                         '--digest', 'sha256:' + 'b' * 64], env=env, capture_output=True, text=True)
                actual = [json.loads(line) for line in calls.read_text().splitlines()]
                self.assertEqual(actual[0], ['gh', 'api', '--paginate', '--slurp',
                                             'repos/genomoncology/biomcp/releases?per_page=100'])
                if promoted:
                    self.assertEqual(result.returncode, 0, result.stderr)
                    self.assertEqual(actual[1:], [
                        ['docker', 'buildx', 'imagetools', 'create', '--tag', 'ghcr.io/genomoncology/biomcp:latest',
                         'ghcr.io/genomoncology/biomcp@sha256:' + 'b' * 64],
                        ['gh', 'release', 'edit', 'v1.0.0', '--repo', 'genomoncology/biomcp', '--latest']])
                elif name == 'container-push-fails':
                    self.assertNotEqual(result.returncode, 0)
                    self.assertEqual(len(actual), 2)
                    self.assertEqual(actual[-1][0], 'docker', 'GitHub latest must not move after a failed container promotion')
                elif name == 'github-update-fails':
                    self.assertNotEqual(result.returncode, 0, 'partial promotion must report failure')
                    self.assertEqual(len(actual), 3, 'partial promotion must not attempt an automatic rollback')
                    self.assertEqual(actual[1][0], 'docker')
                    self.assertEqual(actual[2][:3], ['gh', 'release', 'edit'])
                else:
                    self.assertEqual(len(actual), 1, 'failed or ineligible promotion must leave both pointers unchanged')


if __name__ == '__main__':
    unittest.main()
