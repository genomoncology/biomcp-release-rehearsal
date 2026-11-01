#!/usr/bin/env python3
"""Exercise the runtime container, including installed process and MCP proofs."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

from smoke import SCHEMA_ID, case_suite, mcp_probe, require

RUNNER_SETUP = ('mkdir /tmp/check && '
                'cp /proof/case-runner /proof/tool-runner /tmp/check/ && '
                'cp -R /proof/fixtures /tmp/check/fixtures && exec "$@"')


def main():
    image, arch, proof, version, *extra_args = sys.argv[1:]
    common = ["docker", "run", "--rm", "--memory", "2g", "--cpus", "2", "--platform", "linux/" + arch] + extra_args
    checksum = subprocess.check_output(common + ["--entrypoint", "/usr/bin/sha256sum", image, "/usr/local/bin/biomcp"], text=True).split()[0]
    require(checksum == hashlib.sha256((Path(proof) / "biomcp").read_bytes()).hexdigest(), "container binary differs from the candidate")
    checks = ["candidate-bytes"]
    for name in ('LICENSE', 'THIRD-PARTY-NOTICES.txt'):
        contents = subprocess.check_output(common + ['--entrypoint', '/bin/cat', image,
                                                    '/usr/share/doc/biomcp/' + name])
        require(contents == (Path(proof) / name).read_bytes(), 'container ' + name + ' differs from candidate')
    checks.append('license-notices')
    for args, key in [(["schema", SCHEMA_ID], "$id"), (["help"], "commands")]:
        result = subprocess.run(common + [image] + args, capture_output=True, text=True, check=True)
        value = json.loads(result.stdout)
        require(not result.stderr, "container command wrote stderr")
        require(value["result"][key] if key == "commands" else value[key] == SCHEMA_ID, "container command contract differs")
        checks.append("help" if key == "commands" else "schemas")
    with tempfile.TemporaryDirectory() as root:
        mcp_probe(common + ["-i", image, "serve"], os.environ.copy(), Path(root), version)
    checks.extend(["version", "mcp-all-stdout"])
    executable = common + ["-v", str(Path(proof).resolve()) + ":/proof:ro", "-e", "BIOMCP_TEST_EXECUTABLE=/usr/local/bin/biomcp", "-e", "BIOMCP_TEST_FIXTURES=/tmp/check/fixtures",
                           "--entrypoint", "/bin/sh", image, "-ec", RUNNER_SETUP, "installed-runner"]
    case_suite(executable + ["/tmp/check/tool-runner", "--proofs"],
               os.environ.copy(), Path.cwd(), Path(proof) / "fixtures")
    checks.append("runner-proofs")
    executable += ["/tmp/check/case-runner"]
    summary = case_suite(executable, os.environ.copy(), Path.cwd(), Path(proof) / "fixtures")
    checks.append("all-cases-and-trials")
    print(json.dumps({"artifact": "docker", "platform": "linux/" + arch, "result": "pass", "checks": checks, **summary}))


if __name__ == "__main__":
    main()
