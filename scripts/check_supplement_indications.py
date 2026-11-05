"""Inspect source-owned approval supplement text through both public tool modes."""
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
    output = Path("target/supplement-indications").resolve()
    output.mkdir(parents=True, exist_ok=True)
    if sys.argv[1:] == ["letter"]:
        return letter(binary, output)
    recording = output / "recordings"
    for mode in ("single", "typed"):
        arguments = {"input": "NDA208692", "section": ["approvals"],
                     "source": "drugsfda", "section_limit": 50}
        call = {"name": "get_drug", "arguments": arguments}
        if mode == "single":
            call = {"name": "biomcp", "arguments": {"args": [
                "get", "drug", "NDA208692", "approvals", "--source", "drugsfda",
                "--section-limit", "50"]}}
        frames = [
            {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
                "protocolVersion": "2025-11-25", "capabilities": {},
                "clientInfo": {"name": "supplement-indications-check", "version": "1"}}},
            {"jsonrpc": "2.0", "method": "notifications/initialized"},
            {"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": call},
        ]
        globals_ = ["--record", str(recording)] if mode == "single" else ["--replay", str(recording)]
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run([str(binary), "serve", "--tool-mode", mode, *globals_],
                                    input="".join(json.dumps(row) + "\n" for row in frames),
                                    capture_output=True, text=True, env=environment(directory), timeout=90)
        (output / (mode + ".jsonl")).write_text(result.stdout)
        if result.returncode or result.stderr:
            raise ValueError("approval supplement process failed: " + result.stderr)
        reply = next(json.loads(row)["result"] for row in result.stdout.splitlines()
                     if json.loads(row).get("id") == 2)
        value = reply.get("structuredContent") or json.loads(reply["content"][0]["text"])
        rows = value.get("result", {}).get("approvals", {}).get("data", {}).get("records", [])
        print(json.dumps({"mode": mode, "isError": reply.get("isError"),
                          "rows": len(rows), "order": value.get("result", {}).get("order"),
                          "efficacy": [{"number": row["submission_number"],
                                        "indication": row["event"].get("indication"),
                                        "disclosure": row.get("supplement_purpose")}
                                       for row in rows if row.get("submission_class_code") == "EFFICACY"],
                          "retained": str(output / (mode + ".jsonl"))}), flush=True)
        if reply.get("isError") or not rows:
            raise ValueError("approval reproduction returned no records")


def letter(binary, output):
    with tempfile.TemporaryDirectory() as directory:
        result = subprocess.run([str(binary), "get", "drug", "NDA208692", "approvals",
                                 "--source", "drugsfda", "--document-id", "81763",
                                 "--record", str(output / "letter-recordings")],
                                env=environment(directory), capture_output=True,
                                text=True, timeout=90)
    (output / "letter.json").write_text(result.stdout)
    (output / "letter.stderr").write_text(result.stderr)
    value = json.loads(result.stdout)
    rows = value.get("result", {}).get("approvals", {}).get("data", {}).get("records", [])
    document = next((doc for row in rows for doc in row.get("application_docs", [])
                     if doc.get("id") == "81763"), {})
    print(json.dumps({"check": "selected-native-letter-81763", "exit": result.returncode,
                      "status": value.get("status"),
                      "sources": [{key: source.get(key) for key in ("source", "state", "reason")}
                                  for source in value.get("sources", [])],
                      "document_text_state": document.get("text", {}).get("state"),
                      "retained": str(output / "letter.json")}), flush=True)
    if result.returncode not in (0, 4):
        raise ValueError("selected letter command had a local or usage failure")


if __name__ == "__main__":
    main()
