#!/bin/sh
# Exercise installed TLS diagnostics against local fixtures; never contact a provider.
set -eu
if [ "${1:-focused}" = gate ]; then
    cd "$(dirname "$0")/.."
    exec python3 -c 'import json, os; os.execlp("sh", "sh", "-c", json.load(open("sdlc/pm.json"))["ticketsGate"])'
fi
python="$(uv python find --offline 3.13.17)"
export PATH="$(dirname "$python"):$HOME/.cargo/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.95.0
proofs="$PWD/target/0582-proofs"
mkdir -p "$proofs"
git check-ignore "$proofs/probe"
case "${1:-focused}" in
    red)
        mkdir -p "$proofs/red-baseline"
        scp "$2" "$proofs/red-baseline/biomcp"
        scp "$3" "$proofs/red-baseline/source-bindings.json"
        python3 - "$proofs/red-baseline" "$4" <<'PYHASH'
import hashlib
import json
import sys
from pathlib import Path
root = Path(sys.argv[1])
actual = hashlib.sha256((root / "biomcp").read_bytes()).hexdigest()
binding = json.loads((root / "source-bindings.json").read_text())
if actual != sys.argv[2] or actual != binding["binary_sha256"]:
    raise SystemExit("baseline artifact does not match source binding")
print("Baseline binary SHA-256:", actual)
print("Baseline binary source:", binding["binary_source_commit"])
PYHASH
        chmod +x "$proofs/red-baseline/biomcp"
        export BIOMCP_TRANSCRIPT_DIR="$proofs/red/transcripts"
        export BIOMCP_EVIDENCE_DIR="$proofs/red/fixtures"
        python3 scripts/trials.py --installed --executable "$proofs/red-baseline/biomcp" corporate-network/additive-ca-tls-and-proxy-isolation
        ;;
    focused)
        export BIOMCP_TRANSCRIPT_DIR="$proofs/green/transcripts"
        export BIOMCP_EVIDENCE_DIR="$proofs/green/fixtures"
        python3 scripts/trials.py --installed corporate-network/additive-ca-tls-and-proxy-isolation
        mkdir -p "$proofs/installed/bin"
        cp "${CARGO_TARGET_DIR:-target}/debug/biomcp" "$proofs/installed/bin/biomcp"
        python3 - "$proofs" <<'PYBIND'
import hashlib
import json
from pathlib import Path
import subprocess
import sys
root = Path(sys.argv[1])
sha = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
files = subprocess.check_output(["git", "diff", "--name-only", "974d11223", "HEAD"], text=True).splitlines()
binding = {"source_commit": sha, "binary_source_commit": sha,
           "binary_sha256": hashlib.sha256((root / "installed/bin/biomcp").read_bytes()).hexdigest(),
           "files": {p: hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in files}}
(root / "source-bindings.json").write_text(json.dumps(binding, indent=2) + "\n")
print("Installed binary SHA-256:", binding["binary_sha256"])
PYBIND
        ;;
    scan)
        python3 - "$proofs" <<'PYSCAN'
import json
from pathlib import Path
import subprocess
import sys
proofs = Path(sys.argv[1])
packages = Path.home().glob(".npm/_npx/*/node_modules/jscpd/package.json")
scanner = next((p.parent.parent / ".bin/jscpd" for p in packages
                if json.loads(p.read_text()).get("version") == "4.0.5"), None)
if scanner is None:
    raise SystemExit("cached jscpd 4.0.5 unavailable")
subprocess.run([scanner, "--version"], check=True)
changed = subprocess.check_output(["git", "diff", "--name-only", "974d11223", "HEAD", "--", "crates"], text=True).splitlines()
product = [p for p in changed if p.endswith(".rs") and "/tests/" not in p and "biomcp-test-support" not in p]
tests = ["crates/biomcp/tests/runner/corporate.rs", "crates/biomcp/tests/runner/launch.rs",
         "crates/biomcp-test-support/src/network_stub.rs"]
# Include nearby transport helper context in both scans, without compiling baseline code.
product = sorted(set(product + [str(p) for p in Path("crates/biomcp-fetch/src").rglob("*.rs")]))
baseline = proofs / "baseline-source"
baseline.mkdir(exist_ok=True)
archive = subprocess.run(["git", "archive", "974d11223", *product, *tests], check=True, stdout=subprocess.PIPE)
subprocess.run(["tar", "-xf", "-", "-C", baseline], input=archive.stdout, check=True)
for kind, paths in (("product", product), ("tests", tests)):
    for label, prefix in (("baseline-", baseline), ("", Path("."))):
        subprocess.run([scanner, *[str(prefix / p) for p in paths], "--format", "rust",
                        "--max-size", "10mb", "--reporters", "json", "--output",
                        proofs / ("jscpd-" + label + kind)], check=True)
PYSCAN
        ;;
    final)
        sh "$0" focused
        sh "$0" scan
        sh "$0" gate
        ;;
    lint)
        cargo clippy --keep-going --workspace --all-targets --locked -- -D warnings
        cargo clippy --keep-going --locked -p biomcp --test tools -- -D warnings
        cargo test --locked -p biomcp --test ratchets repository_obeys_structural_contract -- --exact
        ;;
    *) exit 2 ;;
esac
