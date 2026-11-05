"""Isolate the offline nightly command and retain its copied diagnostic reports."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]

CARGO = '''#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
assert sys.argv[1:] == ['test','--locked','-p','biomcp','--test','tools','--','--live-check','--details']
print(json.dumps(json.loads(os.environ['FAKE_REPORT'])))
print('offline build log', file=sys.stderr)
sys.exit(int(os.environ.get('FAKE_EXIT', '0')))
'''
SSH = '''#!/usr/bin/env python3
import io, os, subprocess, sys, tarfile
assert sys.argv[-2] == os.environ.get('FAKE_HOST', 'offline-store')
if '# nightly-history' in sys.argv[-1]:
    print(os.environ.get('FAKE_HISTORY', '[]'))
    sys.exit(0)
if os.environ.get('FAKE_COPY') == 'fail':
    sys.exit(23)
payload = sys.stdin.buffer.read()
if os.environ.get('FAKE_COPY') == 'manifest':
    changed = io.BytesIO()
    with tarfile.open(fileobj=io.BytesIO(payload)) as original, tarfile.open(fileobj=changed, mode='w') as replacement:
        for entry in original:
            body = original.extractfile(entry).read()
            if entry.name == 'report.json':
                body += b'corrupt'
                entry.size = len(body)
            replacement.addfile(entry, io.BytesIO(body))
    payload = changed.getvalue()
result = subprocess.run(sys.argv[-1], shell=True, input=payload)
sys.exit(result.returncode)
'''


class NightlyHarness:
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.checkout = self.root / 'checkout'
        (self.checkout / 'scripts').mkdir(parents=True)
        shutil.copy(ROOT / 'Makefile', self.checkout / 'Makefile')
        for name in ('nightly_live_check.py', 'nightly_evidence.py'):
            script = ROOT / 'scripts' / name
            if script.exists():
                shutil.copy(script, self.checkout / 'scripts')
        self.home = self.root / 'home'
        self.scratch_root = self.root / 'scratch'
        self.store = self.root / 'store'
        self.tools = self.root / 'tools'
        self.tools.mkdir()
        for name, content in [('cargo', CARGO), ('ssh', SSH), ('git', '#!/bin/sh\nprintf \"%040d\\n\" 1\n')]:
            target = self.tools / name
            target.write_text(content)
            target.chmod(0o755)
        self.env = dict(os.environ, HOME=str(self.home),
                        XDG_CONFIG_HOME=str(self.home / '.config'),
                        TMPDIR=str(self.root / 'tmp'),
                        BIOMCP_EVIDENCE_HOST='offline-store',
                        BIOMCP_EVIDENCE_DIR=str(self.store),
                        BIOMCP_SCRATCH_DIR=str(self.scratch_root),
                        PATH=str(self.tools) + os.pathsep + os.environ['PATH'],
                        FAKE_REPORT='{"drift_cases":0,"error_cases":0,"eligible":0,"checked":0}')
        Path(self.env['TMPDIR']).mkdir()

    def run_command(self, args=(), **values):
        env = dict(self.env, **values)
        command = ([sys.executable, 'scripts/nightly_live_check.py', *args] if args
                   else ['make', 'nightly-live-check'])
        return subprocess.run(command, cwd=self.checkout,
                              env=env, text=True, capture_output=True, check=False)

    def copied(self):
        runs = list(self.store.glob('*'))
        self.assertEqual(len(runs), 1)
        run = runs[0]
        manifest = json.loads((run / 'manifest.json').read_text())
        for name, digest in manifest.items():
            self.assertEqual(hashlib.sha256((run / name).read_bytes()).hexdigest(), digest)
        self.assertEqual((run / 'build.log').read_text(), 'offline build log\n')
        return run

    def scratch(self):
        return list(self.scratch_root.glob('*'))

    def classified(self, row, expected, exit_code):
        shutil.rmtree(self.store, ignore_errors=True)
        report = {"drift_cases": int(bool(row["result"]["drift"])), "error_cases": 0,
                  "eligible": 1, "checked": 1, "omitted": 0, "case_sample": [row]}
        result = self.run_command(FAKE_REPORT=json.dumps(report))
        self.assertEqual(result.returncode == 0, exit_code == 0, result.stdout + result.stderr)
        saved = json.loads((self.copied() / 'report.json').read_text())
        self.assertEqual(saved['case_sample'][0]['classification'], expected)
        self.assertEqual(saved[expected + '_cases'], 1)

    def local_settings(self, root):
        settings = root / 'biomcp/nightly.json'
        settings.parent.mkdir(parents=True)
        settings.write_text(json.dumps({
            'host': 'offline-store', 'evidence_dir': str(self.store),
            'scratch_dir': str(self.scratch_root)}))
        for name in ('BIOMCP_EVIDENCE_HOST', 'BIOMCP_EVIDENCE_DIR', 'BIOMCP_SCRATCH_DIR'):
            self.env.pop(name, None)
