"""Check documentation declarations against the checked-out tree."""
import argparse
import re
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


def check(root):
    architecture(root)
    readme = (root / "README.md").read_text()
    examples = [shlex.split(line)[1:] for block in re.findall(r"```sh\n(.*?)```", readme, re.S)
                for line in block.splitlines() if line.startswith("biomcp ")]
    recorded = [step["run"] for path in sorted((root / "tests/cases/readme").glob("*.yaml"))
                for step in json.loads(path.read_text())["steps"]]
    if sorted(examples) != sorted(recorded) or len(recorded) != 4:
        raise ValueError("README examples differ from recorded cases")

    commercial = json.loads(re.search(r"```json\n(.*?)```", readme, re.S).group(1))
    for path in (root / "tests/cases/readme").glob("*.yaml"):
        case = json.loads(path.read_text())
        if case.get("replay") and case.get("config") != commercial:
            raise ValueError(f"README commercial setup differs from {path}")

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
    args = parser.parse_args()
    try:
        check(args.root)
    except (ValueError, OSError) as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
    print("Parity, feature declarations and architecture check references agree")
