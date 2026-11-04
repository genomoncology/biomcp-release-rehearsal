"""Compare installed cell-line outputs with a retained baseline executable."""
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from release.smoke import environment


def invoke(binary, words):
    with tempfile.TemporaryDirectory(prefix="cell-line-compatibility-") as directory:
        root = Path(directory)
        installed = root / "biomcp"
        shutil.copy2(binary, installed)
        return subprocess.run([str(installed), *words], cwd=root, env=environment(root),
                              capture_output=True, timeout=60)


def main():
    baseline, candidate = (Path(value).resolve(strict=True) for value in sys.argv[1:])
    replay = str(ROOT / "tests/replies/cell-line/examples")
    words = ["search", "cell-line", "MOLM13", "--replay", replay]
    before, after = (invoke(binary, words) for binary in (baseline, candidate))
    if before.returncode or after.returncode or before.stdout != after.stdout:
        raise ValueError("free-text output differs from the retained baseline")
    print(f"free-text stdout byte-identical: {len(after.stdout)} bytes")
    red = invoke(baseline, ["search", "cell-line", "--species", "Homo sapiens",
                            "--disease", "Melanoma", "--replay", replay])
    if red.returncode != 2 or b"species" not in red.stdout:
        raise ValueError("baseline did not reject the previously unsupported species filter")
    print("baseline red witness: species filter refused before source access")
    for label, options in [("cli", []), ("markdown", ["--format", "md"])]:
        help_reply = invoke(candidate, ["help", "search", "cell-line", *options])
        if help_reply.returncode:
            raise ValueError("cell-line help failed")
        print(f"cell-line search help [{label}] bytes={len(help_reply.stdout)}")


if __name__ == "__main__":
    main()
