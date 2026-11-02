#!/usr/bin/env python3
"""Run one installed executable with copied fixtures and no checkout dependency."""
import json
import os
import re
from pathlib import Path
import subprocess
import tempfile
import time

SCHEMA_ID = "biomcp.problem/2"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def environment(root):
    # No host keys, proxies, configuration, warm caches, or build PATH.
    root = Path(root).resolve()
    temporary = root / "tmp"
    temporary.mkdir(exist_ok=True)
    env = {"HOME": str(root / "home"), "USERPROFILE": str(root / "home"),
            "XDG_CONFIG_HOME": str(root / "config"), "XDG_CACHE_HOME": str(root / "cache"),
            "APPDATA": str(root / "config"), "LOCALAPPDATA": str(root / "cache"),
            "TMPDIR": str(temporary), "TMP": str(temporary), "TEMP": str(temporary)}
    # Winsock needs the operating system directory, even without a console.
    if os.name == "nt" and "SystemRoot" in os.environ:
        env["SystemRoot"] = os.environ["SystemRoot"]
    return env


def smoke(binary, runner, fixtures, version, server_args=("serve",), timeout=None):
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory).resolve()
        env = environment(root)
        binary = Path(binary).resolve()
        runner = Path(runner).resolve()
        fixtures = Path(fixtures).resolve()
        def command(args):
            result = subprocess.run([str(binary)] + args, env=env, cwd=root, capture_output=True, timeout=60)
            if result.returncode or result.stderr:
                raise ValueError("installed command failed: " + result.stdout.decode() + result.stderr.decode())
            return json.loads(result.stdout)
        checks = []
        require(command(["schema", SCHEMA_ID])["$id"] == SCHEMA_ID, "schema identity differs")
        checks.append("schemas")
        require(command(["help"])["result"]["commands"], "help commands missing")
        checks.append("help")
        # The first tool call creates the ordinary cache; every emitted line,
        # including shutdown, must be a correlated JSON-RPC response.
        tools = mcp_probe([str(binary)] + list(server_args), env, root, version)
        checks.extend(["version", "mcp-all-stdout"])
        require(any((root / "cache").rglob("*.sqlite")), "first tool call did not create the cache")
        checks.append("ordinary-cache-created")
        env.update(BIOMCP_TEST_EXECUTABLE=str(binary), BIOMCP_TEST_FIXTURES=str(fixtures))
        tool = runner.with_name("tool-runner" + runner.suffix)
        case_suite([str(tool), "--proofs"], env, root, fixtures, timeout)
        checks.append("runner-proofs")
        summary = case_suite([str(runner)], env, root, fixtures, timeout)
        checks.append("all-cases-and-trials")
        return {"tools": tools, "checks": checks, **summary}


def case_suite(command, env, root, fixtures, timeout=None, expected_filtered=None):
    started = time.monotonic()
    result = subprocess.run(command + ["--test-threads", "2"], env=env, cwd=root,
                            capture_output=True, text=True, timeout=timeout)
    elapsed = time.monotonic() - started
    print(result.stdout, end="", flush=True)
    require(result.returncode == 0, "installed cases failed: " + result.stdout + result.stderr)
    counts = re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out", result.stdout)
    filtered = 0 if expected_filtered is None else expected_filtered
    require(counts is not None and counts.group(2) == "0" and int(counts.group(5)) == filtered,
            "installed runner did not report the complete declared selection")
    if expected_filtered is not None:
        require(int(counts.group(1)) > 0, "container runner did not execute any proof")
    return {"cases": sum(1 for path in (fixtures / "cases").rglob("*") if path.suffix in (".yaml", ".yml")),
            "trials": int(counts.group(1)) + int(counts.group(3)),
            "elapsed_seconds": elapsed, "timeout_seconds": elapsed * 1.15}


def mcp_probe(command, env, root, version):
    frames = [
        {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "installed-gate", "version": "1"}}},
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
        {"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "biomcp", "arguments": {"args": ["get", "gene", "KRAS", "--offline"]}}},
    ]
    result = subprocess.run(command, input="".join(json.dumps(f) + "\n" for f in frames), env=env, cwd=root, capture_output=True, text=True, timeout=60)
    replies = [json.loads(line) for line in result.stdout.splitlines()]
    require(result.returncode == 0 and not result.stderr, "MCP process failed: " + result.stderr)
    require(len(replies) == 3 and {r["id"] for r in replies} == {1, 2, 3}, "MCP response IDs differ")
    require(all(r["jsonrpc"] == "2.0" for r in replies), "MCP protocol differs")
    indexed = {r["id"]: r for r in replies}
    require(indexed[1]["result"]["serverInfo"]["version"] == version, "MCP version differs")
    require(indexed[2]["result"]["tools"][0]["name"] == "biomcp", "MCP tool differs")

    return [tool["name"] for tool in indexed[2]["result"]["tools"]]
