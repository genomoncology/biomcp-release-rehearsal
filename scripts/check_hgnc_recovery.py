"""Build once and exercise HGNC recovery through copied, installed executables."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from release.artifacts import cargo
from scripts.trials import run_selected


def main():
    os.chdir(ROOT)
    product = Path(cargo("dev")["executable"])
    runner = Path(cargo("dev", tests="tools")["executable"])
    installed = ROOT / "hgnc-installed"
    if installed.exists():
        shutil.rmtree(installed)
    installed.mkdir()
    binary = installed / product.name
    tool = installed / runner.name
    shutil.copy2(product, binary)
    shutil.copy2(runner, tool)
    shutil.copytree(ROOT / "tests", installed / "fixtures")
    environment = {**os.environ, "BIOMCP_TEST_EXECUTABLE": str(binary),
                   "BIOMCP_TEST_FIXTURES": str(installed / "fixtures"),
                   "BIOMCP_TRANSCRIPT_DIR": str(installed / "hgnc-transcript")}
    print(json.dumps({"source": os.environ.get("BIOMCP_BUILD_REVISION"),
                      "product": str(binary), "runner": str(tool),
                      "fixtures": str(installed / "fixtures")}), flush=True)
    listing = subprocess.run([str(tool), "--live-check", "--list"], cwd=installed,
                             env=environment, check=True, capture_output=True)
    (installed / "old-live-list.json").write_bytes(listing.stdout)
    rows = json.loads(listing.stdout)["cases"]
    print(json.dumps({"old_live_rows": [row for row in rows
          if row["case"].endswith(("grammar.yaml", "failure-500.yaml"))]}), flush=True)
    names = ["hgnc-first-snapshot-recovers"] if len(sys.argv) == 1 else sys.argv[1:]
    print(json.dumps({"free_bytes_before_proof": shutil.disk_usage(ROOT).free}), flush=True)
    os.environ.update(environment)
    run_selected([str(tool), "--cases", str(installed / "fixtures/cases")],
                 installed, ["cache-resilience/" + name for name in names],
                 prepare=lambda: None)



if __name__ == "__main__":
    main()
