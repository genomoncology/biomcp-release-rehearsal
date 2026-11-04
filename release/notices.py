"""Generate dependency notices from the release lockfile with the pinned tool."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

if __package__:
    from .vendor import environment as vendor_environment
else:
    from vendor import environment as vendor_environment

ROOT = Path(__file__).resolve().parents[1]
NAMES = ('LICENSE', 'THIRD-PARTY-NOTICES.txt')
MARKER = 'mcp-name: io.github.genomoncology/biomcp'


def payloads(directory):
    files = {name: (directory / name).read_bytes() for name in NAMES}
    validate(files)
    return files


def validate(files):
    for name in NAMES:
        if not files.get(name, b'').strip():
            raise ValueError('artifact requires ' + name)
    if files['LICENSE'] != (ROOT / 'LICENSE').read_bytes():
        raise ValueError('artifact LICENSE differs from the project license')
    if not files[NAMES[1]].startswith(b'Generated third-party notices\n'):
        raise ValueError('artifact requires generated third-party notices')


def metadata(value):
    if value.get('License') != 'MIT' or not value.get_all('Project-URL'):
        raise ValueError('wheel requires License and Project-URL metadata')
    if MARKER not in value.get_payload():
        raise ValueError('wheel description requires ' + MARKER)


def configuration():
    import tomllib
    policy = tomllib.loads((ROOT / 'deny.toml').read_text())['licenses']
    config = {'accepted': policy['allow']}
    for entry in policy.get('exceptions', []):
        config.setdefault(entry['name'], {'accepted': []})['accepted'] += entry['allow']
    # Derive the tool's policy from the existing reviewed dependency policy.
    lines = ['accepted = ' + json.dumps(config.pop('accepted')), 'workarounds = ["ring"]',
             'ignore-dev-dependencies = true', 'ignore-build-dependencies = true']
    for name, entry in sorted(config.items()):
        lines += ['[' + name + ']', 'accepted = ' + json.dumps(sorted(set(entry['accepted'])))]
    return '\n'.join(lines) + '\n'


def generate(output):
    version = json.loads((ROOT / 'release/toolchain.json').read_text())['cargo-about']
    try:
        installed = subprocess.check_output(['cargo', 'about', '--version'], text=True).strip()
    except subprocess.CalledProcessError:
        installed = ''
    if installed != 'cargo-about ' + version:
        subprocess.run(['cargo', 'install', 'cargo-about', '--locked', '--version', version, '--features', 'cli'], check=True)
    with tempfile.TemporaryDirectory(prefix='biomcp-notices-') as temp:
        settings = Path(temp) / 'about.toml'
        settings.write_text(configuration())
        command = ['cargo', 'about', 'generate', '--locked', '--workspace', '--fail',
                   '--config', str(settings), '--format', 'json', '--output-file', str(Path(temp) / 'licenses.json')]
        subprocess.run(command, cwd=ROOT, check=True, env=vendor_environment())
        data = json.loads((Path(temp) / 'licenses.json').read_text())
    if not data.get('licenses') or not data.get('crates'):
        raise ValueError('dependency notices must not be empty')
    lock = hashlib.sha256((ROOT / 'Cargo.lock').read_bytes()).hexdigest()
    lines = ['Generated third-party notices', 'cargo-about ' + version, 'Cargo.lock SHA-256: ' + lock, '']
    for license in data['licenses']:
        users = sorted(item['crate']['name'] + ' ' + item['crate']['version'] for item in license['used_by'])
        lines += [license['name'] + ' (' + license['id'] + ')', ', '.join(users), license['text'], '']
    (output / 'LICENSE').write_bytes((ROOT / 'LICENSE').read_bytes())
    (output / NAMES[1]).write_bytes('\n'.join(lines).encode('utf-8'))
    return payloads(output)
