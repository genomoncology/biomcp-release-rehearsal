"""Run literal trial names from --list; print only Cargo diagnostics and trial results."""
import argparse
import os
import shutil
import tempfile
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
    status = 0
    for name in selected:
        code = subprocess.run([*command, name, "--exact"], cwd=cwd).returncode
        status = status or code
    if status:
        raise subprocess.CalledProcessError(status, command)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", choices=["cases", "tools", "ratchets"], default="cases")
    parser.add_argument("--runner", type=Path, help="Reuse a previously built runner")
    parser.add_argument("--installed", action="store_true", help="Copy product, runner and public fixtures into an isolated install")
    parser.add_argument("--executable", type=Path, help="Exercise this installed product without building another CLI")
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
    target = [str(args.runner.resolve(strict=True)) if args.runner else cargo(args.profile, tests=args.target)["executable"]]
    if args.proofs:
        os.environ["BIOMCP_TEST_PYTHON"] = sys.executable
        target.append("--proofs")
    elif cases or args.target == "tools":
        target += ["--cases", str(cases or ROOT / "tests/cases")]
    cwd = ROOT / "crates/biomcp"
    installed = tempfile.TemporaryDirectory(prefix="biomcp-installed-trials-") if args.installed else None
    if installed:
        from release.gate import stage_fixtures
        cwd = Path(installed.name)
        stage_fixtures(ROOT, cwd / "fixtures")
        os.environ["BIOMCP_TEST_FIXTURES"] = str(cwd / "fixtures")
        shutil.copy2(target[0], cwd / Path(target[0]).name)
        target[0] = str(cwd / Path(target[0]).name)
    if args.list:
        subprocess.run([*target, "--list"], cwd=cwd, check=True)
    else:
        def prepare():
            executable = Path(args.executable.resolve(strict=True)) if args.executable else Path(cargo(args.profile)["executable"])
            if installed:
                shutil.copy2(executable, cwd / executable.name)
                executable = cwd / executable.name
            os.environ["BIOMCP_TEST_EXECUTABLE"] = str(executable)
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
