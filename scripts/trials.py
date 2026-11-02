"""Run literal trial names from --list; print only Cargo diagnostics and trial results."""
import argparse
import os
from pathlib import Path
import subprocess
import sys
from typing import Callable, Sequence

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from release.artifacts import cargo


def run_selected(target: Sequence[str], cwd: Path, names: Sequence[str],
                 prepare: Callable[[], None]) -> None:
    """List and run the same target in its required directory using exact names."""
    if not target:
        raise ValueError("trial selection requires an existing target command")
    command = [str(Path(target[0]).resolve(strict=True)), *target[1:]]
    cwd = cwd.resolve(strict=True)
    listed = subprocess.run([*command, "--list"], cwd=cwd, check=True,
                            stdout=subprocess.PIPE, text=True)
    available = {line.removesuffix(": test") for line in listed.stdout.splitlines()
                 if line.endswith(": test")}
    selected = list(dict.fromkeys(names))
    unknown = [name for name in selected if name not in available]
    if not selected or unknown:
        raise ValueError("trial selection requires exact names from this target's --list: "
                         + ", ".join(unknown))
    prepare()
    for name in selected:
        subprocess.run([*command, name, "--exact"], cwd=cwd, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", choices=["cases", "tools", "ratchets"], default="cases")
    parser.add_argument("--profile", choices=["dev", "release"], default="dev")
    parser.add_argument("--cases", type=Path, help="case directory interpreted by the Rust runner")
    parser.add_argument("--proofs", action="store_true", help="select custom trials in tools")
    parser.add_argument("--list", action="store_true", help="list literal names without preparing the product")
    parser.add_argument("names", nargs="*", help="literal names copied from --list")
    args = parser.parse_args()
    if not args.list and not args.names:
        parser.error("trial selection requires exact names from --list")
    if args.proofs and args.target != "tools":
        parser.error("--proofs requires --target tools")
    if args.cases and (args.target == "ratchets" or args.proofs):
        parser.error("--cases requires a case runner")
    cases = args.cases.resolve(strict=True) if args.cases else None
    os.chdir(ROOT)
    target = [cargo(args.profile, tests=args.target)["executable"]]
    if args.proofs:
        os.environ["BIOMCP_TEST_PYTHON"] = sys.executable
        target.append("--proofs")
    elif cases or args.target == "tools":
        target += ["--cases", str(cases or ROOT / "tests/cases")]
    cwd = ROOT / "crates/biomcp"
    if args.list:
        subprocess.run([*target, "--list"], cwd=cwd, check=True)
    else:
        def prepare():
            os.environ["BIOMCP_TEST_EXECUTABLE"] = cargo(args.profile)["executable"]
        run_selected(target, cwd, args.names, prepare)


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        if error.stdout:
            print(error.stdout, end="", flush=True)
        raise SystemExit(error.returncode)
    except (OSError, ValueError) as error:
        raise SystemExit(str(error))
