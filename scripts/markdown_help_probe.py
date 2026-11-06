"""Retain installed help text and its executable binding for a local diagnosis."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from release.smoke import environment


def main():
    binary = Path(sys.argv[1]).resolve(strict=True)
    output = Path(sys.argv[2]).resolve()
    output.mkdir(parents=True, exist_ok=True)
    manifest = json.loads(binary.with_name("release.json").read_text())
    digest = hashlib.sha256(binary.read_bytes()).hexdigest()
    if digest != manifest["binary_sha256"]:
        raise ValueError("installed producer executable does not match its manifest")
    (output / "binding.json").write_text(json.dumps({
        "revision": manifest["revision"], "binary_sha256": digest,
    }, indent=2) + "\n")
    with tempfile.TemporaryDirectory(prefix="markdown-help-probe-") as scratch:
        for path in (["author"], ["author", "papers"], ["help"]):
            for form in ("json", "md"):
                result = subprocess.run(
                    [str(binary), "--offline", "help", *path, "--format", form],
                    env=environment(scratch), cwd=scratch, capture_output=True, check=True,
                )
                name = "-".join(path) + "." + form
                (output / name).write_bytes(result.stdout)
                (output / (name + ".stderr")).write_bytes(result.stderr)
                print(f"INSTALLED_HELP {name} bytes={len(result.stdout)}")
                if form == "md":
                    print(result.stdout.decode(), flush=True)


if __name__ == "__main__":
    main()
