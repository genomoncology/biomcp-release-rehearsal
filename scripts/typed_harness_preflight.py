#!/usr/bin/env python3
"""Synthetic pipe checks only; never starts BioMCP, a client, or a provider."""

import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time

from typed_harness_capture import exchange
from typed_harness_smoke import PI_CODE, check_sources, cleanup_remote_scratch, copy_script, interrupted, own_process, remote_scratch, ssh


def pi_prompt_contract():
    # Execute only the prompt's JavaScript against a synthetic, exact-name API.
    # Neither Pi, BioMCP, authentication nor a provider is loaded.
    code = """const calls = [], output = [];
const tools = {mcp__typed_smoke__get_trial: async (args) => {
  calls.push(args);
  return calls.length === 1
    ? {structuredContent: {status: 'complete', result: {items: [{card: {sponsor: 'synthetic sponsor'}}]}}}
    : {isError: true, structuredContent: {status: 'usage', error: {code: 'usage.arguments', valid_values: ['synthetic canonical identifier']}}};
}};
const text = (value) => output.push(value);
""" + PI_CODE + "\nconsole.log(JSON.stringify({calls, output}));"
    result = subprocess.run(["node", "--input-type=module", "-e", code], check=True, capture_output=True, text=True, timeout=5)
    observed = json.loads(result.stdout)
    assert observed["calls"] == [{"input": ["NCT02563002"]}, {"input": ["lung cancer"]}]
    first, second = observed["output"]
    assert first["structuredContent"]["result"]["items"][0]["card"]["sponsor"] == "synthetic sponsor"
    assert second["isError"] is True
    assert second["structuredContent"]["error"] == {"code": "usage.arguments", "valid_values": ["synthetic canonical identifier"]}
    print("Pi prompt exact-name API, sequential two-call inputs and complete success/error forwarding pass; synthetic only")


def server(both_errors=False):
    Path(sys.argv[3]).touch()
    for line in sys.stdin.buffer:
        frame = json.loads(line)
        # Whitespace is intentional: forwarding must retain the original bytes.
        usage = frame["params"]["arguments"]["input"] == ["lung cancer"]
        body = {"status": "usage" if usage else "complete"}
        if usage:
            body["error"] = {"code": "usage.arguments"}
        elif both_errors:
            body = {"status": "local", "error": {"code": "local.failure"}}
        result = {"synthetic": True, "isError": usage or both_errors, "structuredContent": body}
        response = b'{ "jsonrpc": "2.0", "id": ' + json.dumps(frame["id"]).encode() + b', "result": ' + json.dumps(result, separators=(",", ":")).encode() + b' }\n'
        sys.stdout.buffer.write(response)
        sys.stdout.buffer.flush()


def wait_started(child, path):
    deadline = time.monotonic() + 15
    while not path.exists():
        if child.poll() is not None:
            raise RuntimeError("synthetic child died before reporting startup")
        if time.monotonic() >= deadline:
            raise RuntimeError("synthetic child did not report startup")
        time.sleep(0.02)


def bridge(path, both_errors=False):
    root = Path(path)
    command = [sys.executable, __file__, "server", "both-errors" if both_errors else "normal", str(root / "server-ready")]
    child = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    try:
        wait_started(child, root / "server-ready")
        (root / "bridge-ready").touch()
        exchange(child, root)
        child.wait(timeout=15)
    finally:
        if child.poll() is None:
            child.kill()
            child.wait()


def request(index, value):
    return json.dumps({"jsonrpc": "2.0", "id": index, "method": "tools/call", "params": {"name": "get_trial", "arguments": {"input": [value]}}}).encode() + b"\n"


def cancellation(root):
    for termination in [signal.SIGTERM, signal.SIGHUP]:
        pid_file = root / str(termination)
        owner = subprocess.Popen([sys.executable, __file__, "owner", str(pid_file)], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        try:
            wait_started(owner, pid_file)
            pid = int(pid_file.read_text())
            owner.send_signal(termination)
            owner.communicate(timeout=15)
            try:
                os.kill(pid, 0)
            except ProcessLookupError:
                pass
            else:
                os.kill(pid, signal.SIGKILL)
                raise RuntimeError("owned child survived owner termination")
        finally:
            if owner.poll() is None:
                owner.kill()
                owner.communicate()


def run_bridge(root, inputs, both_errors=False):
    command = [sys.executable, __file__, "bridge", str(root)]
    if both_errors:
        command.append("both-errors")
    child = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        wait_started(child, root / "bridge-ready")
        stdout, stderr = child.communicate(inputs, timeout=15)
        return subprocess.CompletedProcess(command, child.returncode, stdout, stderr)
    finally:
        if child.poll() is None:
            child.kill()
            child.communicate()


def check():
    check_sources()
    pi_prompt_contract()
    with tempfile.TemporaryDirectory(prefix="typed-harness-preflight-") as temporary:
        root = Path(temporary)
        dead = subprocess.Popen([sys.executable, "-c", "raise SystemExit(7)"])
        dead.wait(timeout=15)
        try:
            wait_started(dead, root / "never-ready")
        except RuntimeError as error:
            if "died before reporting startup" not in str(error):
                raise
        else:
            raise RuntimeError("dead child passed startup preflight")
        inputs = request(1, "NCT02563002") + request(2, "lung cancer")
        successful = root / "successful"
        successful.mkdir()
        run = run_bridge(successful, inputs)
        if run.returncode or (successful / "requests.jsonl").read_bytes() != inputs:
            raise RuntimeError(f"synthetic forwarding failed: {run.stderr!r}")
        if (successful / "replies.jsonl").read_bytes() != run.stdout or run.stdout.count(b'"synthetic":true') != 2:
            raise RuntimeError("reply bytes changed or replies were lost")
        refused = root / "refused"
        refused.mkdir()
        run = run_bridge(refused, inputs + request(3, "lung cancer"))
        if run.returncode == 0 or b"unexpected tools/call" not in run.stderr:
            raise RuntimeError("third-call bound failed")
        both_errors = root / "both-errors"
        both_errors.mkdir()
        run = run_bridge(both_errors, inputs, both_errors=True)
        if run.returncode == 0 or b"expected complete success" not in run.stderr:
            raise RuntimeError("two-error pair silently passed")
        cancellation(root)
    print("Synthetic complete+usage pass, two-error rejection, byte forwarding, call bound and TERM/HUP cleanup pass; no product/client/provider ran")


def transport(host):
    scratch = remote_scratch(host)
    try:
        with tempfile.TemporaryDirectory(prefix="typed-transfer-preflight-") as temporary:
            payload = Path(temporary) / "public ' quoted payload.txt"
            payload.write_bytes(b"synthetic public transport payload\x00\n")
            for source in [Path(__file__).with_name("typed_harness_smoke.py"), Path(__file__).with_name("typed_harness_turn_limit.ts"), payload]:
                copy_script(host, source, scratch)
            try:
                copy_script(host, payload, scratch)
            except subprocess.CalledProcessError as error:
                if b"FileExistsError" not in error.stderr:
                    raise
            else:
                raise RuntimeError("duplicate destination was not refused")
    finally:
        cleanup_remote_scratch(host, scratch)
    ssh(host, ["test", "!", "-e", scratch], timeout=15)
    print("SSH receiving hashes, duplicate refusal and bounded private scratch cleanup pass; no product/client/provider ran")


if __name__ == "__main__":
    action = sys.argv[1] if len(sys.argv) > 1 else "check"
    if action == "server":
        server(len(sys.argv) > 2 and sys.argv[2] == "both-errors")
    elif action == "bridge":
        bridge(sys.argv[2], len(sys.argv) > 3 and sys.argv[3] == "both-errors")
    elif action == "check":
        check()
    elif action == "owner":
        signal.signal(signal.SIGTERM, interrupted)
        signal.signal(signal.SIGHUP, interrupted)
        own_process([sys.executable, __file__, "sleeper", sys.argv[2]], 180)
    elif action == "sleeper":
        ready = Path(sys.argv[2])
        staging = ready.with_name(ready.name + ".starting")
        staging.write_text(str(os.getpid()))
        staging.replace(ready)
        time.sleep(180)
    elif action == "transport":
        transport(sys.argv[2])
    else:
        raise SystemExit("expected check, bridge, or server")
