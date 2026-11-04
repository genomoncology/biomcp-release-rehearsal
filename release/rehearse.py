"""Rehearse Linux release stages on cold containers; retain every stage failure.

Native macOS, Windows, Homebrew and complete publication staging remain in the
five-platform hosted workflow. This command publishes nothing and reads no keys.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def run_stage(rows, name, command):
    print('Release stage: ' + name, flush=True)
    try:
        result = subprocess.run(command, check=False)
        code = result.returncode
    except OSError as error:
        print(str(error), file=sys.stderr, flush=True)
        code = 127
    rows.append({'stage': name, 'exit': code})


def finish(rows, report):
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps(rows, indent=2) + '\n')
    print(json.dumps({'stages': rows, 'failures': [row for row in rows if row['exit']]}), flush=True)
    return int(any(row['exit'] for row in rows))


def mutation(report, arguments):
    """Run one mutation action in its native job and record its exit status."""
    rows = []
    arguments = arguments[1:] if arguments[:1] == ['--'] else arguments
    run_stage(rows, 'mutation', [sys.executable,
              str(Path(__file__).with_name('mutations.py')), *arguments])
    return finish(rows, report)


def inside(report):
    python = sys.executable
    revision = os.environ['BIOMCP_BUILD_REVISION']
    rows = []
    stages = [
        ('toolchain', [python, 'release/toolchain.py', 'install-rust']),
        ('tool-pins', [python, 'release/toolchain.py', 'check']),
        ('python', [python, 'release/toolchain.py', 'python-check']),
        ('identity', [python, 'release/workflow.py', 'identity', '--event',
                      'workflow_dispatch', '--revision', revision, '--tag', 'rehearsal']),
        ('deny-install', ['cargo', 'install', 'cargo-deny', '--locked', '--version',
                          json.loads(Path('release/toolchain.json').read_text())['cargo-deny']]),
        ('audit', ['make', 'audit']),
        ('secrets', [python, 'release/secrets.py', 'history', '.']),
        ('build', [python, 'release/artifacts.py', 'build', '--manylinux',
                   '--revision', revision, '--output', 'dist']),
        ('prepare', [python, 'release/gate.py', '--prepare-only', '--artifacts',
                     'dist', '--scratch', 'proof']),
        ('schema', ['make', 'schema-release-check']),
    ]
    for name, command in stages:
        run_stage(rows, name, command)
    return finish(rows, report)


def cold(report, container_args):
    if __package__:
        from .toolchain import manifest
        from .source_revision import revision as source_revision
    else:
        from toolchain import manifest
        from source_revision import revision as source_revision
    # Clone only tracked source and history, with no host HOME, caches or tools.
    revision = source_revision()
    rows = []
    report = report.resolve()
    report.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='biomcp-cold-') as directory:
        for arch, wheel_arch in [('amd64', 'x86_64'), ('arm64', 'aarch64')]:
            source = Path(directory) / arch
            subprocess.run(['git', 'clone', '--no-hardlinks', '--quiet', '.', str(source)], check=True)
            subprocess.run(['git', '-C', str(source), 'checkout', '--quiet', '--detach', revision], check=True)
            # The Linux recipe starts a new container with no Cargo cache mount.
            env = {key: value for key, value in os.environ.items()
                   if key in ('PATH', 'HOME', 'DOCKER_HOST', 'DOCKER_CONTEXT',
                              'DOCKER_CONFIG', 'XDG_RUNTIME_DIR')}
            env.update(BUILD_IMAGE=manifest()['images']['manylinux-' + wheel_arch],
                       BIOMCP_RELEASE_COLD='1', BIOMCP_BUILD_REVISION=revision)
            stage_report = report.with_name(report.stem + '-' + arch + '.json')
            env['BIOMCP_RELEASE_REPORT'] = str(stage_report)
            command = ['sh', 'release/linux.sh', '--platform', 'linux/' + arch, *container_args]
            print('Cold Linux architecture: ' + arch, flush=True)
            result = subprocess.run(command, cwd=source, env=env, check=False)
            if stage_report.exists():
                rows.extend({**row, 'platform': arch} for row in json.loads(stage_report.read_text()))
            inner = source / 'dist/cold-build-report.json'
            if inner.exists():
                rows.extend({**row, 'platform': arch} for row in json.loads(inner.read_text()))
            rows.append({'stage': 'linux-recipe', 'platform': arch, 'exit': result.returncode})
            shutil.copytree(source / 'dist', report.parent / ('cold-artifacts-' + arch), dirs_exist_ok=True)
    return finish(rows, report)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['cold', 'inside', 'record', 'finish', 'mutation'])
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('arguments', nargs='*')
    args, extra = parser.parse_known_args()
    args.arguments.extend(extra)
    if args.command == 'mutation':
        return mutation(args.report, args.arguments)
    if args.command == 'inside':
        return inside(args.report)
    if args.command == 'cold':
        return cold(args.report, args.arguments[1:] if args.arguments[:1] == ['--'] else args.arguments)
    rows = json.loads(args.report.read_text()) if args.report.exists() else []
    if args.command == 'record':
        rows.append({'stage': args.arguments[0], 'exit': int(args.arguments[1])})
        args.report.write_text(json.dumps(rows, indent=2) + '\n')
        return 0
    return finish(rows, args.report)


if __name__ == '__main__':
    sys.exit(main())
