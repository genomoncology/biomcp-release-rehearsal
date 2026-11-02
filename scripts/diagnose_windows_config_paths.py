"""Build once and capture only the two installed config-path cases on Windows."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(r"C:\h\biomcp-0573")
revision = sys.argv[1]
records = ROOT / "records"
records.mkdir()
installed = ROOT / "installed"
installed.mkdir()
command = ["cargo", "test", "--locked", "-p", "biomcp", "--test", "cases",
           "--no-run", "--message-format=json"]
artifacts = {}
with (records / "build.jsonl").open("w") as log:
    process = subprocess.Popen(command, stdout=subprocess.PIPE, text=True)
    for line in process.stdout:
        log.write(line)
        message = json.loads(line)
        if message.get("reason") == "compiler-artifact" and message.get("executable"):
            artifacts[message["target"]["name"]] = Path(message["executable"])
    if process.wait():
        raise SystemExit("Windows diagnostic build failed")
product = installed / "biomcp.exe"
runner = installed / "case-runner.exe"
shutil.copy2(artifacts["biomcp"], product)
shutil.copy2(artifacts["cases"], runner)
shutil.copytree(ROOT / "tests", installed / "fixtures")
version = subprocess.check_output([str(product), "version", "--verbose"], text=True)
if f"Commit: {revision}\n" not in version:
    raise SystemExit("Installed product has the wrong source revision")
binding = {"source": revision, "product_sha256": hashlib.sha256(product.read_bytes()).hexdigest(),
           "runner_sha256": hashlib.sha256(runner.read_bytes()).hexdigest()}
(records / "binding.json").write_text(json.dumps(binding, indent=2))
print(json.dumps(binding), flush=True)
environment = {**os.environ, "BIOMCP_TEST_EXECUTABLE": str(product),
               "BIOMCP_TEST_FIXTURES": str(installed / "fixtures")}
failed = False
for case in ["housekeeping/config-path-fallbacks", "housekeeping/offline-provenance-and-config"]:
    result = subprocess.run([str(runner), case, "--exact", "--test-threads", "1"],
                            env=environment, capture_output=True)
    name = case.split("/")[1]
    (records / f"{name}.stdout").write_bytes(result.stdout)
    (records / f"{name}.stderr").write_bytes(result.stderr)
    print(result.stdout.decode(errors="replace"), flush=True)
    print(result.stderr.decode(errors="replace"), flush=True)
    failed = failed or result.returncode != 0
raise SystemExit(int(failed))
