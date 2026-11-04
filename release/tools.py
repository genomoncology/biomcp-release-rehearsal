"""Check source tool names and read candidate descriptions from tools/list."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import tempfile


def declared(root):
    protocol = (root / "crates/biomcp/src/mcp/protocol.rs").read_text()
    research = (root / "crates/biomcp/src/mcp/research.rs").read_text()
    primary = protocol.split("pub(super) fn tool(", 1)[1].split("pub(super) fn tools(", 1)[0]
    projections = research.split("pub(super) fn tools(", 1)[1].split("#[derive", 1)[0]
    assembly = protocol.split("pub(super) fn tools(", 1)[1].split("pub(super) fn result", 1)[0]
    if re.sub(r"\s+", "", assembly) != "root:&crate::grammar::Root,read_only:bool)->std::io::Result<Vec<Value>>{letmuttools=vec![tool(root,read_only)?];tools.extend(super::research::tools(root)?);Ok(tools)}":
        raise ValueError("unrecognized server tool assembly")
    names = re.findall(r'"name"\s*:\s*"([a-z][a-z0-9_-]*)"', primary)
    rows = re.fullmatch(r'\s*root:\s*&crate::grammar::Root\)\s*->\s*io::Result<Vec<Value>>\s*\{\s*\[(.*?)\]\s*\.into_iter\(\)\s*\.map\(\|\(name,\s*key,\s*description\)\|\s*\{.*\}\)\s*\.collect\(\)\s*\}\s*', projections, re.S)
    string = r'"(?:[^"\\]|\\[\s\S])*"'
    row = r'\(\s*(' + string + r')\s*,\s*' + string + r'\s*,\s*' + string + r'\s*,?\s*\)'
    if len(names) != 1 or not rows or not re.fullmatch(r'\s*' + row + r'(?:\s*,\s*' + row + r')*\s*,?\s*', rows[1]):
        raise ValueError("unrecognized server tool declarations")
    names += [json.loads(match[1]) for match in re.finditer(row, rows[1])]
    if len(names) != len(set(names)) or any(not re.fullmatch(r'[a-z][a-z0-9_-]*', name) for name in names):
        raise ValueError("server tool declarations are missing or duplicated")
    return sorted(names)


def check(root):
    names = [tool["name"] for tool in json.loads((root / "manifest.json").read_text())["tools"]]
    if sorted(names) != declared(root):
        raise ValueError("desktop manifest tools differ from server source declarations")


def candidate(binary):
    # Reuse the installed gate's isolated environment; tools/list makes no source calls.
    if __package__:
        from .smoke import environment
    else:
        from smoke import environment
    frames = [
        {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "bundle-producer", "version": "1"}}},
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
    ]
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory).resolve()
        result = subprocess.run([str(binary.resolve()), "serve"], input="".join(json.dumps(frame) + "\n" for frame in frames), cwd=root, env=environment(root), capture_output=True, text=True, timeout=60, check=True)
    replies = [json.loads(line) for line in result.stdout.splitlines()]
    if result.stderr or len(replies) != 2 or [reply.get("id") for reply in replies] != [1, 2]:
        raise ValueError("candidate tool discovery failed")
    return [{"name": tool["name"], "description": tool["description"]} for tool in replies[1]["result"]["tools"]]


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        check(args.root)
    except (ValueError, KeyError, IndexError, OSError) as error:
        parser.exit(1, str(error) + "\n")
    print("Desktop manifest tools match server source declarations")
