"""Install a released version from one public channel and verify its identity."""
import argparse
import hashlib
import shutil
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
    from .smoke import environment, mcp_probe
    from .local_server import server
    from .artifacts import candidate_record
else:
    from smoke import environment, mcp_probe
    from local_server import server
    from artifacts import candidate_record

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


# The public 0.9.3 install script has no base-URL setting. Preserve its bytes,
# and redirect only its exact GitHub release downloads through a curl adapter.
LEGACY_REVISION = '5641de7d7b79b53872f8599dbb4dba22e611dcaf'
LEGACY_SHA256 = '8d4592de9b4ac55958b22282620b234fcdeac3f248211265a3da144adf47a34a'


def legacy_installer(root, version, artifacts, revision, script=None):
    data = Path(script).read_bytes() if script else fetch(
        'https://raw.githubusercontent.com/genomoncology/biomcp/' + LEGACY_REVISION + '/install.sh')
    if hashlib.sha256(data).hexdigest() != LEGACY_SHA256:
        raise ValueError('pinned 0.9 installer checksum differs')
    script_path = root / 'install.sh'
    script_path.write_bytes(data)
    record = candidate_record(artifacts, revision, version)
    adapter = root / 'download-bin'
    adapter.mkdir()
    curl = shutil.which('curl')
    if not curl:
        raise ValueError('curl is required for the local legacy installer proof')
    request_log = root / 'downloads.jsonl'
    wrapper = adapter / 'curl'
    wrapper.write_text('#!' + sys.executable + '\n' +
        'import json, os, subprocess, sys\n' +
        'from pathlib import Path\n' +
        'args = sys.argv[1:]\n' +
        'prefix = "https://github.com/genomoncology/biomcp/releases/download/v" + os.environ["BIOMCP_VERSION"] + "/"\n' +
        'urls = [a for a in args if a.startswith("https://")]\n' +
        'if len(urls) != 1 or not urls[0].startswith(prefix): sys.exit("unexpected legacy download URL")\n' +
        'name = urls[0][len(prefix):]\n' +
        'if "/" in name or not name.startswith("biomcp-"): sys.exit("invalid legacy asset")\n' +
        'Path(os.environ["DOWNLOAD_LOG"]).open("a").write(json.dumps(name) + "\\n")\n' +
        'args[args.index(urls[0])] = os.environ["LOCAL_RELEASE"] + "/" + name\n' +
        'sys.exit(subprocess.call([' + repr(curl) + '] + args))\n')
    wrapper.chmod(0o700)
    env = {**environment(root), 'PATH': str(adapter) + os.pathsep + os.defpath,
           'BIOMCP_INSTALL_DIR': str(root / 'bin'), 'BIOMCP_VERSION': version,
           'DOWNLOAD_LOG': str(request_log)}
    with server(artifacts) as base:
        env['LOCAL_RELEASE'] = base
        subprocess.run(['bash', str(script_path), '--version', version], cwd=root, env=env, check=True)
    requested = [json.loads(line) for line in request_log.read_text().splitlines()]
    archives = [name for name in requested if not name.endswith('.sha256')]
    if len(archives) != 1 or requested != [archives[0], archives[0] + '.sha256']:
        raise ValueError('legacy installer did not fetch candidate archive and checksum')
    binary = root / 'bin' / ('biomcp.exe' if os.name == 'nt' else 'biomcp')
    if hashlib.sha256(binary.read_bytes()).hexdigest() != record['binary_sha256']:
        raise ValueError('legacy installation differs from candidate binary')
    print(json.dumps({'channel': 'legacy-installer', 'installer_revision': LEGACY_REVISION,
                      'installer_sha256': LEGACY_SHA256, 'revision': revision,
                      'archive': archives[0], 'archive_sha256': hashlib.sha256(
                          (artifacts / archives[0]).read_bytes()).hexdigest()}))
    return [str(binary)]


def plugin(root, version, source, binary, claude='claude', record=None):
    executable = shutil.which(claude)
    if not executable:
        raise ValueError('Claude CLI is required; plugin channel was not run')
    env = {**environment(root), 'PATH': str(binary.parent) + os.pathsep + os.defpath}
    Path(env['HOME']).mkdir(parents=True, exist_ok=True)
    subprocess.run([executable, 'plugin', 'marketplace', 'add', str(source)], cwd=root, env=env, check=True)
    subprocess.run([executable, 'plugin', 'install', 'biomcp@biomcp', '--scope', 'user'],
                   cwd=root, env=env, check=True)
    registry = Path(env['HOME']) / '.claude/plugins/installed_plugins.json'
    installed, = json.loads(registry.read_text())['plugins']['biomcp@biomcp']
    installed_path = Path(installed['installPath']).resolve()
    installed_path.relative_to(Path(env['HOME']).resolve())
    metadata = json.loads((installed_path / '.claude-plugin/plugin.json').read_text())
    if metadata['version'] != version:
        raise ValueError('installed Claude plugin version differs')
    marketplace = installed_path / '.claude-plugin/marketplace.json'
    entry, = json.loads(marketplace.read_text())['plugins']
    declaration = entry['mcpServers']['biomcp']
    if declaration != {'command': 'biomcp', 'args': ['serve']}:
        raise ValueError('installed plugin MCP launch declaration differs')
    mcp_probe([str(binary), *declaration['args']], env, root, version, call_args=['help'])
    print(json.dumps({'channel': 'plugin', 'installed': 'biomcp@biomcp',
                      'command': ['help'], 'result': 'commands returned',
                      'revision': record['revision'] if record else None,
                      'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest()}))


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
    parser.add_argument('channel', choices=['pypi', 'ghcr', 'homebrew', 'installer', 'registry', 'legacy-installer', 'plugin'])
    parser.add_argument('--version', required=True)
    parser.add_argument('--artifacts', type=Path)
    parser.add_argument('--revision', help='Full artifact source commit, independent of this runner')
    parser.add_argument('--installer-script', type=Path, help='Cached exact pinned public 0.9 installer')
    parser.add_argument('--plugin-source', type=Path)
    parser.add_argument('--binary', type=Path)
    parser.add_argument('--claude', default='claude')
    args = parser.parse_args()
    if not re.fullmatch(r'\d+\.\d+\.\d+', args.version):
        raise ValueError('expected a stable release version')
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        if args.channel == 'plugin':
            if not all([args.plugin_source, args.binary, args.artifacts, args.revision]):
                parser.error('plugin requires --plugin-source, --binary, --artifacts and --revision')
            record = candidate_record(args.artifacts, args.revision, args.version)
            if hashlib.sha256(args.binary.read_bytes()).hexdigest() != record['binary_sha256']:
                raise ValueError('plugin executable differs from candidate identity')
            plugin(root, args.version, args.plugin_source.resolve(), args.binary.resolve(), args.claude, record)
            return
        if args.channel == 'legacy-installer':
            if not args.artifacts or not args.revision or not re.fullmatch('[0-9a-f]{40}', args.revision):
                parser.error('legacy-installer requires --artifacts and full --revision')
            command = legacy_installer(root, args.version, args.artifacts.resolve(), args.revision, args.installer_script)
        else:
            command = globals()[args.channel](root, args.version)
        check_version(run(command + ['version'], cwd=root, env={**environment(root), 'PATH': os.defpath}), args.version)
        print(args.channel + ': installed BioMCP ' + args.version)


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
