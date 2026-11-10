"""Check documentation declarations against the checked-out tree."""
import argparse
import hashlib
import re
import subprocess
import sys
import json
import shlex
from pathlib import Path

from parity import LEDGER_END, LEDGER_START, UPGRADE_END, UPGRADE_START, rows, table, upgrade


def architecture(root):
    text = (root / "sdlc/planning/architecture.md").read_text()
    sections = re.split(r"(?m)^## ", text)[1:]
    if not sections:
        raise ValueError("architecture has no rule sections")
    for section in sections:
        checks = re.findall(r"Check: `([^`]+)`", section)
        if not checks:
            raise ValueError(f"architecture section lacks a check: {section.splitlines()[0]}")
        for check in checks:
            path = Path(check)
            if path.is_absolute() or ".." in path.parts or not (root / path).is_file():
                raise ValueError(f"architecture names a missing check file: {check}")


# Internal machine, account and repository names must stay out of public files.
# They are stored as SHA-256 prefixes of the lowercase word so this check does not publish them.
INTERNAL_WORDS = {"e712a02b83a65ca2", "2572e799420e05c9", "a1fb86a4c024c04c", "080baa17c90b5e97"}
# Common words that name an internal machine only as an SSH or copy host, such as HOST:/path.
INTERNAL_HOSTS = INTERNAL_WORDS | {"b5f2031eb62e37c6", "c685a2c9bab235cc", "16477688c0e00699", "1b4c9133da73a711"}
INTERNAL_USERS = {"b54a95127a4b573f", "a1fb86a4c024c04c"}
# Third-party source data and planning records are reviewed separately.
INTERNAL_SKIP = ("sdlc/", "tests/replies/", "tests/fixtures/")


def digest(word):
    return hashlib.sha256(word.lower().encode()).hexdigest()[:16]


def public_files(root):
    listing = subprocess.run(["git", "-C", str(root), "ls-files", "-z"], capture_output=True, check=False)
    if listing.returncode == 0:
        names = [name for name in listing.stdout.decode().split("\0") if name]
    else:
        names = [path.relative_to(root).as_posix() for path in root.rglob("*") if path.is_file()]
    return [name for name in names if not name.startswith(INTERNAL_SKIP)]


def internal_names(root):
    for name in public_files(root):
        try:
            text = (root / name).read_text()
        except (UnicodeDecodeError, OSError):
            continue
        for number, line in enumerate(text.splitlines(), 1):
            found = (
                any(digest(word) in INTERNAL_WORDS for word in re.findall(r"[A-Za-z0-9]+", line))
                or any(digest(host) in INTERNAL_HOSTS
                       for host in re.findall(r"(?<![\w./-])([A-Za-z0-9-]+)(?:\.local)?:[/~$]", line))
                or any(digest(user) in INTERNAL_USERS
                       for user in re.findall(r"(?<![\w.])/(?:home|Users)/([A-Za-z0-9._-]+)", line))
                or re.search(r"(?:~|\$HOME|\$\{HOME\})/workspace/", line)
            )
            if found:
                raise ValueError(f"{name}:{number}: internal machine name or workspace path in a public file")


# MCP tools through 0.9 and the 1.0 candidates, before the entity tools. Each name appears in public text
# only inside the release notes' mapping table, which tells 0.x and hosted users where it went.
RETIRED_TOOLS = set("""
biomcp_reference biomcp_help article_citation_evidence article_citations article_pdf article_recommendations
article_references article_supplement article_text author_papers cell_line_assay_count get_adverse_event
get_article get_author get_biomarker get_cell_line get_diagnostic get_disease get_drug get_gene get_gwas
get_pathway get_pgx get_phenotype get_protein get_study get_trial get_variant pathway_events pathway_genes
protein_complex_members protein_complexes protein_domains protein_features protein_isoforms protein_sequence
protein_structures search_adverse_event search_article search_author search_biomarker search_cell_line
search_diagnostic search_disease search_drug search_gene search_gwas search_organization search_pathway
search_pgx search_phenotype search_protein search_study search_trial search_variant trial_document
variant_assertions variant_normalize variant_population variant_recurrence
""".split())
REUSED_AS_ACTION = {"author_papers"}  # an old command name that is also a new action name
SHARED_TOOLS = {"biomcp", "help", "search", "fetch"}
ENTITY_TOOLS = {"gene", "variant", "disease", "drug", "trial", "article", "protein", "pathway", "phenotype",
                "adverse_event", "diagnostic", "cell_line", "study"}
TABLE = re.compile(r"<!-- retired-tools:start -->\n(.*?)<!-- retired-tools:end -->\n", re.S)
EXAMPLE = re.compile(r"`(\{\"action\":.*?\})`(?=[\s,.;:)|]|$)")


def doc_files(root):
    return [name for name in public_files(root)
            if name.endswith((".md", ".json")) and not name.startswith(("crates/", "tests/", "scripts/"))
            and (not name.startswith("release/") or name == "release/notes.md")]


def mapping(root):
    """Return the notes' mapping rows as (old name, tool, action), or raise."""
    match = TABLE.search((root / "release/notes.md").read_text())
    if not match:
        raise ValueError("release notes lack the retired tool mapping table")
    rows = [[cell.strip().strip("`") for cell in line.strip("|").split("|")] for line in match[1].splitlines()[2:]]
    if any(len(row) != 3 for row in rows):
        raise ValueError("release notes mapping table rows need three cells")
    return [tuple(row) for row in rows]


def retired(root):
    """A retired tool name is public text only inside the notes' mapping table."""
    for name in doc_files(root):
        text = (root / name).read_text()
        if name == "release/notes.md":
            text = TABLE.sub("", text)
        for number, line in enumerate(text.splitlines(), 1):
            for word in re.findall(r"(?<![\w-])[a-z][a-z_]*[a-z](?![\w-])", line):
                if word in RETIRED_TOOLS - REUSED_AS_ACTION:
                    raise ValueError(f"{name}:{number}: retired tool name {word} outside the release notes mapping table")
    rows = mapping(root)
    if {row[0] for row in rows} != RETIRED_TOOLS:
        raise ValueError("release notes mapping table does not list every retired tool name exactly")
    for old, tool, action in rows:
        if old != "biomcp_reference" and tool not in ENTITY_TOOLS | SHARED_TOOLS:
            raise ValueError(f"release notes map {old} to unknown tool {tool}")


def example_calls(root):
    """Return (file, tool, call) for each example call: a backticked call after the tool's backticked name."""
    found = []
    for name in doc_files(root):
        for number, line in enumerate((root / name).read_text().splitlines(), 1):
            for match in EXAMPLE.finditer(line):
                tools = [word for word in re.findall(r"`([a-z_]+)`", line[:match.start()]) if word in ENTITY_TOOLS]
                if not tools:
                    raise ValueError(f"{name}:{number}: example call names no entity tool before it")
                try:
                    call = json.loads(match[1])
                except json.JSONDecodeError as error:
                    raise ValueError(f"{name}:{number}: example call is not JSON: {error}") from error
                action = call.get("action")
                if sorted(call) != sorted(["action", action]) or not isinstance(call.get(action), dict):
                    raise ValueError(f"{name}:{number}: example call is not an action with one object named after it")
                found.append((f"{name}:{number}", tools[-1], call))
    return found


def catalog(executable):
    """Ask the installed server for the typed tool list, as an MCP client does."""
    frames = [{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
                  "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "docs", "version": "1"}}},
              {"jsonrpc": "2.0", "method": "notifications/initialized"},
              {"jsonrpc": "2.0", "id": 2, "method": "tools/list"}]
    reply = subprocess.run([str(executable), "serve", "--tool-mode", "typed", "--offline"], check=False,
                           input="".join(json.dumps(frame) + "\n" for frame in frames),
                           capture_output=True, text=True, timeout=60)
    for line in reply.stdout.splitlines():
        message = json.loads(line)
        if message.get("id") == 2:
            return {tool["name"]: tool for tool in message["result"]["tools"]}
    raise ValueError(f"typed server returned no tool list: {reply.stderr}")

ANNOTATIONS = {"$schema", "$id", "$defs", "title", "description", "default", "examples", "deprecated", "readOnly",
               "format"}


def schema_errors(schema, value, root, path="$"):
    """Check a value against the JSON Schema keywords MCP tool schemas use; refuse keywords it cannot check."""
    if schema is True:
        return []
    if schema is False:
        return [f"{path}: not allowed"]
    errors = []
    for key, rule in schema.items():
        if key == "$ref":
            target = root
            for part in rule.removeprefix("#/").split("/") if rule != "#" else []:
                target = target[part.replace("~1", "/").replace("~0", "~")]
            errors += schema_errors(target, value, root, path)
        elif key == "type":
            kinds = rule if isinstance(rule, list) else [rule]
            checks = {"object": dict, "array": list, "string": str, "boolean": bool, "null": type(None)}
            ok = any(isinstance(value, checks[k]) if k in checks else
                     (isinstance(value, int) and not isinstance(value, bool)) if k == "integer" else
                     (isinstance(value, (int, float)) and not isinstance(value, bool)) for k in kinds)
            if not ok:
                errors.append(f"{path}: {value!r} is not of type {kinds}")
        elif key == "enum" and value not in rule:
            errors.append(f"{path}: {value!r} is not one of {rule}")
        elif key == "const" and value != rule:
            errors.append(f"{path}: {value!r} is not {rule!r}")
        elif key == "required" and isinstance(value, dict):
            errors += [f"{path}: {name!r} is required" for name in rule if name not in value]
        elif key == "properties" and isinstance(value, dict):
            for name, sub in rule.items():
                if name in value:
                    errors += schema_errors(sub, value[name], root, f"{path}.{name}")
        elif key == "additionalProperties" and isinstance(value, dict):
            extra = [name for name in value if name not in schema.get("properties", {})]
            if rule is False:
                errors += [f"{path}: {name!r} is not allowed" for name in extra]
            elif rule is not True:
                for name in extra:
                    errors += schema_errors(rule, value[name], root, f"{path}.{name}")
        elif key == "items" and isinstance(value, list):
            for index, item in enumerate(value):
                errors += schema_errors(rule, item, root, f"{path}[{index}]")
        elif key in ("minItems", "maxItems") and isinstance(value, list):
            if (len(value) < rule) if key == "minItems" else (len(value) > rule):
                errors.append(f"{path}: {len(value)} items violates {key} {rule}")
        elif key in ("minLength", "maxLength") and isinstance(value, str):
            if (len(value) < rule) if key == "minLength" else (len(value) > rule):
                errors.append(f"{path}: length violates {key} {rule}")
        elif key == "pattern" and isinstance(value, str) and not re.search(rule, value):
            errors.append(f"{path}: {value!r} does not match {rule}")
        elif key in ("minimum", "maximum") and isinstance(value, (int, float)) and not isinstance(value, bool):
            if (value < rule) if key == "minimum" else (value > rule):
                errors.append(f"{path}: {value} violates {key} {rule}")
        elif key in ("oneOf", "anyOf", "allOf"):
            results = [schema_errors(sub, value, root, path) for sub in rule]
            passed = sum(not found for found in results)
            if (key == "allOf" and passed != len(rule)) or (key == "anyOf" and not passed) or (key == "oneOf" and passed != 1):
                errors.append(f"{path}: {value!r} does not satisfy {key}: {[found[:1] for found in results]}")
        elif key in {"properties", "items", "additionalProperties", "required", "minItems", "maxItems", "minLength",
                     "maxLength", "pattern", "minimum", "maximum", "enum", "const"}:
            continue  # a keyword aimed at another kind of value
        elif key not in ANNOTATIONS and not key.startswith("x-"):
            raise ValueError(f"{path}: schema keyword {key} is not supported by the docs check")
    return errors


def validate_examples(root, executable):
    """Each example must pass its tool's input schema, as 0758 requires of tool descriptions."""
    tools = catalog(executable)
    if set(tools) != ENTITY_TOOLS | SHARED_TOOLS:
        raise ValueError(f"typed server offers {sorted(tools)}, not the 17 entity and shared tools")
    seen = set()
    for where, tool, call in example_calls(root):
        errors = schema_errors(tools[tool]["inputSchema"], call, tools[tool]["inputSchema"])
        if errors:
            raise ValueError(f"{where}: example call for {tool} fails its schema: {errors[0]}")
        seen.add(tool)
    if seen != ENTITY_TOOLS:
        raise ValueError(f"no example call for tools: {sorted(ENTITY_TOOLS - seen)}")
    for old, tool, action in mapping(root):
        if tool in tools and f'"{action}"' not in json.dumps(tools[tool]["inputSchema"]) and action != "none":
            raise ValueError(f"release notes map {old} to {tool} {action}, which the server does not offer")


def commercial_setup(root):
    text = (root / "docs/configuration.md").read_text()
    sections = re.findall(r"(?ms)^## Local source exclusions\n(.*?)(?=^## |\Z)", text)
    if len(sections) != 1:
        raise ValueError("configuration needs one local source exclusions section")
    examples = re.findall(r"```json\n(.*?)```", sections[0], re.S)
    if len(examples) != 1:
        raise ValueError("local source exclusions needs one JSON configuration example")
    try:
        return json.loads(examples[0])
    except ValueError as error:
        raise ValueError("local source exclusions configuration example is invalid JSON") from error


def check(root, executable=None):
    architecture(root)
    internal_names(root)
    retired(root)
    example_calls(root)
    if executable:
        validate_examples(root, executable)
    readme = (root / "README.md").read_text()
    examples = [shlex.split(line)[1:] for block in re.findall(r"```sh\n(.*?)```", readme, re.S)
                for line in block.splitlines() if line.startswith("biomcp ")]
    recorded = [step["run"] for path in sorted((root / "tests/cases/readme").glob("*.yaml"))
                for step in json.loads(path.read_text())["steps"]]
    if sorted(examples) != sorted(recorded) or len(recorded) != 4:
        raise ValueError("README examples differ from recorded cases")

    commercial = commercial_setup(root)
    for path in (root / "tests/cases/readme").glob("*.yaml"):
        case = json.loads(path.read_text())
        if case.get("replay") and case.get("config") != commercial:
            raise ValueError(f"Configuration commercial setup differs from {path}")

    if (root / "skills/SKILL.md").stat().st_size > 4096:
        raise ValueError("agent skill exceeds 4 KB")
    roadmap = (root / "ROADMAP.md").read_text()
    if roadmap.count(LEDGER_START) != 1 or roadmap.count(LEDGER_END) != 1:
        raise ValueError("parity table differs: expected one generated ledger block")
    actual = LEDGER_START + roadmap.split(LEDGER_START, 1)[1].split(LEDGER_END, 1)[0] + LEDGER_END
    if actual != table(root):
        raise ValueError("parity table differs: run python3 scripts/parity.py and replace its block")
    note = (root / "sdlc/planning/upgrade.md").read_text()
    if note.count(UPGRADE_START) != 1 or note.count(UPGRADE_END) != 1:
        raise ValueError("upgrade commands differ: expected one generated block")
    actual = UPGRADE_START + note.split(UPGRADE_START, 1)[1].split(UPGRADE_END, 1)[0] + UPGRADE_END
    if actual != upgrade(root):
        raise ValueError("upgrade commands differ: run python3 scripts/parity.py --upgrade and replace its block")
    reference = (root / "docs/reference/commands.md").read_text()
    commands = set(re.findall(r"^## biomcp (.+)$", reference, re.MULTILINE))
    for path, (command, outcome, *_) in rows(root):
        name = command.strip("`")
        # 0.9 placed the bundle before sync; the current grammar places sync first.
        current = "sync " + name.removesuffix(" sync") if name.endswith(" sync") else name
        if outcome in ("Deferred by ruling", "Dropped by ruling") and current in commands:
            raise ValueError(f"{path}: {command} is {outcome.lower()} but appears in command reference")
    cases = {path.relative_to(root).as_posix() for path in (root / "tests/cases").rglob("*.yaml")}
    if not cases:
        raise ValueError("no outside-in cases found")
    names = set()
    for line in (root / "sdlc/features.md").read_text().splitlines():
        if not line:
            continue
        match = re.fullmatch(r"([a-z][a-z0-9-]*): (.+[.!?])", line)
        if not match or match[1] in names or " Cases:" in line:
            raise ValueError(f"invalid or duplicate feature line: {line}")
        names.add(match[1])
    for path in (root / "scripts").glob("*.py"):
        if len(path.read_text().splitlines()) > 600:
            raise ValueError(f"script exceeds file line limit: {path.name}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--executable", type=Path, help="biomcp executable; validates example calls against its tool schemas")
    args = parser.parse_args()
    try:
        check(args.root, args.executable)
    except (ValueError, OSError) as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
    print("Parity, feature declarations and architecture check references agree; public files name no internal machine or workspace path")
