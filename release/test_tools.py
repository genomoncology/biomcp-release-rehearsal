"""Exercise the ordinary gate's desktop tool comparison from source only."""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
import zipfile

from release.test_licenses import write_payloads

from release.artifacts import channels

ROOT = Path(__file__).resolve().parents[1]


class DesktopTools(unittest.TestCase):
    @unittest.skipIf(sys.platform == "win32", "the Docker stub uses a POSIX shebang")
    def test_container_probe_forwards_caller_arguments(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            stub = root / "docker"
            recorded = root / "arguments.json"
            stub.write_text("#!/usr/bin/env python3\nimport json, os, sys\n"
                            "from pathlib import Path\n"
                            "Path(os.environ['ARGUMENT_RECORD']).write_text(json.dumps(sys.argv[1:]))\n"
                            "sys.exit(19)\n")
            stub.chmod(0o755)
            extra = ["--label", "example.owner=release check", "--network", "none"]
            env = dict(os.environ, PATH=str(root) + os.pathsep + os.environ["PATH"],
                       ARGUMENT_RECORD=str(recorded))
            result = subprocess.run(
                [sys.executable, str(ROOT / "release/container_check.py"),
                 "candidate", "arm64", str(root), "1.0.0", *extra],
                env=env, capture_output=True, text=True, check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(json.loads(recorded.read_text()),
                             ["run", "--rm", "--memory", "2g", "--cpus", "2",
                              "--platform", "linux/arm64", *extra, "--entrypoint",
                              "/usr/bin/sha256sum", "candidate", "/usr/local/bin/biomcp"])

    def check(self, root):
        return subprocess.run([sys.executable, str(ROOT / "release/tools.py"), "--root", str(root)], capture_output=True, text=True, check=False)

    def test_manifest_matches_server_and_rejects_an_added_tool(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ("manifest.json", "crates/biomcp/src/mcp/protocol.rs", "crates/biomcp/src/mcp/research.rs"):
                target = root / name
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / name, target)
            result = self.check(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            protocol = root / "crates/biomcp/src/mcp/protocol.rs"
            original_protocol = protocol.read_text()
            protocol.write_text(original_protocol.replace("research::tools(root)?", "research::tools()?"))
            result = self.check(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("unrecognized server tool assembly", result.stderr)
            protocol.write_text(original_protocol)
            source = root / "crates/biomcp/src/mcp/research.rs"
            original = source.read_text()
            planted = re.sub(r'\[\s*\(\s*"search"\s*,',
                             lambda _: '[("fourth", "query", "A fake fourth tool"), ("search",',
                             original, count=1)
            self.assertNotEqual(planted, original, "planted tool did not change the source")
            source.write_text(planted)
            result = self.check(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("desktop manifest tools differ", result.stderr)
            manifest = root / "manifest.json"
            value = json.loads(manifest.read_text())
            value["tools"].append({"name": "fourth", "description": "A fake fourth tool"})
            manifest.write_text(json.dumps(value))
            result = self.check(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            original = source.read_text()
            mutations = [
                ('[("fourth", "query", "A \\"quoted\\" tool"), ("search",', "desktop manifest tools differ"),
                ('[(name(), "query", "A computed tool"), ("search",', "unrecognized server tool declarations"),
            ]
            value["tools"] = [tool for tool in value["tools"] if tool["name"] != "fourth"]
            manifest.write_text(json.dumps(value))
            for replacement, error in mutations:
                with self.subTest(replacement=replacement):
                    source.write_text(original.replace('[("fourth", "query", "A fake fourth tool"), ("search",', replacement, 1))
                    result = self.check(root)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn(error, result.stderr)

    @unittest.skipIf(sys.platform == "win32", "the local executable stub uses a POSIX shebang")
    def test_bundle_descriptions_come_from_candidate_tools_list(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "candidate"
            tools = [{"name": name, "description": "Candidate description for " + name} for name in ("biomcp", "search", "fetch")]
            frames = [{"jsonrpc": "2.0", "id": 1, "result": {}}, {"jsonrpc": "2.0", "id": 2, "result": {"tools": tools}}]
            binary.write_text("#!" + sys.executable + "\nimport sys\nsys.stdin.read()\nprint(" + repr("\n".join(json.dumps(frame) for frame in frames)) + ")\n")
            binary.chmod(0o755)
            write_payloads(root)
            channels(root, binary, {"version": "1.0.0", "platform": "test"})
            with zipfile.ZipFile(root / "biomcp-test.mcpb") as bundle:
                manifest = json.loads(bundle.read("manifest.json"))
                self.assertEqual(manifest["tools"], tools)
                self.assertEqual(bundle.read(manifest["server"]["entry_point"]), binary.read_bytes())


if __name__ == "__main__":
    unittest.main()
