"""Refresh existing help metadata snapshots after an intentional help presentation change."""
import json
from pathlib import Path
import subprocess
import sys


def pointer(value, path):
    for part in path.lstrip("/").split("/"):
        value = value[int(part)] if isinstance(value, list) else value.get(part)
        if value is None:
            return None
    return value


def main():
    binary = sys.argv[1]
    shared_help = json.loads(subprocess.check_output([binary, "help", "help"], text=True))
    shared_names = {
        argument["name"]
        for command in shared_help["result"]["commands"]
        if command.get("command") == ["help"]
        for argument in command.get("arguments", [])
    }
    for path in sorted(Path("tests/cases").rglob("*.yaml")):
        try:
            case = json.loads(path.read_text())
        except ValueError:
            continue
        changed = False
        for step in case.get("steps", []):
            if step.get("run", [None])[0] != "help" or step.get("exit", 0) != 0:
                continue
            output = json.loads(subprocess.check_output([binary, *step["run"]], text=True))
            for key, expected in list(step.get("expect", {}).items()):
                if (key.endswith("/description") and isinstance(expected, str)) or key.endswith("/examples"):
                    actual = pointer(output, key)
                    if isinstance(expected, dict) and "count" in expected:
                        actual = {"count": len(actual or [])}
                    if actual != expected:
                        step["expect"][key] = actual
                        changed = True
                if key.endswith("/arguments") and expected == {"count": 6} and pointer(output, key) is None:
                    del step["expect"][key]
                    step["expect"][key.rsplit("/", 1)[0]] = {"omits": ["arguments"]}
                    changed = True
                if key.endswith("/arguments/*/name"):
                    names = expected if isinstance(expected, list) else (
                        expected.get("includes_all_of") if isinstance(expected, dict) else None)
                    if not isinstance(names, list):
                        continue
                    arguments_path = key.removesuffix("/*/name")
                    actual = pointer(output, arguments_path)
                    actual_names = {argument["name"] for argument in actual or []}
                    trimmed = [word for word in names
                               if word not in shared_names or word in actual_names]
                    if trimmed == names:
                        continue
                    if trimmed:
                        step["expect"][key] = trimmed if isinstance(expected, list) else {
                            **expected, "includes_all_of": trimmed}
                    else:
                        del step["expect"][key]
                        if actual is None:
                            parent = arguments_path.rsplit("/", 1)[0]
                            assertion = step["expect"].setdefault(parent, {})
                            assertion.setdefault("omits", [])
                            if "arguments" not in assertion["omits"]:
                                assertion["omits"].append("arguments")
                    changed = True
        if changed:
            path.write_text(json.dumps(case, indent=2, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
