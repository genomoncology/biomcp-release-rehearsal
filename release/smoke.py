#!/usr/bin/env python3
"""Run one installed executable with copied fixtures and no checkout dependency."""
import concurrent.futures
import json
import os
import re
import shutil
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import threading

SCHEMA_ID = "biomcp.problem/2"


def phase(name):
    """Retain a fixed stage name independently of buffered runner output."""
    path = os.environ.get("BIOMCP_SMOKE_PHASE_FILE")
    if path:
        with Path(path).open("a") as output:
            output.write(json.dumps({"phase": name, "pid": os.getpid(),
                                     "monotonic_ns": time.monotonic_ns()}) + "\n")


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


def cache_first_opens(binary, root):
    """Eight real installed commands share one fresh cache, with no lock observer."""
    root = Path(root).resolve()
    root.mkdir(parents=True, exist_ok=True)
    env = environment(root)
    for name in ["home", "config", "cache"]:
        (root / name).mkdir()
    configuration = root / "config/biomcp"
    configuration.mkdir()
    (configuration / "config.json").write_text(json.dumps({"disable_sources": ["open", "noncommercial", "restricted"]}))
    barrier = threading.Barrier(9, timeout=60)
    for index in range(8):
        (root / f"{index:02}-sqlite.jsonl").write_bytes(b"")

    def run(index):
        local = dict(env, BIOMCP_SQLITE_DIAGNOSTICS=str(root / f"{index:02}-sqlite.jsonl"))
        barrier.wait()
        started = time.monotonic_ns()
        with (root / f"{index:02}.stdout").open("wb") as stdout, (root / f"{index:02}.stderr").open("wb") as stderr:
            try:
                result = subprocess.run([str(binary), "health", "--offline"], env=local,
                                        stdout=stdout, stderr=stderr, timeout=60)
                status = result.returncode
            except subprocess.TimeoutExpired:
                status = "watchdog"
        record = {"worker": index, "dispatch_start_ns": started,
                  "duration_ms": (time.monotonic_ns() - started) / 1e6, "exit": status}
        (root / f"{index:02}-exec.json").write_text(json.dumps(record) + "\n")
        return record

    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        futures = [pool.submit(run, index) for index in range(8)]
        empty = not any((root / "cache").iterdir())
        release = {"monotonic_ns": time.monotonic_ns(), "unix_ns": time.time_ns(),
                   "host_load": os.getloadavg() if hasattr(os, "getloadavg") else None,
                   "empty_cache": empty}
        barrier.wait()
        records = [future.result() for future in futures]
    (root / "barrier.json").write_text(json.dumps(release) + "\n")
    diagnostics = [json.loads(line) for index in range(8)
                   for line in (root / f"{index:02}-sqlite.jsonl").read_text().splitlines()]
    record = {"host_load": release["host_load"], "empty_cache_before_release": empty,
              "dispatch_spread_ms": (max(row["dispatch_start_ns"] for row in records) - min(row["dispatch_start_ns"] for row in records)) / 1e6,
              "exits": [row["exit"] for row in records], "sqlite_errors": diagnostics}
    (root / "commands.json").write_text(json.dumps(record, indent=2) + "\n")
    return record


def smoke(binary, runner, fixtures, version, server_args=("serve",), timeout=None, python=sys.executable):
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
        phase("schema")
        require(command(["schema", SCHEMA_ID])["$id"] == SCHEMA_ID, "schema identity differs")
        checks.append("schemas")
        phase("help")
        require(command(["help"])["result"]["commands"], "help commands missing")
        checks.append("help")
        evidence = Path(os.environ.get("BIOMCP_EVIDENCE_DIR", Path.cwd() / "target/fixture-evidence"))
        evidence.mkdir(parents=True, exist_ok=True)
        first_open = Path(tempfile.mkdtemp(prefix="installed-cache-first-open-", dir=evidence))
        phase("concurrent-first-opens")
        opened = cache_first_opens(binary, first_open)
        require(opened["empty_cache_before_release"] and opened["exits"] == [0] * 8,
                "installed concurrent first opens failed; retained evidence: " + str(first_open))
        require(any((first_open / "cache").rglob("*.sqlite")), "concurrent first opens did not create the cache")
        shutil.rmtree(first_open)
        checks.append("concurrent-first-opens")
        # The first tool call creates the ordinary cache; every emitted line,
        # including shutdown, must be a correlated JSON-RPC response.
        phase("stdio-mcp")
        tools = mcp_probe([str(binary)] + list(server_args), env, root, version)
        checks.extend(["version", "mcp-all-stdout"])
        require(any((root / "cache").rglob("*.sqlite")), "first tool call did not create the cache")
        checks.append("ordinary-cache-created")
        interpreter = shutil.which(python)
        require(interpreter is not None, "proof Python interpreter is unavailable: " + python)
        env.update(BIOMCP_TEST_EXECUTABLE=str(binary), BIOMCP_TEST_FIXTURES=str(fixtures),
                   BIOMCP_TEST_PYTHON=os.path.abspath(interpreter))
        tool = runner.with_name("tool-runner" + runner.suffix)
        phase("tool-proofs")
        case_suite([str(tool), "--proofs"], env, root, fixtures, timeout)
        checks.append("runner-proofs")
        phase("all-cases")
        summary = case_suite([str(runner)], env, root, fixtures, timeout)
        checks.append("all-cases-and-trials")
        phase("complete")
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


def mcp_probe(command, env, root, version, call_args=None):
    frames = [
        {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "installed-gate", "version": "1"}}},
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
        {"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "biomcp", "arguments": {"args": call_args or ["get", "gene", "KRAS", "--offline"]}}},
    ]
    result = subprocess.run(command, input="".join(json.dumps(f) + "\n" for f in frames), env=env, cwd=root, capture_output=True, text=True, timeout=60)
    replies = [json.loads(line) for line in result.stdout.splitlines()]
    require(result.returncode == 0 and not result.stderr, "MCP process failed: " + result.stderr)
    require(len(replies) == 3 and {r["id"] for r in replies} == {1, 2, 3}, "MCP response IDs differ")
    require(all(r["jsonrpc"] == "2.0" for r in replies), "MCP protocol differs")
    indexed = {r["id"]: r for r in replies}
    require(indexed[1]["result"]["serverInfo"]["version"] == version, "MCP version differs")
    require(indexed[2]["result"]["tools"][0]["name"] == "biomcp", "MCP tool differs")

    if call_args == ["help"]:
        reply = indexed[3]["result"]
        value = reply.get("structuredContent") or json.loads(reply["content"][0]["text"])
        require(not reply.get("isError") and value["result"]["commands"],
                "installed plugin command did not return help commands")
    return [tool["name"] for tool in indexed[2]["result"]["tools"]]
