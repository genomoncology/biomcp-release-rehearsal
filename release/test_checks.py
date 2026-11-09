"""Installed smoke contracts still reject broken candidates under Python optimization."""
import json
import io
import os
from collections import Counter
from contextlib import redirect_stdout
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from release import secrets as history_scan
from release.smoke import SCHEMA_ID, environment, later_suite_timeout


class ReviewedPublicHistory(unittest.TestCase):
    """Exercise the real append ancestry when its safe scan receipt is supplied."""

    @classmethod
    def setUpClass(cls):
        target = os.environ.get("BIOMCP_APPEND_HISTORY")
        findings = os.environ.get("BIOMCP_APPEND_FINDINGS")
        if not target or not findings:
            raise unittest.SkipTest("requires the complete public append ancestry and safe scan receipt")
        cls.target = Path(target).resolve(strict=True)
        cls.observed = json.loads(Path(findings).read_text())
        cls.rows = [json.loads(line) for line in history_scan.HISTORY_IGNORE.read_text().splitlines()]
        cls.cache = Path(__file__).resolve().parents[1] / "target/security-tools"

    @staticmethod
    def identity(row):
        return row["blob"], row["file"], row["line"], row["rule"]

    def reviewed_rows(self):
        observed = Counter(map(self.identity, self.observed))
        self.assertEqual(len(self.observed), 50, "actual public scan receipt changed")
        self.assertEqual(set(observed.values()), {1}, "one row must not cover several findings")
        selected = [row for row in self.rows if self.identity(row) in observed]
        self.assertEqual(Counter(map(self.identity, selected)), observed,
                         "each actual finding requires exactly one reviewed binding")
        return selected

    def test_each_row_matches_one_actual_finding(self):
        self.reviewed_rows()

    def test_production_history_scan_passes(self):
        with tempfile.TemporaryDirectory() as directory:
            env = {**environment(Path(directory)), "PATH": os.environ["PATH"]}
            result = subprocess.run(
                [sys.executable, history_scan.__file__, "history", str(self.target),
                 "--cache", str(self.cache)], env=env, capture_output=True, text=True)
        print(result.stdout, end="")
        self.assertEqual(result.returncode, 0, "production public history scan failed")

    def test_removing_any_reviewed_row_fails_the_production_scanner(self):
        # The full scan above owns ancestry coverage. Each negative scan selects
        # only that immutable historical blob, without changing scanner policy.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            env = {**environment(root), "PATH": os.defpath, "GIT_CONFIG_NOSYSTEM": "1"}
            ledger = root / "secrets.ignore.jsonl"
            for row in self.reviewed_rows():
                with self.subTest(file=row["file"], blob=row["blob"], line=row["line"]):
                    blob = history_scan.git(self.target, env, "rev-parse",
                                            row["commit"] + ":" + row["file"]).decode().strip()
                    self.assertEqual(blob, row["blob"], "reviewed source blob changed")
                    payload = history_scan.git(self.target, env, "show",
                                               row["commit"] + ":" + row["file"])
                    ledger.write_text("".join(json.dumps(item) + "\n"
                                              for item in self.rows if item != row))
                    output = io.StringIO()
                    with (patch.object(history_scan, "HISTORY_IGNORE", ledger),
                          patch.object(history_scan, "inputs",
                                       return_value=iter([(row["file"], payload)])),
                          redirect_stdout(output)):
                        status = history_scan.scan("history", self.target, self.cache)
                    self.assertEqual(status, 1, "removing a reviewed row must reject its finding")
                    reported = [json.loads(line) for line in output.getvalue().splitlines()
                                if line.startswith("{")]
                    findings = [item for item in reported if item.get("level") != "info"]
                    self.assertEqual(findings, [{"file": row["file"], "rule": row["rule"]}],
                                     "omission must expose exactly its one actual finding")
        print("Reviewed public history row omissions rejected: 50")


class ReviewedSiblingHistory(unittest.TestCase):
    """Check exact public sibling rows against their real immutable blobs."""

    def test_each_reviewed_sibling_row_is_exact_and_required(self):
        target = os.environ.get("BIOMCP_APPEND_HISTORY")
        if not target:
            self.skipTest("requires the complete public append ancestry")
        target = Path(target)
        path = "src/entities/variant/resolution/coding_oracles/aliases.json"
        sources = [
            ("6c9c2d5995f9b2f2b83fa2625f6f540ae8799e2d", "5b4821ad23375d8096e596c42026e134045ae9bf"),
            ("6d517ab2047dad49695adbe918596d856295c562", "8ce1e9a200985491fa3e6d30aa982bd3d717bff9"),
        ]
        rows = [json.loads(line) for line in history_scan.HISTORY_IGNORE.read_text().splitlines()]
        selected = [row for row in rows if row["file"] == path]
        self.assertEqual(Counter((row["commit"], row["blob"], row["line"], row["rule"]) for row in selected),
                         Counter((sources[0][0], sources[0][1], line, "generic-api-key")
                                 for line in (2306, 2361, 2421)),
                         "the safe two-blob scan receipt requires exactly three reviewed rows")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            env = {**environment(root), "PATH": os.defpath, "GIT_CONFIG_NOSYSTEM": "1"}
            payloads = []
            for commit, blob in sources:
                self.assertEqual(history_scan.git(target, env, "rev-parse", commit + ":" + path).decode().strip(), blob)
                payloads.append((path, history_scan.git(target, env, "cat-file", "blob", blob)))
            ledger = root / "secrets.ignore.jsonl"
            original = history_scan.findings
            def screen(records):
                ledger.write_text("".join(json.dumps(row) + "\n" for row in records))
                observed = []
                def collect(report, mapping, staging):
                    for finding, staged, coordinates in original(report, mapping, staging):
                        observed.append({key: finding[key] for key in ("blob", "file", "line", "rule")})
                        yield finding, staged, coordinates
                with (patch.object(history_scan, "HISTORY_IGNORE", ledger),
                      patch.object(history_scan, "inputs", return_value=iter(payloads)),
                      patch.object(history_scan, "findings", side_effect=collect),
                      redirect_stdout(io.StringIO())):
                    status = history_scan.scan("history", target, Path("target/security-tools"))
                return status, observed
            self.assertEqual(screen(rows), (0, []), "production screening must accept both reviewed sibling blobs")
            for row in selected:
                with self.subTest(blob=row["blob"], line=row["line"]):
                    expected = {key: row[key] for key in ("blob", "file", "line", "rule")}
                    self.assertEqual(screen([item for item in rows if item != row]), (1, [expected]),
                                     "omitting one row must expose exactly its own finding")
        print("Reviewed public sibling row omissions rejected: 3")


class SmokeSchema(unittest.TestCase):
    def test_installed_probe_schema_is_in_checked_in_registry(self):
        schemas = Path(__file__).resolve().parents[1] / "crates/biomcp-types/schemas"
        schema_ids = {json.loads(path.read_text())["$id"] for path in schemas.glob("*.json")}
        self.assertIn(SCHEMA_ID, schema_ids, "installed smoke schema is absent from checked-in registry")


@unittest.skipIf(sys.platform == "win32", "the local executable stubs use POSIX shebangs")
class OptimizedSmoke(unittest.TestCase):
    def test_reports_checks_from_completed_installed_commands(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "candidate"
            log = root / "commands.jsonl"
            candidate = """import json, os, pathlib, sys
with open(LOG, 'a') as stream:
    stream.write(json.dumps(sys.argv[1]) + '\\n')
if sys.argv[1] == 'schema':
    print(json.dumps({'$id': sys.argv[2]}))
elif sys.argv[1] == 'help':
    print(json.dumps({'result': {'commands': ['get']}}))
else:
    cache = pathlib.Path(os.environ['XDG_CACHE_HOME'])
    cache.mkdir(parents=True, exist_ok=True)
    (cache / 'reply.sqlite').touch()
    for value in [
        {'jsonrpc': '2.0', 'id': 1, 'result': {'serverInfo': {'version': '1'}}},
        {'jsonrpc': '2.0', 'id': 2, 'result': {'tools': [{'name': 'biomcp'}]}},
        {'jsonrpc': '2.0', 'id': 3, 'result': {}},
    ]:
        print(json.dumps(value))
"""
            binary.write_text(f"#!{sys.executable}\nLOG = {str(log)!r}\n" + candidate)
            binary.chmod(0o755)
            runner = root / "runner"
            runner.write_text(f"#!{sys.executable}\nimport json, sys\nwith open({str(log)!r}, 'a') as stream:\n    stream.write(json.dumps(sys.argv[1]) + '\\n')\nprint('test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out')\n")
            runner.chmod(0o755)
            tool = runner.with_name("tool-runner")
            tool.write_text(runner.read_text())
            tool.chmod(0o755)
            script = "from release.smoke import smoke\nimport json, sys\nprint(json.dumps(smoke(*sys.argv[1:])))"
            result = subprocess.run([sys.executable, "-O", "-c", script, str(binary), str(runner), str(root), "1"],
                                    capture_output=True, text=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stderr)
            completed = json.loads(result.stdout.splitlines()[-1])
            commands = [json.loads(line) for line in log.read_text().splitlines()]
            self.assertEqual(commands, ["schema", "help"] + ["health"] * 8 + ["serve", "--proofs", "--test-threads"])
            self.assertEqual(completed["checks"], ["schemas", "help", "concurrent-first-opens", "version", "mcp-all-stdout",
                             "ordinary-cache-created", "runner-proofs", "all-cases-and-trials"])
            self.assertEqual(completed["tools"], ["biomcp"])
            self.assertEqual(completed["trials"], 2)
            self.assertEqual(completed["timeout_seconds"], later_suite_timeout(completed["elapsed_seconds"]))
            tool.unlink()
            missing = subprocess.run([sys.executable, "-O", "-c", script, str(binary), str(runner), str(root), "1"],
                                     capture_output=True, text=True, timeout=30)
            self.assertNotEqual(missing.returncode, 0)
            self.assertIn("tool-runner", missing.stderr)


    def test_broken_installed_commands_fail_under_optimization(self):
        replies = [
            {"jsonrpc": "2.0", "id": 1, "result": {"serverInfo": {"version": "1"}}},
            {"jsonrpc": "2.0", "id": 2, "result": {"tools": [{"name": "biomcp"}]}},
            {"jsonrpc": "2.0", "id": 3, "result": {}},
        ]
        cases = [("schema", {"$id": "wrong"}, "schema identity"),
                 ("help", {"result": {"commands": []}}, "help commands")]
        for mutation, message in [("version", "MCP version"), ("tools", "MCP tool"),
                                  ("protocol", "MCP protocol"), ("ids", "MCP response IDs")]:
            altered = json.loads(json.dumps(replies))
            if mutation == "version":
                altered[0]["result"]["serverInfo"]["version"] = "wrong"
            elif mutation == "tools":
                altered[1]["result"]["tools"][0]["name"] = "wrong"
            elif mutation == "protocol":
                altered[0]["jsonrpc"] = "wrong"
            else:
                altered.pop()
            cases.append(("mcp", altered, message))
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "candidate"
            for boundary, reply, message in cases:
                with self.subTest(boundary=boundary, message=message):
                    # A real child executable supplies a planted bad candidate reply.
                    output = "\n".join(json.dumps(row) for row in reply) if boundary == "mcp" else json.dumps(reply)
                    binary.write_text(f"#!{sys.executable}\nimport sys\nprint({output!r})\n")
                    binary.chmod(0o755)
                    call = ("mcp_probe([sys.argv[1]], environment(root), root, '1')" if boundary == "mcp"
                            else "smoke(sys.argv[1], sys.argv[1], root, '1')")
                    if boundary == "help":
                        binary.write_text(f"#!{sys.executable}\nimport json, sys\nprint(json.dumps({{'$id': sys.argv[2]}}) if sys.argv[1] == 'schema' else {output!r})\n")
                    script = "from release.smoke import *\nimport sys\nroot = Path(sys.argv[2])\n" + call
                    result = subprocess.run([sys.executable, "-O", "-c", script, str(binary), str(root)],
                                            capture_output=True, text=True, timeout=30)
                    self.assertNotEqual(result.returncode, 0, result.stdout)
                    self.assertIn(message, result.stderr)


if __name__ == "__main__":
    unittest.main()
