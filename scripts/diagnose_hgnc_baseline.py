"""Run eight bounded, isolated recovery attempts against the existing Windows install."""
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile

BASELINE_SOURCE = "19032ac74503ff225dfb55e7a1a02afa5b5423f4"
ROOT = Path(r"C:\h\biomcp-0566")
INSTALLED = ROOT / "hgnc-installed"
PRODUCT = INSTALLED / "biomcp.exe"
RUNNER = INSTALLED / "tools-698351c37b45344f.exe"
TRIAL = "cache-resilience/hgnc-first-snapshot-recovers"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def capture_cli(transcripts, output):
    captures = []
    for path in sorted(transcripts.glob("*.json")):
        record = json.loads(path.read_text())
        if record.get("kind") == "trial":
            continue
        for execution in record["executions"]:
            stdout = bytes(execution["stdout"])
            name = f"cli-{len(captures) + 1}.json"
            (output / name).write_bytes(stdout)
            try:
                status = json.loads(stdout).get("status")
            except (ValueError, AttributeError):
                status = None
            captures.append({"file": name, "exit": execution["exit"], "status": status})
    return captures


def attempt(number, work, output):
    root = work / f"attempt-{number}"
    record = output / f"attempt-{number}"
    root.mkdir()
    record.mkdir()
    fixtures = root / "fixtures"
    shutil.copytree(INSTALLED / "fixtures", fixtures)
    for name in ["home", "config", "cache", "tmp", "evidence"]:
        (root / name).mkdir()
    transcripts = record / "transcript"
    environment = {**os.environ, "HOME": str(root / "home"),
                   "USERPROFILE": str(root / "home"),
                   "APPDATA": str(root / "config"), "LOCALAPPDATA": str(root / "cache"),
                   "XDG_CONFIG_HOME": str(root / "config"), "XDG_CACHE_HOME": str(root / "cache"),
                   "TMP": str(root / "tmp"), "TEMP": str(root / "tmp"),
                   "TMPDIR": str(root / "tmp"),
                   "BIOMCP_TEST_FIXTURES": str(fixtures),
                   "BIOMCP_TEST_EXECUTABLE": str(PRODUCT),
                   "BIOMCP_EVIDENCE_DIR": str(root / "evidence"),
                   "BIOMCP_TRANSCRIPT_DIR": str(transcripts)}
    free = shutil.disk_usage(ROOT).free
    result = subprocess.run([str(RUNNER), "--proofs", TRIAL, "--exact", "--test-threads", "1"],
                            cwd=root, env=environment, capture_output=True)
    (record / "runner.stdout").write_bytes(result.stdout)
    (record / "runner.stderr").write_bytes(result.stderr)
    raw = result.stdout + result.stderr
    database = re.search(rb'"local_error"\s*:\s*"database"', raw) is not None
    summary = {"attempt": number, "exit": result.returncode,
               "original_typed_database_failure": database,
               "free_bytes_before": free, "free_bytes_after": shutil.disk_usage(ROOT).free,
               "cli": capture_cli(transcripts, record)}
    summary["complete_cli_captured"] = any(
        row["exit"] == 0 and row["status"] == "complete" for row in summary["cli"])
    (record / "result.json").write_text(json.dumps(summary, indent=2))
    return summary


def main():
    request_source = sys.argv[1]
    output = ROOT / ("hgnc-bounded-output-" + request_source[:12])
    work = ROOT / ("hgnc-bounded-work-" + request_source[:12])
    output.mkdir()
    work.mkdir()
    try:
        version = subprocess.run([str(PRODUCT), "version", "--verbose"], capture_output=True, check=True)
        if f"Commit: {BASELINE_SOURCE}".encode() not in version.stdout:
            raise ValueError("installed product does not match the baseline source")
        binding = {"baseline_source": BASELINE_SOURCE, "embedded_commit_verified": True, "recipe_source": request_source,
                   "product": str(PRODUCT), "product_sha256": digest(PRODUCT),
                   "runner": str(RUNNER), "runner_sha256": digest(RUNNER),
                   "trial": TRIAL, "planned_attempts": 8, "maximum_concurrent_attempts": 4,
                   "controller": "m5", "remote_vm": {"host": "gator", "cpus": 2, "memory_gib": 8}}
        (output / "binding.json").write_text(json.dumps(binding, indent=2))
        print(json.dumps(binding), flush=True)
        prior = ROOT / "hgnc-bounded-output-a08841e78b27" / "binding.json"
        if prior.exists():
            shutil.copy2(prior, output / "j25685-partial-binding.json")
        shutil.copytree(INSTALLED / "hgnc-transcript", output / "baseline-transcript")
        baseline = output / "baseline-cli"
        baseline.mkdir()
        print(json.dumps({"baseline_cli": capture_cli(output / "baseline-transcript", baseline)}), flush=True)
        summaries = []
        with ThreadPoolExecutor(max_workers=4) as pool:
            for batch in [range(1, 5), range(5, 9)]:
                results = list(pool.map(lambda number: attempt(number, work, output), batch))
                summaries.extend(results)
                for result in results:
                    print(json.dumps(result), flush=True)
                if any(result["original_typed_database_failure"] for result in results):
                    break
        unchanged = digest(PRODUCT) == binding["product_sha256"] and digest(RUNNER) == binding["runner_sha256"]
        summary = {"attempts": summaries, "executables_unchanged": unchanged}
        (output / "summary.json").write_text(json.dumps(summary, indent=2))
        return 0 if unchanged and len(summaries) == 8 and all(row["exit"] == 0 and row["complete_cli_captured"] for row in summaries) else 1
    finally:
        archive = ROOT / (output.name + ".tar.gz")
        with tarfile.open(archive, "w:gz") as package:
            package.add(output, arcname=output.name)
        print(f"HGNC_RECORDS_READY {archive}", flush=True)


def check_parser():
    # These are the two shapes written by runner/transcript.rs, not two executions.
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        transcripts = root / "transcript"
        output = root / "output"
        transcripts.mkdir()
        output.mkdir()
        (transcripts / "metadata.json").write_text(json.dumps({
            "kind": "trial", "name": TRIAL, "uncaptured": True}))
        stdout = b'{"status":"complete"}'
        (transcripts / "execution.json").write_text(json.dumps({
            "key": "trial/command/1", "executions": [{"exit": 0, "stdout": list(stdout)}]}))
        actual = capture_cli(transcripts, output)
        assert actual == [{"file": "cli-1.json", "exit": 0, "status": "complete"}], actual
        assert (output / "cli-1.json").read_bytes() == stdout
    print("mixed trial metadata and CLI execution transcript: ok")


if __name__ == "__main__":
    if sys.argv[1:] == ["--check-parser"]:
        check_parser()
    else:
        raise SystemExit(main())
