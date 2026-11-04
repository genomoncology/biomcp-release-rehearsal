"""Run the routine gate in order and report stage costs on success or failure."""
import subprocess
import sys
import time

STAGES = ('pre-job', 'workflow-lint', 'docs-check', 'tensor-proto-check',
          'audit', 'secrets', 'library-tests', 'coverage')


def main():
    started = time.monotonic()
    status = 0
    try:
        for stage in STAGES:
            # Coverage's prerequisites were timed above in this same gate.
            options = ['-o', 'docs-check', '-o', 'library-tests'] if stage == 'coverage' else []
            status = subprocess.run([sys.executable, 'scripts/timed_stage.py', stage,
                                     'make', '--no-print-directory', *options, stage]).returncode
            if status:
                return status
        return 0
    finally:
        print(f'gate total_seconds={time.monotonic() - started:.3f} exit={status}', flush=True)


if __name__ == '__main__':
    raise SystemExit(main())
