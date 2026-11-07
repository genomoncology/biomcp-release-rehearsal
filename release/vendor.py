"""Vendor the locked dependency graph before compiling release artifacts offline."""
import atexit
from functools import lru_cache
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def path_flags(folder):
    """Give a release binary's panic locations fixed names, not this machine's folders."""
    home = os.environ.get("CARGO_HOME") or Path.home() / ".cargo"
    prefixes = ((home, "cargo"), (ROOT, "biomcp"), (folder / "sources", "vendor"))
    flags = [f"--remap-path-prefix={path}={name}" for path, name in prefixes]
    # Cargo joins this table with .cargo/config.toml. RUSTFLAGS would replace
    # that file's flags, and Windows would lose its static C runtime silently.
    return "\n[target.'cfg(all())']\nrustflags = " + json.dumps(flags) + "\n"


@lru_cache(maxsize=1)
def stage():
    """Reuse one isolated source set for this release command and its children."""
    temporary = tempfile.TemporaryDirectory(
        prefix="biomcp-release-vendor-", dir=Path(tempfile.gettempdir()).resolve())
    atexit.register(temporary.cleanup)
    folder = Path(temporary.name)
    try:
        result = subprocess.run(
            ["cargo", "vendor", "--locked", str(folder / "sources")],
            cwd=ROOT, check=True, stdout=subprocess.PIPE, text=True)
        config = result.stdout + "\n[net]\noffline = true\n"
        for name, text in (("cargo-home", config), ("release-cargo-home", config + path_flags(folder))):
            (folder / name).mkdir()
            (folder / name / "config.toml").write_text(text)
    except BaseException:
        temporary.cleanup()
        raise
    return folder


def environment(clean_paths=False):
    """Only the shipped binary needs clean paths, so every other build keeps the
    plain config."""
    home = stage() / ("release-cargo-home" if clean_paths else "cargo-home")
    return dict(os.environ, CARGO_HOME=str(home), CARGO_NET_OFFLINE="true")


def main():
    command = sys.argv[1:]
    if command[:1] == ["--"]:
        command = command[1:]
    if not command:
        raise SystemExit("usage: vendor.py -- COMMAND [ARGUMENTS...]")
    raise SystemExit(subprocess.run(command, cwd=ROOT, env=environment()).returncode)


if __name__ == "__main__":
    main()
