#!/usr/bin/env python3
"""Compare optional case-runner transcripts. Print at most 20 differences."""
import json
from pathlib import Path
import re
import shlex
import sys
from collections import Counter

from reply_runtime import Bindings, align, context_roots, owner, pair


def capture_kind(execution):
    if not isinstance(execution, dict):
        raise ValueError("invalid execution shape")
    keys = set(execution)
    required = {"args", "stdout", "exit", "surface"}
    metadata = {"root", "checkout", "version", "executable", "sequence_start", "sequence_end", "replay_owner_root"}
    if required <= keys <= required | metadata and execution["surface"] in ("cli", "mcp"):
        return "process"
    if keys == {"args", "surface", "http_status", "supplied_header", "extra_argument", "reply"} and execution["surface"] == "http":
        return "http"
    if keys == {"surface", "keyed_reply", "unkeyed_reply"} and execution["surface"] == "http-concurrent":
        return "http-concurrent"
    raise ValueError("unknown execution shape")


def load(directory):
    records = {}
    uncaptured = []
    native = {}
    structured = {}
    seen = set()
    for path in sorted(Path(directory).rglob("*.json")):
        if path.name.startswith("._"):
            continue
        record = json.loads(path.read_text())
        if record.get("kind") == "trial":
            if record["uncaptured"]:
                uncaptured.append(record["name"])
            continue
        key = record["key"]
        if not isinstance(key, str) or key in seen:
            raise ValueError("duplicate or invalid transcript key")
        seen.add(key)
        # Context::record stores native wire request pairs separately from executions.
        if (isinstance(key, str) and key.startswith("cache/") and key.endswith("/requests")
                and len(key.split("/")) == 4
                and all(part not in ("", ".", "..") for part in key.split("/"))):
            executions = record.get("executions")
            if (set(record) != {"key", "executions"} or not isinstance(executions, list)
                    or not all(isinstance(batch, list) and all(
                        isinstance(pair, list) and len(pair) == 2
                        and all(isinstance(value, str) for value in pair)
                        for pair in batch) for batch in executions)):
                raise ValueError("invalid native request metadata")
            if key in native:
                raise ValueError(f"duplicate native request key: {key}")
            native[key] = record
            continue
        owner(key)
        executions = record.get("executions")
        if not isinstance(executions, list) or not executions:
            raise ValueError("invalid executions")
        kinds = {capture_kind(execution) for execution in executions}
        if len(kinds) != 1:
            raise ValueError("mixed execution shapes")
        (records if kinds == {"process"} else structured)[key] = record
    if not records and not structured and not uncaptured:
        raise ValueError(f"no transcripts: {directory}")
    return records, uncaptured, native, structured


def normalize(record):
    text = bytes(record["stdout"]).decode("utf-8", errors="surrogateescape")
    args = record["args"]
    if args[:1] in (["version"], ["--version"]):
        # Only the version command's identity line names the package version.
        version = re.escape(record.get("version", ""))
        text = re.sub(r"(?m)^biomcp " + version + r"$", "biomcp <version>", text)
        if "--verbose" in args:
            text = re.sub(r"(?m)^Commit: [^\n]*", "Commit: <commit>", text)
            # Calibration: isolated config/cache locations vary between runs.
            root = re.escape(record.get("root", ""))
            text = re.sub(r"(?m)^(Config|Cache): " + root + r"(?=/|$)",
                          lambda m: m[1] + ": <root>", text)
    if args[:1] == ["mcp-config"]:
        # mcp-config owns its command field and its complete Codex launch line.
        executable = record.get("executable", "")
        text = re.sub(r'("command"\s*:\s*)' + re.escape(json.dumps(executable)),
                      lambda m: m[1] + '"<executable>"', text)
        line = "codex mcp add biomcp -- " + shlex.quote(executable) + " serve\n"
        if text == line:
            text = "codex mcp add biomcp -- <executable> serve\n"
        # A copied installation may select an owned stable symlink instead.
        root = record.get("root")
        if root:
            text = re.sub(r'("command"\s*:\s*")' + re.escape(json.dumps(root)[1:-1]) + r'(?=/|")',
                          lambda m: m[1] + "<root>", text)
            text = re.sub(r"(?m)^(codex mcp add biomcp -- '?)" + re.escape(root) + r"(?=/|$)",
                          lambda m: m[1] + "<root>", text)
    # Calibration: runner-owned argument paths vary, including JSON request tokens.
    for field in ("root", "checkout"):
        value = record.get(field)
        if value:
            for prefix in {value, json.dumps(value)[1:-1]}:
                pattern = r'(^|")' + re.escape(prefix) + r'(?=/|$|")'
                args = [re.sub(pattern, lambda m: m[1] + f"<{field}>", a) for a in args]
    return {"args": args, "surface": record["surface"], "exit": record["exit"], "stdout": text}



def pointer(left, right, path=""):
    if type(left) is not type(right):
        return path or "/"
    if isinstance(left, dict):
        for key in sorted(left.keys() | right.keys()):
            child = path + "/" + key.replace("~", "~0").replace("/", "~1")
            if key not in left or key not in right:
                return child
            difference = pointer(left[key], right[key], child)
            if difference:
                return difference
    elif isinstance(left, list):
        for index, (before, after) in enumerate(zip(left, right)):
            difference = pointer(before, after, path + f"/{index}")
            if difference:
                return difference
        if len(left) != len(right):
            return path + f"/{min(len(left), len(right))}"
    elif left != right:
        if path.endswith("/stdout"):
            try:
                difference = pointer(json.loads(left), json.loads(right), path)
                if difference:
                    return difference
            except (ValueError, TypeError):
                pass
            index = next((i for i, pair in enumerate(zip(left, right))
                          if pair[0] != pair[1]), min(len(left), len(right)))
            return path + f"/@character:{index}"
        return path or "/"
    return None


def fields(left, right, path=""):
    """Enumerate every delta, parsing JSON replies while retaining raw text changes."""
    if type(left) is type(right) and isinstance(left, dict):
        for key in sorted(left.keys() | right.keys()):
            child = path + "/" + key.replace("~", "~0").replace("/", "~1")
            if key not in left or key not in right:
                yield {"path": child, "before": left.get(key), "after": right.get(key),
                       "missing": "before" if key not in left else "after"}
            else:
                yield from fields(left[key], right[key], child)
    elif type(left) is type(right) and isinstance(left, list):
        for index in range(max(len(left), len(right))):
            child = path + f"/{index}"
            if index >= min(len(left), len(right)):
                yield {"path": child, "missing": "before" if index >= len(left) else "after"}
            else:
                yield from fields(left[index], right[index], child)
    elif type(left) is not type(right) or left != right:
        if path.endswith("/stdout") and isinstance(left, str) and isinstance(right, str):
            try:
                before, after = json.loads(left), json.loads(right)
            except (ValueError, TypeError):
                pass
            else:
                if pointer(before, after):
                    yield from fields(before, after, path)
                    return
        yield {"path": path or "/", "before": left, "after": right}


def show(key, before, after, location, count, full):
    if full:
        changes = fields(before, after) if before is not None and after is not None else [{"path": location}]
        for change in changes:
            print(json.dumps({"key": key, **change}, ensure_ascii=False, sort_keys=True))
    elif count <= 20:
        print(f"{key}: {location}")


def main():
    arguments = sys.argv[1:]
    full = "--all-fields" in arguments
    if full:
        arguments.remove("--all-fields")
    if len(arguments) != 2:
        print("usage: reply_diff.py [--all-fields] BASE HEAD", file=sys.stderr)
        return 2
    try:
        (base, base_missing, base_native, base_structured), (head, head_missing, head_native, head_structured) = (load(d) for d in arguments)
        for label, processes, structured, native in (
                ("BASE", base, base_structured, base_native),
                ("HEAD", head, head_structured, head_native)):
            counts = Counter()
            records = Counter()
            for record in [*processes.values(), *structured.values()]:
                kind = capture_kind(record["executions"][0])
                records[kind] += 1
                counts[kind] += len(record["executions"])
            counts["native-request-batches"] = sum(len(r["executions"]) for r in native.values())
            records["native-request-batches"] = len(native)
            print(f"{label} captures: " + ", ".join(
                f"{k}={records[k]} records/{v} executions" for k, v in sorted(counts.items())), file=sys.stderr)
        if base.keys() & head_structured.keys() or head.keys() & base_structured.keys():
            raise ValueError("execution category changed between captures")
        for label, missing in (("BASE", base_missing), ("HEAD", head_missing)):
            if missing:
                print(f"{label} uncaptured trials ({len(missing)}):", file=sys.stderr)
                for name in sorted(missing):
                    print(f"  {name}", file=sys.stderr)
        if not (base or base_structured) or not (head or head_structured):
            raise ValueError("comparison requires captured steps in both sets")
        bindings = Bindings()
        roots, aliases = align(base, head, normalize, bindings)
        paired = {}
        for key in sorted(base.keys() & head.keys() | aliases.keys()):
            target = aliases.get(key, key)
            a, b = base[key]["executions"], head[target]["executions"]
            # Collect every declaration before comparing references and Markdown.
            for i, (x, y) in enumerate(zip(a, b)):
                declared = [context_roots(r, k, i) for r, k in zip(roots, (key, target))]
                pair(x, y, owner(key), bindings, declared, normalize)
            paired[key] = (a, b)
        differences = 0
        for key in sorted(base_structured.keys() | head_structured.keys()):
            before, after = base_structured.get(key), head_structured.get(key)
            if key not in base_structured or key not in head_structured:
                location = "/ missing in " + ("HEAD" if key in base_structured else "BASE")
            else:
                before, after = base_structured[key], head_structured[key]
                if capture_kind(before["executions"][0]) != capture_kind(after["executions"][0]):
                    raise ValueError("structured execution category changed")
                location = pointer(before, after)
            if location:
                differences += 1
                show(key, before, after, location, differences, full)
        for key in sorted(base_native.keys() | head_native.keys()):
            before, after = base_native.get(key), head_native.get(key)
            location = ("/ missing in " + ("HEAD" if key in base_native else "BASE")
                        if key not in base_native or key not in head_native
                        else pointer(base_native[key], head_native[key]))
            if location:
                differences += 1
                show(key, before, after, location, differences, full)
        for key in sorted(base.keys() | (head.keys() - set(aliases.values()))):
            before, after = (None, None)
            if key in paired:
                before, after = [list(values) for values in zip(*[
                    pair(x, y, owner(key), bindings,
                         [context_roots(r, k, i) for r, k in zip(roots, (key, aliases.get(key, key)))],
                         normalize)
                    for i, (x, y) in enumerate(zip(*paired[key]))])]
            location = ("/ missing in " + ("HEAD" if key in base else "BASE")
                        if key not in paired else pointer(before, after))
            if key in paired and len(paired[key][0]) != len(paired[key][1]):
                location = "/executions/length"
                if full:
                    print(json.dumps({"key": key, "path": location,
                                      "before": len(paired[key][0]), "after": len(paired[key][1])}, sort_keys=True))
            if location:
                differences += 1
                show(key, before, after, location, differences, full)
        if differences > 20 and not full:
            print(f"{differences - 20} more differing keys")
        return int(differences > 0)
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"reply_diff: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
