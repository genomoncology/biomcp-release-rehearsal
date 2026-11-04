"""Run one gate stage and print its elapsed time even when it fails."""
import subprocess
import sys
import time


def main():
    stage, *command = sys.argv[1:]
    started = time.monotonic()
    status = 127
    try:
        status = subprocess.run(command).returncode
    finally:
        print(f"gate stage={stage} elapsed_seconds={time.monotonic() - started:.3f} exit={status}", flush=True)
    return status if status >= 0 else 128 - status


if __name__ == '__main__':
    raise SystemExit(main())
