"""Prove that the lint entry point refuses an unused dependency before compiling."""
from pathlib import Path
import subprocess


manifest = Path("crates/biomcp-fetch/Cargo.toml")
original = manifest.read_text()
try:
    manifest.write_text(original.replace("[dependencies]", "[dependencies]\nunused_probe = { package = 'libc', version = '0.2' }", 1))
    result = subprocess.run(["make", "lint"], capture_output=True, text=True, check=False)
    print(result.stdout, end="")
    print(result.stderr, end="")
    if result.returncode == 0 or "unused_probe" not in result.stdout + result.stderr:
        raise SystemExit("lint did not reject the planted unused dependency")
    if "cargo clippy" in result.stdout or "cargo check" in result.stdout:
        raise SystemExit("lint compiled before rejecting the unused dependency")
finally:
    manifest.write_text(original)
print("Lint rejected the planted unused dependency before compilation")
