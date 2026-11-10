"""Run literal trial names from --list; print only Cargo diagnostics and trial results."""
import argparse
import difflib
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


SELECTOR_HINT = ("; use --target cases for YAML cases and routine Rust CLI/MCP trials "
                 "(including protocol and pool trials); --cases DIRECTORY selects a YAML root. "
                 "--target tools --proofs selects a separate tooling-proof catalog. "
                 "Copy exact names from the selected runner's --list")


def run_selected(target: Sequence[str], cwd: Path, names: Sequence[str],
                 prepare: Callable[[], None], *, case_files: Sequence[str] = ()) -> None:
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
    for path in dict.fromkeys(case_files):
        matches = sorted(name for name in available
                         if name.partition("/")[2] == path and path.endswith(".yaml"))
        if len(matches) != 1:
            nearest = matches[:3] or difflib.get_close_matches(
                path, sorted(available), n=3, cutoff=0)
            raise ValueError(f"case file {path!r} requires exactly one catalog match; "
                             f"found {len(matches)}; nearest catalog entries: "
                             + ", ".join(nearest) + SELECTOR_HINT)
        selected.append(matches[0])
    selected = list(dict.fromkeys(selected))
    unknown = [name for name in selected if name not in available]
    if not selected or unknown:
        raise ValueError("trial selection requires exact names from this target's --list: "
                         + ", ".join(unknown) + SELECTOR_HINT)
    prepare()
    status = 0
    for name in selected:
        code = subprocess.run([*command, name, "--exact"], cwd=cwd).returncode
        status = status or code
    if status:
        raise subprocess.CalledProcessError(status, command)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", choices=["cases", "tools", "ratchets"], default="cases",
                        help="cases (default): YAML cases and routine Rust CLI/MCP trials, including protocol and pool trials; tools --proofs: separate tooling-proof catalog")
    parser.add_argument("--runner", type=Path, help="Reuse this runner; omitting it builds the --target runner using Cargo JSON artifact identity")
    parser.add_argument("--installed", action="store_true", help="Copy product, runner and public fixtures into an isolated install")
    parser.add_argument("--executable", type=Path, help="Reuse this product; omitting it builds the CLI after selection (except --list)")
    parser.add_argument("--profile", choices=["dev", "release"], default="dev")
    parser.add_argument("--cases", type=Path, help="Explicit case root interpreted by the Rust runner; not a proof selector")
    parser.add_argument("--case-file", action="append", default=[], metavar="RELATIVE/PATH.yaml",
                        help="select one exact case-relative path from the runner catalog; repeatable")
    parser.add_argument("--proofs", action="store_true", help="select the separate tooling-proof catalog in tools; routine Rust CLI/MCP trials use --target cases")
    parser.add_argument("--list", action="store_true", help="list literal names without preparing the product")
    parser.add_argument("names", nargs="*", help="exact names copied from the selected runner's --list")
    args = parser.parse_args()
    if not args.list and not args.names and not args.case_file:
        parser.error("trial selection requires exact names from --list")
    if args.proofs and args.target != "tools":
        parser.error("--proofs requires --target tools")
    if args.cases and (args.target == "ratchets" or args.proofs):
        parser.error("--cases requires a case runner")
    if args.case_file and (args.target == "ratchets" or args.proofs or args.list):
        parser.error("--case-file requires a case runner and cannot accompany --list")
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
        run_selected(target, cwd, args.names, prepare, case_files=args.case_file)


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        if error.stdout:
            print(error.stdout, end="", flush=True)
        raise SystemExit(error.returncode)
    except (OSError, ValueError) as error:
        raise SystemExit(str(error))
