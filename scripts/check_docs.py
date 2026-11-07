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


def check(root):
    architecture(root)
    internal_names(root)
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
    print("Parity, feature declarations and architecture check references agree; public files name no internal machine or workspace path")
