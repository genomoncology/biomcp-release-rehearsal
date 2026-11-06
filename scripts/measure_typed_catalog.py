"""Measure compact UTF-8 tools/list definitions from isolated installed servers."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import sys


def encoded(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode()


def main():
    if len(sys.argv) == 1:
        subprocess.run(["cargo", "+1.95.0", "build", "--locked", "-p", "biomcp", "--bin", "biomcp"], check=True)
        binary = Path(os.environ.get("CARGO_TARGET_DIR", "target")) / "debug/biomcp"
    else:
        binary = Path(sys.argv[1])
    binary = binary.resolve()
    output = Path(sys.argv[2]) if len(sys.argv) > 2 else Path("target/typed-catalog-measurement")
    output.mkdir(parents=True, exist_ok=True)
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    (output / "binding.json").write_bytes(encoded({"revision": revision,
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "command": "serve --offline; isolated tools/list; compact UTF-8 result.tools"}))
    frame = {"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {"_meta": {
        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientCapabilities": {}}}}
    profiles = [("single", [], {})]
    for hosted in (False, True):
        base = "hosted-typed" if hosted else "typed"
        flags = ["--tool-mode", "typed"] + (["--hosted"] if hosted else [])
        profiles.append((base, flags, {}))
        for schema in (False, True):
            for limit in (1, 30, 1000):
                config = {"mcp_typed_output_schema": schema, "mcp_typed_enum_limit": limit}
                profiles.append((f"{base}-output-{schema}-enum-{limit}", flags, config))
    profiles.append(("typed-excluded", ["--tool-mode", "typed"],
                     {"disable_sources": ["ctgov", "who-pq", "gwas"]}))
    measurements = []
    with tempfile.TemporaryDirectory() as scratch:
        env = {key: value for key, value in os.environ.items()
               if key in {"PATH", "SystemRoot", "WINDIR", "TMPDIR", "TEMP", "TMP"}}
        env.update(HOME=scratch, XDG_CONFIG_HOME=scratch, XDG_CACHE_HOME=scratch, APPDATA=scratch)
        config_path = Path(scratch) / "profile.json"
        for label, flags, config in profiles:
            config_path.write_bytes(encoded(config))
            result = subprocess.run([str(binary), "serve", "--offline", "--config", str(config_path), *flags],
                                    input=json.dumps(frame) + "\n", env=env,
                                    capture_output=True, text=True, timeout=30, check=True)
            (output / (label + ".reply.json")).write_text(result.stdout)
            (output / (label + ".config.json")).write_bytes(encoded(config))
            reply = json.loads(result.stdout)
            if "error" in reply:
                raise RuntimeError(f"{label}: {reply['error']}")
            tools = reply["result"]["tools"]
            (output / (label + ".tools.json")).write_bytes(encoded(tools))
            row = {"mode": label, "revision": revision, "config": config,
                   "tool_count": len(tools), "definition_bytes": len(encoded(tools)),
                   "tools": [{"name": tool["name"], "bytes": len(encoded(tool)),
                              "description_bytes": len(tool.get("description", "").encode())}
                             for tool in tools]}
            measurements.append(row)
            print(json.dumps(row, separators=(",", ":")))
    (output / "measurements.json").write_bytes(encoded(measurements))


if __name__ == "__main__":
    main()
