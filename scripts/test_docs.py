"""Exercise documentation contracts through the checker command."""

import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from check_docs import ENTITY_TOOLS, SHARED_TOOLS, mapping
from parity import LEDGER_END, LEDGER_START, UPGRADE_END, UPGRADE_START, table, upgrade

ROOT = Path(__file__).resolve().parents[1]


def check(root):
    return subprocess.run(
        [sys.executable, str(ROOT / "scripts/check_docs.py"), "--root", str(root)],
        capture_output=True,
        text=True,
        check=False,
    )


def replace_block(text, start, end, generated):
    return text.split(start, 1)[0] + generated + text.split(end, 1)[1]


FAKE_SERVER = """#!/usr/bin/env python3
import json, sys
ACTIONS = %s
tools = []
for name in %s:
    schema = {"type": "object", "required": ["action"], "additionalProperties": False,
              "properties": {"action": {"enum": ACTIONS},
                             "get": {"type": "object", "properties": {"input": {"type": ["array", "string"]}}},
                             "search": {"type": "object"}}}
    tools.append({"name": name, "inputSchema": schema})
for line in sys.stdin:
    message = json.loads(line)
    if message.get("id") == 2:
        print(json.dumps({"jsonrpc": "2.0", "id": 2, "result": {"tools": tools}}), flush=True)
"""


def tools_check(root, readme, original_readme):
    """A retired tool name, a bad example and a missing example each fail the checker."""
    notes = root / "release/notes.md"
    original_notes = notes.read_text()
    for path, text in ((readme, original_readme), (notes, original_notes)):
        path.write_text(text + "\nCall get_gene for a gene.\n")
        result = check(root)
        assert result.returncode != 0 and "retired tool name get_gene" in result.stderr, result
        path.write_text(text)
    notes.write_text(original_notes.replace("| `search_gene` | `gene` | `search` |\n", "", 1))
    result = check(root)
    assert result.returncode != 0 and "does not list every retired tool name" in result.stderr, result
    notes.write_text(original_notes)
    readme.write_text(original_readme.replace('`{"action":"get","get":{"input":["BRAF"]}}`', '`{"action":"get","input":["BRAF"]}`'))
    result = check(root)
    assert result.returncode != 0 and "not an action with one object" in result.stderr, result
    readme.write_text(original_readme)
    server = root / "fake-biomcp"
    actions = sorted({row[2] for row in mapping(root)} - {"none"})
    server.write_text(FAKE_SERVER % (repr(actions), repr(sorted(ENTITY_TOOLS | SHARED_TOOLS))))
    server.chmod(0o755)
    run = lambda: subprocess.run([sys.executable, str(ROOT / "scripts/check_docs.py"), "--root", str(root), "--executable", str(server)],
                                 capture_output=True, text=True, check=False)
    result = run()
    assert result.returncode == 0, result.stderr
    readme.write_text(original_readme.replace('`{"action":"get","get":{"input":["BRAF"]}}`', '`{"action":"get","get":{"input":5}}`'))
    result = run()
    assert result.returncode != 0 and "fails its schema" in result.stderr, result
    readme.write_text(original_readme)


def main():
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        for name in (
            "ROADMAP.md",
            "README.md",
            "skills/SKILL.md",
            "docs/reference/commands.md",
            "docs/configuration.md",
            "release/notes.md",
            "sdlc/features.md",
            "sdlc/planning/upgrade.md",
            "sdlc/planning/architecture.md",
            "sdlc/planning/parity-0.9.3.json",
        ):
            target = root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, target)
        for name in (
            "sdlc/planning/slices",
            "tests/cases",
            "crates/biomcp/tests/runner",
            "crates/biomcp-types",
            "crates/biomcp-fetch",
            "crates/biomcp-adapters",
        ):
            shutil.copytree(ROOT / name, root / name)
        architecture = root / "sdlc/planning/architecture.md"
        original_architecture = architecture.read_text()
        for name in re.findall(r"Check: `([^`]+)`", original_architecture):
            target = root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, target)
        reference = root / "docs/reference/commands.md"
        reference.write_text(reference.read_text().replace("biomcp_help", "help"))  # as 0763's regeneration does
        result = check(root)
        assert result.returncode == 0, result.stderr
        readme = root / "README.md"
        original_readme = readme.read_text()
        readme.write_text(original_readme.replace("get gene KRAS", "get gene INVENTED"))
        result = check(root)
        assert result.returncode != 0 and "README examples differ" in result.stderr, result
        readme.write_text(original_readme)
        architecture.write_text(original_architecture + "\n## Planted rule\n\nCheck: `scripts/missing_check.py`.\n")
        result = check(root)
        assert result.returncode != 0 and "architecture names a missing check file" in result.stderr, result
        architecture.write_text(original_architecture + "\n## Planted rule\n\nA rule without enforcement.\n")
        result = check(root)
        assert result.returncode != 0 and "architecture section lacks a check" in result.stderr, result
        architecture.write_text(original_architecture)
        guide = root / "skills/SKILL.md"
        original_guide = guide.read_bytes()
        guide.write_bytes(b"x" * 4096)
        assert check(root).returncode == 0
        guide.write_bytes(b"x" * 4097)
        result = check(root)
        assert result.returncode != 0 and "agent skill exceeds 4 KB" in result.stderr, result
        guide.write_bytes(original_guide)
        roadmap = root / "ROADMAP.md"
        original = roadmap.read_text()
        roadmap.write_text(
            original.replace("| Ported or renamed |", "| Invented outcome |", 1)
        )
        result = check(root)
        assert result.returncode != 0 and "parity table differs" in result.stderr, (
            result
        )
        roadmap.write_text(original)
        accounting = root / "sdlc/planning/slices/cross-cutting.md"
        original_accounting = accounting.read_text()
        note = root / "sdlc/planning/upgrade.md"
        original_note = note.read_text()
        for outcome in ("Deferred by ruling", "Dropped by ruling"):
            accounting.write_text(original_accounting.replace(
                "| `health` | Ported or renamed |", f"| `health` | {outcome} |", 1
            ))
            roadmap.write_text(replace_block(original, LEDGER_START, LEDGER_END, table(root)))
            note.write_text(replace_block(original_note, UPGRADE_START, UPGRADE_END, upgrade(root)))
            result = check(root)
            assert result.returncode != 0 and "appears in command reference" in result.stderr, result
        accounting.write_text(original_accounting)
        roadmap.write_text(original)
        note.write_text(original_note)
        note = root / "sdlc/planning/upgrade.md"
        original_note = note.read_text()
        note.write_text(
            original_note.replace("`gene definition`", "`gene invented`", 1)
        )
        result = check(root)
        assert result.returncode != 0 and "upgrade commands differ" in result.stderr, (
            result
        )
        note.write_text(original_note)
        slice_path = root / "sdlc/planning/slices/gene.md"
        original_slice = slice_path.read_text()
        slice_path.write_text(
            original_slice.replace(
                "Section 1 replaces this leaf", "Section 1 removes this leaf", 1
            )
        )
        result = check(root)
        assert result.returncode != 0 and "upgrade commands differ" in result.stderr, (
            result
        )
        slice_path.write_text(original_slice)
        for changed, expected in (
            (
                original_slice.replace(
                    "`search gene` | Ported", "`search invented` | Ported", 1
                ),
                "unknown inventory leaf",
            ),
            (
                original_slice.replace(
                    "| `search gene` | Ported or renamed | `--type`",
                    "| `search gene` | Ported or renamed | `--invented`",
                    1,
                ),
                "unknown option",
            ),
            (
                "\n".join(
                    line
                    for line in original_slice.split("\n")
                    if not line.startswith("| `gene definition` |")
                ),
                "parity table differs",
            ),
        ):
            assert changed != original_slice, expected
            slice_path.write_text(changed)
            result = check(root)
            assert result.returncode != 0 and expected in result.stderr, result
        slice_path.write_text(original_slice)
        features = root / "sdlc/features.md"
        original_features = features.read_text()
        features.write_text(
            original_features
            + "\nmissing: A caller sees a result. Cases: `tests/cases/missing/*.yaml`.\n"
        )
        result = check(root)
        assert result.returncode != 0 and "invalid or duplicate feature line" in result.stderr, result
        features.write_text(original_features)
        folder = "work" + "space"  # split so this file passes the check it tests
        for planted in (f"cd ~/{folder}/biomcp\n", f"cp $HOME/{folder}/t/x .\n"):
            readme.write_text(original_readme + planted)
            result = check(root)
            assert result.returncode != 0 and "README.md:" in result.stderr and "workspace path" in result.stderr, result
        readme.write_text(original_readme + "See https://example.org/home/about and /home/runner/work.\n")
        assert check(root).returncode == 0, check(root).stderr
        readme.write_text(original_readme)
        tools_check(root, readme, original_readme)
    print(
        "Documentation contracts reject missing architecture checks, phantom leaves, unknown options, missing leaf accounting, drift, invalid feature declarations and internal workspace paths"
    )


if __name__ == "__main__":
    main()
