"""Inspect the evaluation's research fetch through both public MCP modes."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from release.artifacts import cargo
from release.smoke import environment


def main():
    binary = cargo("dev")["executable"]
    output = Path("target/research-fetch")
    output.mkdir(parents=True, exist_ok=True)
    for mode in ("single", "typed"):
        frames = [
            {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
                "protocolVersion": "2025-11-25", "capabilities": {},
                "clientInfo": {"name": "research-fetch-check", "version": "1"}}},
            {"jsonrpc": "2.0", "method": "notifications/initialized"},
            {"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
                "name": "fetch", "arguments": {"id": "article 39214094"}}},
        ]
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run([str(binary), "serve", "--tool-mode", mode],
                                    input="".join(json.dumps(row) + "\n" for row in frames),
                                    capture_output=True, text=True, env=environment(directory), timeout=120)
        (output / (mode + ".jsonl")).write_text(result.stdout)
        if result.returncode or result.stderr:
            raise ValueError("research fetch process failed: " + result.stderr)
        reply = next(json.loads(row)["result"] for row in result.stdout.splitlines()
                     if json.loads(row).get("id") == 2)
        record = reply.get("structuredContent") or json.loads(reply["content"][0]["text"])
        text = record.get("text", "")
        print(json.dumps({"mode": mode, "isError": reply.get("isError"),
                          "id": record.get("id"), "title": record.get("title"),
                          "escaped_period": "&#46;" in text,
                          "sup_tag": "<sup" in text or "&lt;sup" in text,
                          "duplicate_record": "result" in record.get("metadata", {}),
                          "retained": str(output / (mode + ".jsonl"))}), flush=True)


if __name__ == "__main__":
    main()
