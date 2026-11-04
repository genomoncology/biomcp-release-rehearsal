"""Prove the article license gate rejects a planted restricted recording."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]


def check(root):
    return subprocess.run([sys.executable, str(ROOT / "scripts/article_licenses.py"),
                           "--root", str(root)], capture_output=True, text=True, check=False)


def main():
    with tempfile.TemporaryDirectory(prefix="article-licenses-") as scratch:
        root = Path(scratch)
        destination = root / "tests/replies/article"
        shutil.copytree(ROOT / "tests/replies/article", destination)
        result = check(root)
        assert result.returncode == 0, result.stderr
        print("GREEN baseline: " + result.stdout.strip(), flush=True)
        origins = json.loads((destination / "licenses.json").read_text())
        path = destination / origins["PMC5606870"]["evidence"]
        original = path.read_bytes()
        record = json.loads(original)
        record["exchanges"][0]["body"]["value"]["license_code"] = "CC BY-NC"
        path.write_text(json.dumps(record))
        result = check(root)
        assert result.returncode != 0, result.stdout
        assert "disallowed article fixture license: CC BY-NC" in result.stderr, result.stderr
        print("RED planted capture: " + result.stderr.strip(), flush=True)
        path.write_bytes(original)
        result = check(root)
        assert result.returncode == 0, result.stderr
        print("GREEN restored: " + result.stdout.strip(), flush=True)
        # A publisher URL or a case name cannot certify generated PDF bytes.
        for case in ("synthetic", "timeout"):
            path = next((destination / "pdf" / case).rglob("*.bin"))
            original = path.read_bytes()
            path.write_bytes(b"%PDF-1.4\nUnlicensed publisher article bytes\n%%EOF")
            result = check(root)
            assert result.returncode != 0, result.stdout
            assert "article capture has no recorded reuse license" in result.stderr, result.stderr
            print("RED unlicensed PDF: " + result.stderr.strip(), flush=True)
            path.write_bytes(original)
        path = destination / "fulltext/synthetic-unlicensed/pmc-oa/planted.json"
        path.parent.mkdir(parents=True)
        path.write_text(json.dumps({
            "source": "pmc-oa", "url": "https://example.org/PMC999999/article.xml",
            "exchanges": [{"body": {"kind": "text", "value": "<article>unlicensed</article>"}}],
        }))
        result = check(root)
        assert result.returncode != 0, result.stdout
        assert "article capture has no recorded reuse license" in result.stderr, result.stderr
        print("RED unlicensed synthetic case: " + result.stderr.strip(), flush=True)


if __name__ == "__main__":
    main()
