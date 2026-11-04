"""Install a released version from one public channel and verify its identity."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

if __package__:
    from .smoke import environment
else:
    from smoke import environment

IMAGE = 'ghcr.io/genomoncology/biomcp'
SERVER = 'io.github.genomoncology/biomcp'


def run(command, **kwargs):
    return subprocess.check_output(command, text=True, timeout=300, **kwargs)


def fetch(url):
    # Public indexes and Pages can lag publication. Retry only transport/HTTP failures.
    for attempt in range(12):
        try:
            with urllib.request.urlopen(url, timeout=30) as response:
                return response.read()
        except (urllib.error.URLError, TimeoutError):
            if attempt == 11:
                raise
            time.sleep(10)


def check_version(output, version):
    if output != 'biomcp ' + version + '\n':
        raise ValueError('published installation returned a different version')


def pypi(root, version, package='biomcp-cli'):
    venv = root / 'venv'
    subprocess.run([sys.executable, '-m', 'venv', str(venv)], check=True)
    python = str(venv / 'bin/python')
    env = {**environment(root), 'PATH': os.defpath, 'PIP_CONFIG_FILE': os.devnull,
           'PIP_NO_CACHE_DIR': '1', 'PIP_DISABLE_PIP_VERSION_CHECK': '1'}
    subprocess.run([python, '-m', 'pip', '--isolated', 'install', '--no-deps', '--only-binary=:all:',
                    '--index-url', 'https://pypi.org/simple', package + '==' + version], env=env, cwd=root, check=True)
    return [str(venv / 'bin/biomcp')]


def registry(root, version):
    url = 'https://registry.modelcontextprotocol.io/v0.1/servers/' + urllib.parse.quote(SERVER, safe='')
    record = json.loads(fetch(url + '/versions/' + version))['server']
    if record['name'] != SERVER or record['version'] != version:
        raise ValueError('published MCP Registry server identity differs')
    package, = record['packages']
    if (package['registryType'] != 'pypi' or package['identifier'] != 'biomcp-cli'
            or package['version'] != version or package['runtimeHint'] != 'uvx'
            or package['transport'] != {'type': 'stdio'}
            or package['packageArguments'] != [{'type': 'positional', 'value': 'serve'}]):
        raise ValueError('published MCP Registry install declaration differs')
    # The registry distributes a declaration. Install the package it declares.
    return pypi(root, package['version'], package['identifier'])


def installer(root, version):
    script = root / 'install.sh'
    script.write_bytes(fetch('https://biomcp.org/install.sh'))
    script.chmod(0o700)
    env = {**environment(root), 'PATH': os.defpath, 'BIOMCP_INSTALL_DIR': str(root / 'bin')}
    subprocess.run(['bash', str(script), '--version', version], cwd=root, env=env, check=True)
    return [str(root / 'bin/biomcp')]


def homebrew(root, version):
    # macOS runners include Homebrew. Refresh the actual public tap before install.
    subprocess.run(['brew', 'tap', 'genomoncology/biomcp'], check=True)
    subprocess.run(['brew', 'update'], check=True)
    subprocess.run(['brew', 'install', 'genomoncology/biomcp/biomcp'], check=True)
    prefix = run(['brew', '--prefix', 'genomoncology/biomcp/biomcp']).strip()
    return [str(Path(prefix) / 'bin/biomcp')]


def ghcr(root, version):
    # Verify the published manifest attestation before running either architecture.
    reference = IMAGE + ':' + version
    descriptor = json.loads(run(['docker', 'buildx', 'imagetools', 'inspect', reference,
                                 '--format', '{{json .Manifest}}']))
    digest = descriptor['digest']
    if not re.fullmatch(r'sha256:[0-9a-f]{64}', digest):
        raise ValueError('published manifest digest is invalid')
    pinned = IMAGE + '@' + digest
    subprocess.run(['gh', 'attestation', 'verify', 'oci://' + pinned, '--repo',
                    'genomoncology/biomcp', '--signer-workflow',
                    'genomoncology/biomcp/.github/workflows/release.yml',
                    '--source-ref', 'refs/tags/v' + version, '--source-digest', os.environ['GITHUB_SHA']], check=True)
    subprocess.run(['docker', 'pull', '--platform', 'linux/amd64', pinned], check=True)
    return ['docker', 'run', '--rm', '--platform', 'linux/amd64', pinned]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('channel', choices=['pypi', 'ghcr', 'homebrew', 'installer', 'registry'])
    parser.add_argument('--version', required=True)
    args = parser.parse_args()
    if not re.fullmatch(r'\d+\.\d+\.\d+', args.version):
        raise ValueError('expected a stable release version')
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        command = globals()[args.channel](root, args.version)
        check_version(run(command + ['version'], cwd=root, env={**environment(root), 'PATH': os.defpath}), args.version)
        print(args.channel + ': installed BioMCP ' + args.version)


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
