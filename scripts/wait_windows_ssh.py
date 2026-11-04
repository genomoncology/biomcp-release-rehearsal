"""Wait at most 90 seconds for authenticated guest SSH, including boot resets."""
import subprocess
import time

end = time.monotonic() + 90
attempts = 0
while time.monotonic() < end:
    attempts += 1
    try:
        result = subprocess.run(
            ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=3", "-p", "2222",
             "Docker@localhost", "cmd /c ver"], capture_output=True,
            timeout=max(0.01, min(5, end - time.monotonic())))
    except subprocess.TimeoutExpired:
        continue
    if result.returncode == 0:
        print(f"Windows authenticated SSH ready after {attempts} probes", flush=True)
        break
else:
    raise SystemExit("Windows authenticated SSH did not become ready within 90 seconds")
