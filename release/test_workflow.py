"""Release staging contracts exercised through the operator command."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest
import zipfile

from release.test_licenses import archive as write_archive, payloads, wheel_fixture, write_payloads


class Staging(unittest.TestCase):
    def test_workflow_attests_all_subjects_before_candidate_upload(self):
        workflow = Path(".github/workflows/release.yml").read_text()
        stage = workflow.split("\n  stage:\n", 1)[1].split("\n  staged-homebrew:\n", 1)[0]
        self.assertIn("subject-checksums: ${{ runner.temp }}/subjects.sha256", stage)
        self.assertIn("create-storage-record: false", stage)
        self.assertIn("push-to-registry: false", stage)
        attest = re.search(r"      - name: Attest every staged artifact\n(.*?)(?=      - )", stage, re.S)
        self.assertIsNotNone(attest)
        self.assertRegex(attest[1], r"uses: actions/attest@[0-9a-f]{40}")
        self.assertNotIn("if:", attest[1])
        self.assertLess(stage.index("release/secrets.py artifacts"), attest.start())
        self.assertLess(attest.end(), stage.index("name: release-candidate"))
        # Artifact signing remains isolated from channel publication credentials.
        self.assertIn('      id-token: write', stage)
        self.assertIn('      attestations: write', stage)
        self.assertNotIn('      packages: write', stage)
        self.assertNotIn('      contents: write', stage)

    def test_tag_builds_skip_cache_restore(self):
        workflow = Path('.github/workflows/release.yml').read_text()
        caches = re.findall(r'      - uses: actions/cache@[^\n]+\n(.*?)(?=      - |\Z)', workflow, re.S)
        self.assertTrue(caches)
        for cache in caches:
            self.assertIn("        if: github.ref_type != 'tag'\n", cache)

    def test_environment_requires_an_independent_operator(self):
        for rules, accepted in [([], False), ([{"type": "required_reviewers", "reviewers": []}], False),
                                ([{"type": "required_reviewers", "reviewers": [{"type": "Team"}], "prevent_self_review": False}], False),
                                ([{"type": "required_reviewers", "reviewers": [{"type": "Team"}], "prevent_self_review": True}], True)]:
            for bypass in (False, True):
                with self.subTest(rules=rules, bypass=bypass), tempfile.TemporaryDirectory() as directory:
                    settings = Path(directory) / "environment.json"
                    settings.write_text(json.dumps({"protection_rules": rules, "can_admins_bypass": bypass}))
                    result = subprocess.run([sys.executable, "release/workflow.py", "protection", str(settings)], capture_output=True, text=True)
                    self.assertEqual(result.returncode == 0, accepted and not bypass, result.stderr)

    def prepare(self, root):
        artifacts = root / "artifacts"
        platforms = ["manylinux_2_28_x86_64", "manylinux_2_28_aarch64",
                     "macosx_11_0_x86_64", "macosx_11_0_arm64", "win_amd64"]
        archives = ["linux-x86_64.tar.gz", "linux-arm64.tar.gz",
                    "darwin-x86_64.tar.gz", "darwin-arm64.tar.gz", "windows-x86_64.zip"]
        for platform, archive in zip(platforms, archives):
            directory = artifacts / platform
            directory.mkdir(parents=True)
            _, record = wheel_fixture(directory, platform)
            write_payloads(directory)
            (directory / "release.json").write_text(json.dumps(record))
            name = "biomcp-" + archive
            write_archive(directory / name, {"biomcp": platform.encode(), **payloads()})
            digest = hashlib.sha256((directory / name).read_bytes()).hexdigest()
            (directory / (name + ".sha256")).write_text(digest + "  " + name + "\n")
            write_archive(directory / ("biomcp-" + platform + ".mcpb"), payloads())
            registry = {"name": "io.github.genomoncology/biomcp", "version": "1.0.0"}
            (directory / "server.json").write_text(json.dumps(registry))
            if platform.startswith("manylinux"):
                arch = "amd64" if platform.endswith("x86_64") else "arm64"
                layer = directory / "layer.tar"
                write_archive(layer, {"usr/share/doc/biomcp/" + name: value for name, value in payloads().items()})
                write_archive(directory / ("biomcp-container-" + arch + ".tar"), {
                    "manifest.json": b'[ {"Layers": ["layer.tar"]} ]', "layer.tar": layer.read_bytes()})
                layer.unlink()
        plugin = root / "plugin"
        (plugin / ".claude-plugin").mkdir(parents=True)
        (plugin / ".claude-plugin/plugin.json").write_text('{"name":"biomcp","version":"1.0.0","skills":["./skills/"]}')
        (plugin / ".claude-plugin/marketplace.json").write_text('{"name":"biomcp","plugins":[{"name":"biomcp","source":"./","version":"1.0.0","mcpServers":{"biomcp":{"command":"biomcp","args":["serve"]}}}]}')
        (plugin / "skills").mkdir()
        (plugin / "skills/SKILL.md").write_text("BioMCP guide\n")
        return artifacts, plugin

    def stage(self, root, artifacts, plugin):
        return subprocess.run([sys.executable, "release/workflow.py", "stage",
                               "--artifacts", str(artifacts), "--plugin", str(plugin),
                               "--version", "1.0.0", "--revision", "a" * 40,
                               "--output", str(root / "staged")], capture_output=True, text=True)

    def test_stage_preserves_public_channels_and_native_payloads(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            artifacts, plugin = self.prepare(root)
            result = self.stage(root, artifacts, plugin)
            self.assertEqual(result.returncode, 0, result.stderr)
            staged = root / "staged"
            formula = (staged / "biomcp.rb").read_text()
            for arch in ["arm64", "x86_64"]:
                self.assertIn("https://github.com/genomoncology/biomcp/releases/download/v1.0.0/biomcp-darwin-" + arch + ".tar.gz", formula)
                digest = hashlib.sha256((artifacts / ("macosx_11_0_" + arch) / ("biomcp-darwin-" + arch + ".tar.gz")).read_bytes()).hexdigest()
                self.assertIn(digest, formula)
            self.assertNotIn("file:", formula)
            self.assertEqual(len(list(staged.glob("*.whl"))), 5)
            self.assertEqual(len(list(staged.glob("*.mcpb"))), 3)
            self.assertEqual(len(list(staged.glob("biomcp-container-*.tar"))), 2)
            self.assertEqual((staged / "install.sh").read_bytes(), Path("release/install.sh").read_bytes())
            notes = (staged / "RELEASE-NOTES.md").read_text()
            self.assertIn("gh attestation verify", notes)
            self.assertIn("--source-ref refs/tags/v1.0.0", notes)
            self.assertIn("replace --source-ref with the actual rehearsal ref", notes)
            self.assertIn("--repo genomoncology/biomcp", notes)
            self.assertIn("a" * 40, notes)
            checksums = root / "subjects.sha256"
            subjects = subprocess.run([sys.executable, "release/workflow.py", "subjects",
                                       "--artifacts", str(staged), "--output", str(checksums)],
                                      capture_output=True, text=True)
            self.assertEqual(subjects.returncode, 0, subjects.stderr)
            actual = {name: digest for digest, name in
                      (line.split("  ", 1) for line in checksums.read_text().splitlines())}
            expected = {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                        for path in staged.iterdir()}
            self.assertEqual(actual, expected)
            with zipfile.ZipFile(staged / "biomcp-plugin.zip") as archive:
                self.assertIn(".claude-plugin/marketplace.json", archive.namelist())
                for name, data in payloads().items():
                    self.assertEqual(archive.read(name), data)
                self.assertEqual(archive.read("skills/SKILL.md"), b"BioMCP guide\n")
                self.assertEqual(json.loads(archive.read(".claude-plugin/plugin.json"))["skills"], ["./skills/"])

    def test_stage_refuses_incomplete_or_mixed_candidates(self):
        for defect in ["missing-platform", "revision", "checksum", "registry", "plugin", "skills", "plugin-version"]:
            with self.subTest(defect=defect), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                artifacts, plugin = self.prepare(root)
                linux = artifacts / "manylinux_2_28_x86_64"
                if defect == "missing-platform":
                    (linux / "release.json").unlink()
                elif defect == "revision":
                    path = linux / "release.json"
                    path.write_text(path.read_text().replace("a" * 40, "b" * 40))
                elif defect == "checksum":
                    (linux / "biomcp-linux-x86_64.tar.gz").write_bytes(b"corrupt")
                elif defect == "registry":
                    (linux / "server.json").write_text('{"version":"0.9.3"}')
                elif defect == "plugin":
                    (plugin / ".claude-plugin/plugin.json").write_text('{"name":"other"}')
                elif defect == "skills":
                    (plugin / ".claude-plugin/plugin.json").write_text('{"name":"biomcp"}')
                else:
                    path = plugin / ".claude-plugin/plugin.json"
                    metadata = json.loads(path.read_text())
                    del metadata['version']
                    path.write_text(json.dumps(metadata))
                self.assertNotEqual(self.stage(root, artifacts, plugin).returncode, 0)

    def test_subjects_refuse_missing_empty_or_unsafe_staging(self):
        for defect in ["missing", "empty", "symlink", "newline", "output-inside"]:
            with self.subTest(defect=defect), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                staged = root / "staged"
                if defect != "missing":
                    staged.mkdir()
                if defect == "symlink":
                    (root / "outside").write_bytes(b"outside")
                    (staged / "link").symlink_to(root / "outside")
                elif defect == "newline":
                    (staged / "bad\nname").touch()
                output = root / "subjects.sha256"
                if defect == "output-inside":
                    (staged / "artifact").write_bytes(b"candidate")
                    output = staged / "subjects.sha256"
                result = subprocess.run([sys.executable, "release/workflow.py", "subjects",
                                         "--artifacts", str(staged), "--output", str(output)],
                                        capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
