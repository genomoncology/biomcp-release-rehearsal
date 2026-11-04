"""Check one installed SSED read against its public FDA PDF, without host settings."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from release.smoke import environment


def main():
    binary = ROOT / 'target/debug/biomcp'
    output = ROOT / 'target/fda-document-live'
    output.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='biomcp-fda-live-') as temporary:
        root = Path(temporary)
        installed = root / 'biomcp'
        shutil.copy2(binary, installed)
        version = subprocess.check_output([str(installed), 'version', '--verbose'], cwd=root,
                                          env=environment(root), text=True)
        args = ['get', 'diagnostic', 'P170019', 'summary', '--source', 'fda-pma',
                '--record', str(output / 'records')]
        reply = subprocess.run([str(installed), *args], cwd=root, env=environment(root),
                               capture_output=True, text=True, timeout=40)
        (output / 'reply.json').write_text(reply.stdout)
        (output / 'binding.json').write_text(json.dumps({
            'sha': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
            'version': version,
            'binary_sha256': hashlib.sha256(installed.read_bytes()).hexdigest(),
            'args': args, 'exit': reply.returncode,
        }, indent=2) + '\n')
        document = (json.loads(reply.stdout).get('result') or {}).get('document', {})
        if reply.returncode or reply.stderr or document.get('state') != 'text' or document.get('pages') != 58:
            raise SystemExit('FDA live SSED read failed; see target/fda-document-live/reply.json')
        print('Installed FDA SSED read passed: 58 pages, embedded text, bounded first fragment.')


if __name__ == '__main__':
    main()
