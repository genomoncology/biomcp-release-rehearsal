"""Run the short landing check, or the full suite with --full, and report stage costs."""
import subprocess
import sys
import time

# Rights, audit and secrets take seconds and need no build, so they stop a bad
# change before the compile in pre-job.
LANDING = ('rights', 'audit', 'secrets', 'pre-job', 'workflow-lint', 'cases')
FULL = ('audit', 'secrets-full', 'pre-job', 'workflow-lint', 'docs-check',
        'tensor-proto-check', 'library-tests', 'coverage')


def main():
    stages = FULL if sys.argv[1:] == ['--full'] else LANDING
    started = time.monotonic()
    status = 0
    try:
        for stage in stages:
            # Coverage's prerequisites were timed above in this same run.
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
