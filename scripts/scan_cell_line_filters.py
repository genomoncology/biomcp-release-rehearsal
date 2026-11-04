"""Compare duplicate Rust blocks in the cell-line slice before and after this change."""
import json
from pathlib import Path
import subprocess
import sys


def main():
    baseline = sys.argv[1]
    roots = ["crates/biomcp-types/src/cell_line", "crates/biomcp-query/src/cell_line",
             "crates/biomcp-adapters/src/cellosaurus"]
    packages = Path.home().glob(".npm/_npx/*/node_modules/jscpd/package.json")
    scanner = next((p.parent.parent / ".bin/jscpd" for p in packages
                    if json.loads(p.read_text()).get("version") == "4.0.5"), None)
    if scanner is None:
        raise ValueError("cached jscpd 4.0.5 is required")
    output = Path("target/cell-line-scans")
    before = output / "baseline-source"
    before.mkdir(parents=True, exist_ok=True)
    archive = subprocess.run(["git", "archive", baseline, *roots], check=True,
                             stdout=subprocess.PIPE)
    subprocess.run(["tar", "-xf", "-", "-C", str(before)], input=archive.stdout, check=True)
    subprocess.run([str(scanner), "--version"], check=True)
    for label, prefix in [("baseline", before), ("current", Path("."))]:
        destination = output / label
        subprocess.run([str(scanner), *[str(prefix / root) for root in roots], "--format", "rust",
                        "--max-size", "10mb", "--reporters", "json", "--output", str(destination)],
                       check=True)
        stats = json.loads((destination / "jscpd-report.json").read_text())["statistics"]["total"]
        print(label, json.dumps(stats, sort_keys=True))


if __name__ == "__main__":
    main()
