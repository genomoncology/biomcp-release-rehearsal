"""Exercise Make's worker defaults through a recording Cargo executable."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
CARGO = r'''#!/usr/bin/env python3
import json, os, sys
print(json.dumps({"args": sys.argv[1:], "jobs": os.environ.get("CARGO_BUILD_JOBS"),
                  "threads": os.environ.get("RUST_TEST_THREADS")}))
# Stop docs-check after recording its Cargo command.
if os.environ.get("RECORD_DOCS") and "grammar::reference::reference_matches_grammar" in sys.argv:
    sys.exit(3)
'''


def main():
    with tempfile.TemporaryDirectory() as directory:
        tools = Path(directory)
        cargo = tools / "cargo"
        cargo.write_text(CARGO)
        cargo.chmod(0o755)
        base = {key: value for key, value in os.environ.items() if key not in {
            "PM_JOB_CORES", "CARGO_BUILD_JOBS", "RUST_TEST_THREADS", "MAKEFLAGS",
            "MFLAGS", "MAKELEVEL", "MAKEOVERRIDES",
        }}
        base["PATH"] = str(tools) + os.pathsep + base["PATH"]
        cases = [
            ({}, None, None),
            ({"PM_JOB_CORES": "2"}, "2", "2"),
            ({"PM_JOB_CORES": "2.25"}, "2", "2"),
            ({"PM_JOB_CORES": "0.5"}, "1", "1"),
            ({"PM_JOB_CORES": "2.25", "CARGO_BUILD_JOBS": "4"}, "4", "2"),
            ({"PM_JOB_CORES": "2", "RUST_TEST_THREADS": "5"}, "2", "5"),
            ({"CARGO_BUILD_JOBS": "3", "RUST_TEST_THREADS": "4"}, "3", "4"),
        ]
        targets = {
            "test": [["test", "--workspace", "--locked"]],
            "plain-cli-build": [["build", "--locked", "-p", "biomcp", "--bin", "biomcp", "--timings"]],
            "ratchets": [
                ["build", "--locked", "-p", "biomcp", "--bin", "biomcp", "--timings"],
                ["test", "--locked", "-p", "biomcp", "--test", "ratchets"],
            ],
            "docs-check": [
                ["test", "--locked", "-p", "biomcp", "--test", "cases", "readme/"],
                ["test", "--locked", "-p", "biomcp", "--bin", "biomcp",
                 "grammar::reference::reference_matches_grammar"],
            ],
            "docs-generate": [["test", "--locked", "-p", "biomcp", "--bin", "biomcp",
                               "grammar::reference::write_reference"]],
        }
        for settings, jobs, threads in cases:
            for overrides, expected_jobs, expected_threads in (
                ([], jobs, threads),
                (["CARGO_BUILD_JOBS=6", "RUST_TEST_THREADS=7"], "6", "7"),
            ):
                for target, commands in targets.items():
                    result = subprocess.run(["make", "--no-print-directory", "-s", target, *overrides],
                                            cwd=ROOT, env=dict(base, RECORD_DOCS="1", **settings),
                                            capture_output=True, text=True)
                    assert result.returncode == (2 if target == "docs-check" else 0), result.stderr
                    actual = [json.loads(line) for line in result.stdout.splitlines()]
                    expected = []
                    for command in commands:
                        arguments = command + (["--", "--test-threads", expected_threads]
                                               if command[0] == "test" and expected_threads else [])
                        if target == "docs-generate":
                            arguments += ["--ignored"] if expected_threads else ["--", "--ignored"]
                        expected.append({"args": arguments, "jobs": expected_jobs, "threads": expected_threads})
                    assert actual == expected, (settings, overrides, target, actual)
    print("Make honors core bookings and caller compiler/test worker overrides")


if __name__ == "__main__":
    main()
