"""Exercise the source capture command against an offline HTTP source."""
import hashlib
import http.server
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import threading
import unittest

SCRIPT = Path(__file__).with_name('source_capture.py').resolve()
ROOT = SCRIPT.parent.parent


class Source(http.server.BaseHTTPRequestHandler):
    calls = []

    def do_GET(self):
        self.server.calls.append(self.path)
        status = 429 if self.path == '/stop' else 200
        if self.path == '/redirect':
            status = 302
        self.send_response(status)
        if status == 302:
            self.send_header('Location', '/never')
        self.send_header('Content-Type', 'application/json')
        if self.path == '/sensitive-headers':
            self.send_header('sEt-CoOkIe', 'session=synthetic\r\n folded-synthetic-cookie')
            self.send_header('Authorization', 'synthetic-authorization')
            self.send_header('Proxy-Authorization', 'synthetic-proxy')
            self.send_header('X-Api-Key', 'synthetic-key')
            self.send_header('X-Ordinary', 'retained')
        if self.path == '/headers':
            for index in range(12):
                self.send_header('X-Padding-' + str(index), 'x' * 8192)
        self.end_headers()
        if self.path == '/escaped':
            secret = self.headers.get('X-Key', '')
            escaped = ''.join('\\u%04x' % ord(char) for char in secret)
            self.wfile.write(('{' + '"key":"' + escaped + '"}').encode())
        elif self.path == '/attached':
            attached = self.headers.get('x-api-key') == 'synthetic-credential'
            self.wfile.write(json.dumps({'attached': attached}).encode())
        elif self.path == '/reflect':
            self.wfile.write(self.headers.get('X-Key', '').encode())
        elif self.path == '/large':
            self.wfile.write(b'x' * 8192)
        else:
            self.wfile.write(b'{"rows":[]}')

    def do_POST(self):
        body = self.rfile.read(int(self.headers['Content-Length']))
        self.server.calls.append((self.path, body))
        self.send_response(200)
        self.end_headers()
        self.wfile.write(b'{}')

    def log_message(self, *_args):
        pass


class CapturePlanTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Source)
        self.server.calls = []
        thread = threading.Thread(target=self.server.serve_forever)
        thread.start()
        self.addCleanup(thread.join)
        self.addCleanup(self.server.server_close)
        self.addCleanup(self.server.shutdown)
        self.scratch = self.base / 'tmp'
        self.scratch.mkdir()
        self.env = dict(os.environ, TMPDIR=str(self.scratch))

    def plan(self, paths):
        url = 'http://127.0.0.1:' + str(self.server.server_port)
        value = {'deadline_seconds': 2, 'max_bytes': 4096, 'spacing_seconds': 0,
                 'requests': [{'id': str(n), 'source': 'stub', 'url': url + path,
                               'response_format': 'json'} for n, path in enumerate(paths)]}
        path = self.base / 'plan.json'
        path.write_text(json.dumps(value))
        return path

    def run_capture(self, plan, out, *args, env=None):
        return subprocess.run(['python3', str(SCRIPT), '--plan', str(plan),
                               '--out', str(out), *args], env=env or self.env,
                              capture_output=True, text=True)

    def test_full_run_retains_and_verifies_archive_before_cleanup(self):
        out = self.base / 'missing' / 'parents' / 'capture'
        self.assertFalse(out.parent.exists())
        result = self.run_capture(self.plan(['/one', '/two']), out)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.server.calls, ['/one', '/two'])
        self.assertEqual((out / '0000.body').read_bytes(), b'{"rows":[]}')
        archive = out.with_suffix('.tar.gz')
        result = subprocess.run(['python3', str(SCRIPT), '--verify', str(archive)],
                                env=self.env, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(out.exists())
        with tarfile.open(archive) as packed:
            names = [member.name for member in packed.getmembers()]
        expected = set(json.loads((out / 'manifest.json').read_text())) | {'manifest.json'}
        self.assertEqual(set(names), expected)
        self.assertEqual(len(names), len(expected))

    def test_sensitive_response_headers_and_folds_never_enter_evidence(self):
        out = self.base / 'capture'
        result = self.run_capture(self.plan(['/sensitive-headers']), out)
        self.assertEqual(result.returncode, 0, result.stderr)
        headers = (out / '0000.headers').read_bytes().lower()
        for forbidden in (b'set-cookie', b'authorization', b'x-api-key', b'synthetic'):
            self.assertNotIn(forbidden, headers)
        self.assertIn(b'x-ordinary: retained', headers)
        with tarfile.open(out.with_suffix('.tar.gz')) as packed:
            self.assertEqual(packed.extractfile('0000.headers').read().lower(), headers)

    def test_named_key_attachment_matches_actual_curl_receiver_without_value_evidence(self):
        for keyed in (False, True):
            plan = self.plan(['/attached'])
            value = json.loads(plan.read_text())
            if keyed:
                value['requests'][0]['key'] = {'env': 'CAPTURE_TEST_KEY', 'header': 'x-api-key'}
            plan.write_text(json.dumps(value))
            out = self.base / ('keyed' if keyed else 'keyless')
            env = dict(self.env, CAPTURE_TEST_KEY='synthetic-credential')
            result = self.run_capture(plan, out, env=env)
            self.assertEqual(result.returncode, 0, result.stderr)
            receipt = json.loads((out / '0000.receipt.json').read_text())
            self.assertEqual(receipt['credential_header_attached'], keyed)
            self.assertEqual(receipt['credential_header'], 'x-api-key' if keyed else None)
            self.assertEqual(json.loads((out / '0000.body').read_text())['attached'], keyed)
            for path in out.iterdir():
                self.assertNotIn(b'synthetic-credential', path.read_bytes())

    @unittest.skipUnless(sys.platform == 'darwin', 'AppleDouble requires macOS tar')
    def test_archive_verification_rejects_hidden_appledouble_members(self):
        evidence = self.base / 'metadata'
        evidence.mkdir()
        payload = evidence / 'payload'
        payload.write_bytes(b'{}')
        subprocess.run(['xattr', '-w', 'com.biomcp.capture-test',
                        'outside-in-witness', str(payload)], check=True)
        (evidence / 'manifest.json').write_text(json.dumps(
            {'payload': hashlib.sha256(b'{}').hexdigest()}))
        archive = self.base / 'metadata.tar.gz'
        env = {name: value for name, value in self.env.items() if name != 'COPYFILE_DISABLE'}
        subprocess.run(['tar', '-czf', str(archive), '-C', str(evidence),
                        'payload', 'manifest.json'], env=env, check=True)
        with tarfile.open(archive) as packed:
            self.assertIn('._payload', packed.getnames())
        result = subprocess.run(['python3', str(SCRIPT), '--verify', str(archive)],
                                env=self.env, capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)

    def test_rate_limit_stops_and_continuation_sends_only_undelivered(self):
        plan = self.plan(['/one', '/stop', '/three'])
        first, second = self.base / 'first', self.base / 'second'
        result = self.run_capture(plan, first)
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertEqual(self.server.calls, ['/one', '/stop'])
        result = self.run_capture(plan, second, '--continue-from', str(first))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.server.calls, ['/one', '/stop', '/three'])
        receipt = json.loads((second / '0001.receipt.json').read_text())
        self.assertEqual(receipt['status'], 429)

    def test_preflight_refuses_existing_output_and_missing_os_variable(self):
        plan = self.plan(['/one'])
        for name, env in [('existing', self.env),
                          ('missing', {k: v for k, v in self.env.items() if k != 'TMPDIR'})]:
            with self.subTest(name=name):
                out = self.base / name
                if name == 'existing':
                    out.mkdir()
                    (out / 'retained.body').write_bytes(b'existing capture')
                result = self.run_capture(plan, out, env=env)
                self.assertEqual(result.returncode, 1)
                self.assertEqual(self.server.calls, [])
                if name == 'existing':
                    self.assertIn('output directory and archive must not exist', result.stderr)
                    self.assertEqual(list(out.iterdir()), [out / 'retained.body'])
                    self.assertEqual((out / 'retained.body').read_bytes(), b'existing capture')
                    self.assertFalse(out.with_suffix('.tar.gz').exists())
                if name == 'missing':
                    self.assertIn('missing required OS variable: TMPDIR', result.stderr)

    def test_preflight_refuses_resolved_temporary_output_before_request(self):
        plan = self.plan(['/one'])
        alias = self.base / 'scratch-alias'
        alias.symlink_to(self.scratch, target_is_directory=True)
        for out, temporary in [(self.scratch, self.scratch),
                               (self.scratch / 'capture', self.scratch),
                               (alias / 'capture', self.scratch),
                               (self.scratch / 'capture', alias)]:
            with self.subTest(out=out, temporary=temporary):
                result = self.run_capture(plan, out, env=dict(self.env, TMPDIR=str(temporary)))
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                self.assertIn('durable location outside TMPDIR', result.stderr)
                self.assertEqual(self.server.calls, [])
                self.assertFalse(Path(str(out) + '.tar.gz').exists())

    def test_manifest_rejects_one_changed_byte_before_continuation(self):
        plan = self.plan(['/one'])
        first = self.base / 'first'
        self.assertEqual(self.run_capture(plan, first).returncode, 0)
        (first / '0000.body').write_bytes(b'{"rows":[1]}')
        result = self.run_capture(plan, self.base / 'second', '--continue-from', str(first))
        self.assertEqual(result.returncode, 1)
        self.assertEqual(self.server.calls, ['/one'])
        archive = self.base / 'changed.tar.gz'
        subprocess.run(['tar', '-czf', str(archive), '-C', str(first),
                        *sorted(p.name for p in first.iterdir())], check=True)
        result = subprocess.run(['python3', str(SCRIPT), '--verify', str(archive)],
                                env=self.env, capture_output=True, text=True)
        self.assertEqual(result.returncode, 1)

    def test_bounds_and_key_reflection_never_leak_or_follow(self):
        for route in ['/redirect', '/large', '/reflect', '/escaped']:
            with self.subTest(route=route):
                plan = self.plan([route, '/never'])
                value = json.loads(plan.read_text())
                value['requests'][0]['key'] = {'env': 'CAPTURE_TEST_KEY', 'header': 'X-Key'}
                plan.write_text(json.dumps(value))
                out = self.base / route[1:]
                env = dict(self.env, CAPTURE_TEST_KEY='placeholder-capture-secret')
                result = self.run_capture(plan, out, env=env)
                self.assertEqual(result.returncode, 2, result.stderr)
                self.assertNotIn('placeholder-capture-secret', result.stdout + result.stderr)
                for path in out.iterdir():
                    self.assertNotIn(b'placeholder-capture-secret', path.read_bytes())
                self.assertLessEqual(len((out / '0000.body').read_bytes()), 4096)
                if route in ('/reflect', '/escaped'):
                    self.assertEqual((out / '0000.body').read_bytes(), b'')
                    self.assertEqual(json.loads((out / '0000.receipt.json').read_text())
                                     ['outcome'], 'transport_failure')
        self.assertEqual(self.server.calls, ['/redirect', '/large', '/reflect', '/escaped'])

    def test_literal_post_and_url_are_sent_once_without_rewriting(self):
        plan = self.plan(['/query?value=[1-2]'])
        value = json.loads(plan.read_text())
        body = json.dumps({'query': 'café'}, ensure_ascii=False)
        value['requests'][0].update(method='POST', body=body)
        plan.write_text(json.dumps(value))
        result = self.run_capture(plan, self.base / 'post')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.server.calls, [('/query?value=[1-2]', body.encode())])

    def test_oversized_headers_stop_with_bounded_evidence(self):
        out = self.base / 'headers'
        result = self.run_capture(self.plan(['/headers', '/never']), out)
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertEqual(self.server.calls, ['/headers'])
        self.assertLessEqual((out / '0000.headers').stat().st_size, 65536)
        self.assertLessEqual((out / '0000.body').stat().st_size, 4096)

    def test_make_target_runs_one_request(self):
        result = subprocess.run(['make', 'source-capture', 'PLAN=' + str(self.plan(['/make'])),
                                 'OUT=' + str(self.base / 'make')], cwd=ROOT,
                                env=self.env, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.server.calls, ['/make'])


if __name__ == '__main__':
    unittest.main()
