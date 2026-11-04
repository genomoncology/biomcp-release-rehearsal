"""Check installed discovery and measure every public command against an installed baseline."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def run(binary, args, env):
    result = subprocess.run([str(binary), *args], env=env, capture_output=True, text=True, timeout=30)
    assert result.returncode == 0, (args, result.stdout, result.stderr)
    return result.stdout


def discover(binary, env):
    pending = [tuple(row.get("command", [])) for row in
               json.loads(run(binary, ["help"], env))["result"]["commands"]]
    visited, leaves = set(), set()
    while pending:
        path = pending.pop()
        if not path or path in visited:
            continue
        visited.add(path)
        output = json.loads(run(binary, ["help", *path], env))
        children = [tuple(row.get("command", [])) for row in output["result"].get("commands", [])]
        deeper = [child for child in children if child[:len(path)] == path and len(child) > len(path)]
        if deeper:
            pending.extend(deeper)
        else:
            leaves.add(path)
    return leaves


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--mcp-baseline", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--revision")
    parser.add_argument("--baseline-revision")
    parser.add_argument("--baseline-sha256")
    args = parser.parse_args()
    with tempfile.TemporaryDirectory() as scratch:
        root = Path(scratch)
        args.output = args.output or root / "proof"
        args.output.mkdir(parents=True, exist_ok=True)
        env = {key: value for key, value in os.environ.items()
               if key in {"PATH", "SystemRoot", "WINDIR", "TMPDIR", "TEMP", "TMP"}}
        env.update(HOME=str(root), XDG_CONFIG_HOME=str(root / "config"),
                   XDG_CACHE_HOME=str(root / "cache"), APPDATA=str(root / "config"))
        binary = root / "bin" / "biomcp"
        binary.parent.mkdir()
        shutil.copy2(args.binary, binary)
        front = json.loads(run(binary, ["help"], env))["result"]["commands"][0]["description"]
        assert run(binary, ["--format", "md", "help"], env).startswith(front + "\n\n")
        assert len(front) <= int(os.environ.get("BIOMCP_FRONT_MAP_CHARACTER_LIMIT", 4000))
        assert "--" not in front
        rendered = run(binary, ["skill", "render"], env)
        assert rendered.startswith(front + "\n\n")
        source_help = json.loads(run(binary, ["help", "sources"], env))["result"]["commands"][0]["description"]
        assert source_help.startswith("Services:") and "Components:" in source_help
        trial_help = json.loads(run(binary, ["help", "search", "trial"], env))["result"]["commands"][0]
        country = next(argument for argument in trial_help["arguments"] if argument["name"] == "--country")
        assert "Exact ClinicalTrials.gov country label" in country["description"]
        schema_id = country["description"].rsplit("biomcp schema ", 1)[1].rstrip(".")
        query_schema = json.loads(run(binary, ["schema", schema_id], env))
        complete_choices = query_schema["properties"]["country"]["items"]["enum"]
        assert country["description"].startswith(str(len(complete_choices)) + " allowed values;")
        assert "United States" in complete_choices
        install = root / "installed"
        run(binary, ["skill", "install", str(install)], env)
        skill = (install / "skills/biomcp/SKILL.md").read_text()
        assert skill.split("\n---\n", 1)[1].lstrip().startswith(front + "\n\n")
        request = {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-11-25", "capabilities": {},
            "clientInfo": {"name": "front-map-proof", "version": "1"}}}
        process = subprocess.run([str(binary), "serve"], input=json.dumps(request) + "\n",
                                 env=env, capture_output=True, text=True, timeout=30)
        assert process.returncode == 0 and not process.stderr, process.stderr
        reply = json.loads(process.stdout.splitlines()[0])
        assert reply["result"]["instructions"] == front
        (args.output / "front-map.txt").write_text(front)
        (args.output / "initialize.json").write_text(process.stdout)
        version = run(binary, ["version", "--verbose"], env)
        assert not args.revision or args.revision in version, version
        (args.output / "version.txt").write_text(version)
        (args.output / "SKILL.md").write_text(skill)
        if args.baseline:
            retained = args.output / "installed/bin/biomcp"
            retained.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(binary, retained)
        if args.mcp_baseline:
            mcp_measurements = {}
            for label, executable in (("before", args.mcp_baseline), ("after", binary)):
                frame = {"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {"_meta": {
                    "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                    "io.modelcontextprotocol/clientCapabilities": {}}}}
                process = subprocess.run([str(executable), "serve", "--offline"], input=json.dumps(frame) + "\n",
                                         env=env, capture_output=True, text=True, timeout=30)
                assert process.returncode == 0 and not process.stderr, process.stderr
                tools = json.loads(process.stdout)["result"]["tools"]
                (args.output / f"mcp-{label}.json").write_text(process.stdout)
                tool = next(tool for tool in tools if tool["name"] == "biomcp")
                mcp_measurements[label] = {}
                for field in ("description", "inputSchema", "outputSchema"):
                    value = tool[field] if field == "description" else json.dumps(tool[field], ensure_ascii=False, separators=(",", ":"))
                    mcp_measurements[label][field] = {"bytes": len(value.encode()), "characters": len(value)}
            (args.output / "mcp-sizes.json").write_text(json.dumps(mcp_measurements, indent=2) + "\n")
            print(json.dumps({"mcp_sizes": mcp_measurements}))
        raw = args.output / "help"
        raw.mkdir(exist_ok=True)
        measurements = []
        inventory = {}
        if args.baseline:
            current_paths, baseline_paths = discover(binary, env), discover(args.baseline, env)
            inventory = {"candidate": sorted(current_paths), "baseline": sorted(baseline_paths),
                         "added_help_topics": sorted(current_paths - baseline_paths),
                         "removed": sorted(baseline_paths - current_paths)}
            assert not inventory["removed"], inventory
            assert all(path[0] == "help" for path in inventory["added_help_topics"]), inventory
            baseline_version = run(args.baseline, ["version", "--verbose"], env)
            assert not args.baseline_revision or args.baseline_revision in baseline_version, baseline_version
            (args.output / "baseline-version.txt").write_text(baseline_version)
        else:
            current_paths, baseline_paths = set(), set()
        for path in sorted(current_paths | baseline_paths):
            for fmt in ("json", "md"):
                command = ["--format", fmt, "help", *path]
                current = run(binary, command, env)
                name = "-".join(path) + "." + fmt
                (raw / ("candidate-" + name)).write_text(current)
                record = {"command": list(path), "format": fmt, "characters": len(current)}
                if path in baseline_paths:
                    baseline = run(args.baseline, command, env)
                    record["baseline_characters"] = len(baseline)
                    (raw / ("baseline-" + name)).write_text(baseline)
                measurements.append(record)
        report = {"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                  "map_characters": len(front), "map_bytes": len(front.encode()),
                  "inventory": inventory, "measurements": measurements}
        if args.baseline:
            report["baseline_sha256"] = hashlib.sha256(args.baseline.read_bytes()).hexdigest()
            assert not args.baseline_sha256 or args.baseline_sha256 == report["baseline_sha256"]
            for fmt in ("json", "md"):
                rows = [row for row in measurements if row["format"] == fmt]
                before = max(row["baseline_characters"] for row in rows if "baseline_characters" in row)
                after = max(row["characters"] for row in rows)
                report[fmt] = {"longest_before": before, "longest_after": after, "reduction": 1 - after / before}
        (args.output / "proof.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({key: value for key, value in report.items() if key not in {"measurements", "inventory"}}))
        if args.baseline:
            assert report["json"]["reduction"] >= 0.40, report["json"]
            assert report["md"]["reduction"] >= 0.40, report["md"]


if __name__ == "__main__":
    main()
