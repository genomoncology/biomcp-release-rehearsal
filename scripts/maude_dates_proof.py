"""Compare installed receipt requests with openFDA and retained FAERS replies."""
import argparse
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
    with tempfile.TemporaryDirectory(prefix="biomcp-receipt-proof-") as temporary:
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

        prefix = ["search", "adverse-event", "insulin pump", "--source", "maude"]
        run(baseline, "maude-month-before", prefix + [
            "--received-from", "2020-01", "--received-to", "2020-01"], expected=2)
        month = json.loads(run(binary, "maude-month", prefix + [
            "--received-from", "2020-01", "--received-to", "2020-01"]))
        day = json.loads(run(binary, "maude-day", prefix + [
            "--received-from", "2020-01-01", "--received-to", "2020-01-31"]))
        assert month["query"] == day["query"]
        expression = 'device.generic_name:"insulin pump" AND date_received:[20200101 TO 20200131]'
        assert month["result"]["source_query"] == day["result"]["source_query"] == expression
        url = "https://api.fda.gov/device/event.json?" + urllib.parse.urlencode({
            "search": expression, "limit": 1, "skip": 0, "sort": "date_received:desc"})
        with urllib.request.urlopen(url, timeout=30) as response:
            native = response.read()
        (output / "openfda.json").write_bytes(native)
        (output / "openfda-url.txt").write_text(url + "\n")
        total = json.loads(native)["meta"]["results"]["total"]
        assert month["page"]["total"] == day["page"]["total"] == total
        for index, flags in enumerate([[], ["--received-from", "2024-01-01",
                                            "--received-to", "2024-12-31"]]):
            command = ["--replay", str(ROOT / "tests/replies/adverse-event/faers-synthetic"),
                       "search", "adverse-event", "pembrolizumab", *flags]
            before = run(baseline, f"faers-{index}-before", command)
            after = run(binary, f"faers-{index}-after", command)
            assert before == after, f"FAERS case {index} changed"
        print(json.dumps({"maude_total": total, "month_day_request_equal": True,
                          "faers_byte_equal_cases": 2}))


if __name__ == "__main__":
    main()
