#!/usr/bin/env python3
"""Run only explicitly authorized, bounded client smoke from an M5 pm job.

No default action calls a model. The client runs on an existing SSH host while
its MCP server runs in this job's checkout; authentication never leaves the client.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import signal
import subprocess
import sys
import time


PROMPT = """Use BioMCP's typed get_trial tool exactly twice, in this order:
first input ["NCT02563002"]; then input ["lung cancer"]. These are arrays of
strings in the declared schema. Do not recover from the error,
retry, call other BioMCP tools, or use files, shell, web, or other data sources.
Quote one nested success record field (prefer the sponsor) with its JSON path
and exact value. Quote the second reply's error.code and error.valid_values
with JSON paths and exact values. Use only the actual replies; if data is hidden
or missing, report that limitation. Stop after reporting these two replies.
"""


def ssh(host, command, **kwargs):
    return subprocess.run(["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=8", host, shlex.join(command)], check=True, **kwargs)


def remote_scratch(host):
    scratch = ssh(host, ["mktemp", "-d", "-t", "biomcp-typed-smoke.XXXXXXXX"], capture_output=True, text=True, timeout=15).stdout.strip()
    if not scratch.startswith("/tmp/biomcp-typed-smoke."):
        raise RuntimeError("unexpected remote scratch path")
    return scratch


def cleanup_remote_scratch(host, scratch):
    ssh(host, ["rm", "-rf", "--", scratch], timeout=15)


def copy_script(host, source, scratch):
    receiver = """import hashlib, os, pathlib, stat, sys
directory = pathlib.Path(sys.argv[1])
metadata = directory.lstat()
if not stat.S_ISDIR(metadata.st_mode) or metadata.st_uid != os.geteuid() or metadata.st_mode & 0o077:
    raise SystemExit('remote scratch must be an owned private directory')
path = directory / sys.argv[2]
with os.fdopen(os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), 'wb') as output:
    output.write(sys.stdin.buffer.read())
print(hashlib.sha256(path.read_bytes()).hexdigest())
"""
    data = source.read_bytes()
    received = ssh(host, ["python3", "-c", receiver, scratch, source.name], input=data, capture_output=True, timeout=15).stdout.decode().strip()
    expected = hashlib.sha256(data).hexdigest()
    if received != expected:
        raise RuntimeError("remote script receiving hash differs")
    print(json.dumps({"transferred_script": source.name, "receiving_sha256": received}), flush=True)


def interrupted(_signal, _frame):
    raise KeyboardInterrupt("typed harness owner interrupted")


def own_process(command, timeout, **kwargs):
    child = subprocess.Popen(command, start_new_session=True, **kwargs)
    try:
        status = child.wait(timeout=timeout)
        if status:
            raise RuntimeError(f"owned process exit {status}")
    finally:
        # Kill the owned group even when its leader has exited before a child.
        try:
            os.killpg(child.pid, signal.SIGTERM)
        except ProcessLookupError:
            return_cleanup = False
        else:
            return_cleanup = True
        if return_cleanup:
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                try:
                    os.killpg(child.pid, 0)
                except ProcessLookupError:
                    break
                time.sleep(0.05)
            else:
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
        child.wait()


def check_sources():
    # Syntax only, in memory: no bytecode, model, MCP server, build or install.
    for name in ["typed_harness_smoke.py", "typed_harness_capture.py"]:
        path = Path(__file__).with_name(name)
        compile(path.read_text(), str(path), "exec")
    print("Python syntax passes; no client or product was launched")


def client(args):
    if not args.execute_approved_clients:
        raise RuntimeError("root model GO is required")
    if args.client == "pi" and not args.pi_subscription_evidence:
        raise RuntimeError("pi billing/access is unverified; no pi call permitted")
    version_command = [args.pi_node, args.pi_bundle] if args.client == "pi" else [args.claude]
    version = subprocess.check_output([*version_command, "--version"], text=True, timeout=15).strip()
    print(json.dumps({"client_version": version}), flush=True)
    scratch = Path(__file__).resolve().parent
    server = ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=8", args.server_host,
              shlex.join([args.server_python, args.recorder, "--binary", args.binary,
                          "--replay", args.replay, "--output", args.output])]
    config = {"mcpServers": {"typed_smoke": {"command": server[0], "args": server[1:]}}}
    if args.client == "pi":
        # Omit exposure intentionally: the installed client's default is codemode.
        directory = scratch / ".pi"
        directory.mkdir()
        (directory / "mcp.json").write_text(json.dumps(config))
        command = [args.pi_node, args.pi_bundle, "--approve", "--offline", "--mode", "json", "--print", "--no-session", "--no-extensions", "--extension", "builtin:mcp", "--extension", "builtin:codemode", "--extension", str(scratch / "typed_harness_turn_limit.ts"), "--no-builtin-tools", "--no-skills", "--no-prompt-templates", "--no-context-files", PROMPT]
    else:
        config_path = scratch / "mcp.json"
        config_path.write_text(json.dumps(config))
        command = [args.claude, "--print", "--verbose", "--output-format", "stream-json", "--no-session-persistence", "--disable-slash-commands", "--no-chrome", "--settings", '{"disableAllHooks":true}', "--strict-mcp-config", "--mcp-config", str(config_path), "--tools", "", "--allowedTools", "mcp__typed_smoke__get_trial", "--permission-mode", "dontAsk", "--permission-prompts", "none", "--max-budget-usd", "1.00", PROMPT]
    metadata = {"client": args.client, "pi_exposure": "default codemode", "outer_timeout_seconds": 180, "mcp_timeout_seconds": 60, "max_tools_calls": 2, "max_pi_provider_requests": 3, "claude_max_budget_usd": 1.00, "pi_subscription_evidence": args.pi_subscription_evidence}
    print(json.dumps({"smoke_configuration": metadata}), flush=True)
    # No hooks, sessions, provider/model overrides or authentication files added.
    own_process(command, 180, cwd=scratch)


def job(args):
    if not args.execute_approved_clients:
        raise RuntimeError("root model GO is required; use check for syntax-only preflight")
    if "pi" in args.clients and not args.pi_subscription_evidence:
        raise RuntimeError("pi access/billing verification is required before launching")
    output = Path(args.output).resolve()
    output.mkdir(parents=True, exist_ok=False)
    root = Path(__file__).resolve().parents[1]
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    if revision != args.final_revision:
        raise RuntimeError("checkout is not the explicitly authorized final revision")
    (output / "revision.txt").write_text(revision + "\n")
    binary_sha = hashlib.sha256(Path(args.binary).read_bytes()).hexdigest()
    if binary_sha != args.binary_sha256:
        raise RuntimeError("binary differs from the explicitly authorized artifact")
    (output / "binary-sha256.txt").write_text(binary_sha + "\n")
    for name in args.clients:
        scratch = remote_scratch(args.client_host)
        try:
            for script in [Path(__file__).resolve(), Path(__file__).with_name("typed_harness_turn_limit.ts")]:
                copy_script(args.client_host, script, scratch)
            command = ["python3", f"{scratch}/typed_harness_smoke.py", "client", "--client", name, "--execute-approved-clients", "--server-host", args.server_host, "--server-python", sys.executable, "--recorder", str(root / "scripts/typed_harness_capture.py"), "--binary", str(Path(args.binary).resolve()), "--replay", str(root / "tests/replies/trial/get/NCT02563002"), "--output", str(output / name)]
            if name == "pi":
                command += ["--pi-node", args.pi_node, "--pi-bundle", args.pi_bundle]
            else:
                command += ["--claude", args.claude]
            if args.pi_subscription_evidence:
                command += ["--pi-subscription-evidence", args.pi_subscription_evidence]
            with (output / f"{name}.events.jsonl").open("wb") as stdout, (output / f"{name}.stderr").open("wb") as stderr:
                remote = shlex.join(["zsh", "-lc", shlex.join(command)])
                own_process(["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=8", args.client_host, remote], 195, stdout=stdout, stderr=stderr)
            # The client can exit zero after two tool errors. Scope still failed.
            from typed_harness_capture import validate_transcript
            validate_transcript(output / name)
        finally:
            cleanup_remote_scratch(args.client_host, scratch)


def main():
    signal.signal(signal.SIGTERM, interrupted)
    signal.signal(signal.SIGHUP, interrupted)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["check", "job", "client"])
    parser.add_argument("--execute-approved-clients", action="store_true")
    parser.add_argument("--pi-subscription-evidence", default="")
    parser.add_argument("--binary-sha256")
    parser.add_argument("--client", choices=["pi", "claude"])
    parser.add_argument("--clients", nargs="+", choices=["pi", "claude"], default=["pi", "claude"])
    for name in ["client-host", "server-host", "server-python", "recorder", "binary", "replay", "output", "final-revision", "pi-node", "pi-bundle", "claude"]:
        parser.add_argument("--" + name)
    args = parser.parse_args()
    {"check": check_sources, "job": lambda: job(args), "client": lambda: client(args)}[args.action]()


if __name__ == "__main__":
    main()
