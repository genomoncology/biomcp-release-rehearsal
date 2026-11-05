#!/usr/bin/env python3
"""Transparent MCP stdio capture for the bounded external-client smoke."""

import argparse
import json
import os
from pathlib import Path
import queue
import signal
import subprocess
import sys
import tempfile
import threading
import time


def pump(stream, label, events):
    while chunk := os.read(stream.fileno(), 65536):
        events.put((label, chunk))
    events.put((label, None))


def validate_transcript(output):
    requests = [json.loads(line) for line in (output / "requests.jsonl").read_text().splitlines() if line.strip()]
    calls = [frame for frame in requests if frame.get("method") == "tools/call"]
    replies = {frame.get("id"): frame.get("result", {}) for line in (output / "replies.jsonl").read_text().splitlines() if line.strip() for frame in [json.loads(line)]}
    if len(calls) != 2:
        raise RuntimeError("expected exactly two tools/call requests")
    first, second = [replies.get(call["id"], {}) for call in calls]
    success = first.get("structuredContent", {})
    usage = second.get("structuredContent", {})
    if first.get("isError") or success.get("status") != "complete":
        raise RuntimeError("expected complete success for first get_trial; raw evidence retained")
    if second.get("isError") is not True or usage.get("status") != "usage" or usage.get("error", {}).get("code") != "usage.arguments":
        raise RuntimeError("expected CLI usage error for second get_trial; raw evidence retained")


def exchange(child, output):
    events = queue.Queue()
    for stream, label in [(sys.stdin.buffer, "requests"), (child.stdout, "replies")]:
        threading.Thread(target=pump, args=(stream, label, events), daemon=True).start()
    calls, pending, buffers = [], {}, {"requests": b"", "replies": b""}
    with (output / "requests.jsonl").open("wb") as requests, (output / "replies.jsonl").open("wb") as replies:
        sinks = {"requests": requests, "replies": replies}
        while True:
            if any(time.monotonic() - started > 60 for started in pending.values()):
                raise RuntimeError("MCP call exceeded 60 seconds")
            try:
                label, chunk = events.get(timeout=0.1)
            except queue.Empty:
                continue
            if chunk is None:
                if label == "requests":
                    child.stdin.close()
                    continue
                break
            sinks[label].write(chunk)
            sinks[label].flush()
            buffers[label] += chunk
            while b"\n" in buffers[label]:
                line, buffers[label] = buffers[label].split(b"\n", 1)
                if not line.strip():
                    continue
                frame = json.loads(line)
                if label == "requests" and frame.get("method") == "tools/call":
                    expected = ["NCT02563002", "lung cancer"]
                    params = frame.get("params", {})
                    arguments = params.get("arguments", {})
                    if len(calls) >= 2 or params.get("name") != "get_trial" or arguments.get("input") != [expected[len(calls)]]:
                        raise RuntimeError("unexpected tools/call; refusing to forward")
                    calls.append(frame)
                    pending[frame["id"]] = time.monotonic()
                elif label == "replies":
                    pending.pop(frame.get("id"), None)
            if label == "requests":
                child.stdin.write(chunk)
                child.stdin.flush()
            else:
                sys.stdout.buffer.write(chunk)
                sys.stdout.buffer.flush()
    if len(calls) != 2 or pending or any(buffers.values()):
        raise RuntimeError("incomplete two-call MCP exchange")
    validate_transcript(output)


def interrupted(_signal, _frame):
    raise KeyboardInterrupt("MCP recorder interrupted")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--replay", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    signal.signal(signal.SIGTERM, interrupted)
    signal.signal(signal.SIGHUP, interrupted)
    args.output.mkdir(parents=True, exist_ok=False)
    with tempfile.TemporaryDirectory(prefix="biomcp-harness-") as temporary:
        # macOS /var and /tmp are aliases; replay rejects symlink ancestors.
        root = Path(temporary).resolve()
        environment = {}
        # Match biomcp-test-support::environment::ENV_PATHS, child only.
        for key, directory in [("HOME", "home"), ("USERPROFILE", "home"), ("APPDATA", "config"), ("LOCALAPPDATA", "cache"), ("XDG_CONFIG_HOME", "config"), ("XDG_CACHE_HOME", "cache")]:
            path = root / directory
            path.mkdir(exist_ok=True)
            environment[key] = str(path)
        config = root / "config.json"
        config.write_text(json.dumps({"mcp_tool_mode": "typed", "mcp_reply_form": "both"}))
        command = [str(args.binary.resolve()), "serve", "--config", str(config), "--replay", str(args.replay.resolve()), "--case-cache", str(root / "case-cache")]
        (args.output / "setup.json").write_text(json.dumps({"scratch_root": str(root), "argv": command, "configuration": json.loads(config.read_text())}))
        with (args.output / "server.stderr").open("wb") as errors:
            child = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=errors, env=environment, cwd=root, start_new_session=True)
            try:
                exchange(child, args.output)
                child.wait(timeout=5)
                if child.returncode:
                    raise RuntimeError(f"server exit {child.returncode}")
            finally:
                if child.poll() is None:
                    os.killpg(child.pid, signal.SIGKILL)
                    child.wait()


if __name__ == "__main__":
    main()
