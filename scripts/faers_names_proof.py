"""Exercise installed repeatable FAERS names and compare retained single-name bytes."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import urllib.parse
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from release.smoke import environment


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    baseline_sha = hashlib.sha256(args.baseline.read_bytes()).hexdigest()
    assert baseline_sha == "b50998e586ccc38a2a4f6d851cce229651e380560814e5d4212d4694f89c7660"
    with tempfile.TemporaryDirectory(prefix="biomcp-faers-names-") as temporary:
        root = Path(temporary)
        binary = root / "biomcp"
        shutil.copy2(args.binary, binary)
        baseline = root / "baseline-biomcp"
        shutil.copy2(args.baseline, baseline)

        def run(program, name, command, expected=0):
            call_root = root / name
            call_root.mkdir()
            result = subprocess.run([str(program), *command], cwd=call_root,
                                    env=environment(call_root), capture_output=True, timeout=60)
            (output / f"{name}.stdout.json").write_bytes(result.stdout)
            (output / f"{name}.stderr").write_bytes(result.stderr)
            if result.returncode != expected:
                raise RuntimeError(f"{name} exited {result.returncode}; see retained output")
            return result.stdout

        repeated = ["search", "adverse-event", "--drug", "imatinib", "--drug", "nilotinib",
                    "--source", "faers", "--limit", "1"]
        reply = json.loads(run(binary, "two-names", repeated))
        expression = reply["result"]["source_query"]
        url = "https://api.fda.gov/drug/event.json?" + urllib.parse.urlencode({
            "search": expression, "limit": 1, "skip": 0, "sort": "receivedate:desc"})
        with urllib.request.urlopen(url, timeout=30) as response:
            native = response.read()
        (output / "openfda.json").write_bytes(native)
        (output / "openfda-url.txt").write_text(url + "\n")
        bindings = {
            "source_commit": subprocess.check_output(
                ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
            "binary_sha256": hashlib.sha256(args.binary.read_bytes()).hexdigest(),
            "baseline_sha256": baseline_sha,
            "baseline_public_run": 41806259796,
            "native_sha256": hashlib.sha256(native).hexdigest(),
            "native_url": url,
            "source_query": expression,
        }
        (output / "bindings.json").write_text(json.dumps(bindings, indent=2) + "\n")
        total = json.loads(native)["meta"]["results"]["total"]
        assert reply["page"]["total"] == total
        assert total > 50000
        for index, flags in enumerate([[], ["--received-from", "2024-01-01",
                                            "--received-to", "2024-12-31"]]):
            command = ["--replay", str(ROOT / "tests/replies/adverse-event/faers-synthetic"),
                       "search", "adverse-event", "pembrolizumab", *flags]
            before = run(baseline, f"single-{index}-before", command)
            after = run(binary, f"single-{index}-after", command)
            assert before == after, f"single-name case {index} changed"
        oversized = ["search", "adverse-event"] + ["--drug", "synthetic-name"] * 40
        refusal = json.loads(run(binary, "cap", oversized, expected=2))
        assert "FAERS_QUERY_MAX_BYTES" in refusal["error"]["message"]
        print(json.dumps({"two_name_native_total": total, "single_name_byte_equal_cases": 2,
                          "cap_usage_exit": 2}))


if __name__ == "__main__":
    main()
