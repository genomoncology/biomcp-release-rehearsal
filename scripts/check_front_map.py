"""Check installed discovery and measure every public command against an installed baseline."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def require(condition, reason):
    # A bare assert prints no reason and disappears under python -O.
    if not condition:
        raise SystemExit(f"front map check failed: {reason}")


def run(binary, args, env):
    result = subprocess.run([str(binary), *args], env=env, capture_output=True, text=True, timeout=30)
    require(result.returncode == 0, f"{args} exited {result.returncode}: {result.stderr or result.stdout}")
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
        help_reply = run(binary, ["help"], env)
        front = json.loads(help_reply)["result"]["commands"][0]["description"]
        require(run(binary, ["--format", "md", "help"], env).startswith(front + "\n\n"),
                "Markdown help does not open with the front map")
        limit = int(os.environ.get("BIOMCP_FRONT_MAP_CHARACTER_LIMIT", 4000))
        require(len(front) <= limit, f"front map is {len(front)} characters, over the limit of {limit}")
        require("--" not in front, "front map names a flag, and it must name commands only")
        rendered = run(binary, ["skill", "render"], env)
        require(rendered.startswith(front + "\n\n"), "rendered skill does not open with the front map")
        source_help = json.loads(run(binary, ["help", "sources"], env))["result"]["commands"][0]["description"]
        require(source_help.startswith("Services:") and "Components:" in source_help,
                "help sources lacks its Services and Components lists")
        trial_help = json.loads(run(binary, ["help", "search", "trial"], env))["result"]["commands"][0]
        country = next((argument for argument in trial_help["arguments"] if argument["name"] == "--country"), None)
        require(country is not None, "search trial help has no --country argument")
        require("Exact ClinicalTrials.gov country label" in country["description"],
                "search trial --country help lost its label description")
        require("biomcp schema " in country["description"], "search trial --country help names no schema to read")
        schema_id = country["description"].rsplit("biomcp schema ", 1)[1].rstrip(".")
        query_schema = json.loads(run(binary, ["schema", schema_id], env))
        complete_choices = query_schema["properties"]["country"]["items"]["enum"]
        require(f"{len(complete_choices)} allowed values." in country["description"],
                f"search trial --country help does not state its {len(complete_choices)} allowed values")
        require("United States" in complete_choices, "country schema lacks United States")
        install = root / "installed"
        run(binary, ["skill", "install", str(install)], env)
        skill = (install / "skills/biomcp/SKILL.md").read_text()
        require(skill.split("\n---\n", 1)[1].lstrip().startswith(front + "\n\n"),
                "installed skill does not open with the front map")
        request = {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-11-25", "capabilities": {},
            "clientInfo": {"name": "front-map-proof", "version": "1"}}}
        process = subprocess.run([str(binary), "serve"], input=json.dumps(request) + "\n",
                                 env=env, capture_output=True, text=True, timeout=30)
        require(process.returncode == 0 and not process.stderr, f"serve failed: {process.stderr}")
        reply = json.loads(process.stdout.splitlines()[0])
        guidance = "Listed choices accept any letter case; replies use their listed spelling."
        require(reply["result"]["instructions"] == front + "\n" + guidance,
                "server instructions differ from the front map and its guidance line")
        (args.output / "help.json").write_text(help_reply)
        (args.output / "front-map.txt").write_text(front)
        (args.output / "initialize.json").write_text(process.stdout)
        version = run(binary, ["version", "--verbose"], env)
        require(not args.revision or args.revision in version, f"binary is not {args.revision}: {version}")
        (args.output / "version.txt").write_text(version)
        (args.output / "SKILL.md").write_text(skill)
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
                require(process.returncode == 0 and not process.stderr, f"{label} serve failed: {process.stderr}")
                tools = json.loads(process.stdout)["result"]["tools"]
                (args.output / f"mcp-{label}.json").write_text(process.stdout)
                tool = next((tool for tool in tools if tool["name"] == "biomcp"), None)
                require(tool is not None, f"{label} tools/list has no biomcp tool")
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
            require(not inventory["removed"], f"commands left help: {inventory['removed']}")
            require(all(path[0] == "help" for path in inventory["added_help_topics"]),
                    f"new paths outside help: {inventory['added_help_topics']}")
            baseline_version = run(args.baseline, ["version", "--verbose"], env)
            require(not args.baseline_revision or args.baseline_revision in baseline_version,
                    f"baseline is not {args.baseline_revision}: {baseline_version}")
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
            require(not args.baseline_sha256 or args.baseline_sha256 == report["baseline_sha256"],
                    f"baseline hash is {report['baseline_sha256']}, not {args.baseline_sha256}")
            for fmt in ("json", "md"):
                rows = [row for row in measurements if row["format"] == fmt]
                before = max(row["baseline_characters"] for row in rows if "baseline_characters" in row)
                after = max(row["characters"] for row in rows)
                report[fmt] = {"longest_before": before, "longest_after": after, "reduction": 1 - after / before}
        (args.output / "proof.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({key: value for key, value in report.items() if key not in {"measurements", "inventory"}}))
        if args.baseline:
            for fmt in ("json", "md"):
                require(report[fmt]["reduction"] >= 0.40, f"{fmt} help shrank less than 40%: {report[fmt]}")


if __name__ == "__main__":
    main()
