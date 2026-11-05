"""Read-only reconstructed evaluation calls; preserve both MCP modes' actual replies."""
import json
import os
from pathlib import Path
import subprocess
import tempfile


def exchange(process, request):
    process.stdin.write(json.dumps(request) + "\n")
    process.stdin.flush()
    while line := process.stdout.readline():
        reply = json.loads(line)
        if reply.get("id") == request["id"]:
            return reply
    raise RuntimeError("MCP closed before its response")


def main():
    executable = Path("target/debug/biomcp").resolve()
    output = Path("dist/drug-alias-baseline")
    output.mkdir(parents=True, exist_ok=True)
    for mode in ["single", "typed"]:
        with tempfile.TemporaryDirectory(prefix="biomcp-drug-alias-") as folder:
            home = Path(folder)
            config = home / "config.json"
            config.write_text(json.dumps({"mcp_tool_mode": mode}))
            environment = {"PATH": os.environ["PATH"], "HOME": folder,
                           "XDG_CONFIG_HOME": folder, "XDG_CACHE_HOME": folder}
            with (output / f"{mode}.stderr").open("w") as errors:
                process = subprocess.Popen(
                    [str(executable), "--config", str(config), "serve"],
                    env=environment, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                    stderr=errors, text=True)
                try:
                    exchange(process, {"jsonrpc": "2.0", "id": 1, "method": "initialize",
                                       "params": {"protocolVersion": "2025-11-25", "capabilities": {},
                                                  "clientInfo": {"name": "drug-alias-probe", "version": "1"}}})
                    process.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
                    for index, (name, source) in enumerate(
                            [(name, source) for name in ["daraxonrasib", "RMC-6236"]
                             for source in ["chembl", "opentargets"]], 2):
                        arguments = {"query": name, "source": source}
                        call = {"name": "search_drug", "arguments": arguments} if mode == "typed" else {
                            "name": "biomcp", "arguments": {"args": ["search", "drug", name, "--source", source]}}
                        reply = exchange(process, {"jsonrpc": "2.0", "id": index,
                                                   "method": "tools/call", "params": call})
                        (output / f"{mode}-{name}-{source}.json").write_text(json.dumps(reply, indent=2) + "\n")
                        result = reply.get("result", {})
                        body = result.get("structuredContent", {})
                        print(json.dumps({"mode": mode, "name": name, "source": source,
                                          "isError": result.get("isError"), "status": body.get("status"),
                                          "page": body.get("page"), "sources": body.get("sources")}), flush=True)
                finally:
                    process.terminate()
                    process.communicate(timeout=10)


if __name__ == "__main__":
    main()
