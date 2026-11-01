"""The install gate runs literal CLI checks against installed candidate bytes."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile

from release.test_licenses import write_payloads

from release.artifacts import binary_archive, platform_tag, wheel


CANDIDATE = r'''import json, os, pathlib, sys
args = sys.argv[1:]
with open(LOG, 'a') as stream:
    stream.write(json.dumps({'binary': sys.argv[0], 'args': args}) + '\n')
if args[0] in ('version', 'health'):
    assert 'ONCOKB_API_KEY' not in os.environ
    config = pathlib.Path(os.environ['APPDATA']) / 'biomcp/config.json'
    disabled = json.loads(config.read_text())['disable_sources']
    assert disabled[:3] == ['open', 'noncommercial', 'restricted']
    assert pathlib.Path.cwd() != pathlib.Path(LOG).parent
    if args[0] == FAIL:
        sys.exit(7)
    if args == ['version']:
        print('biomcp 1.0.0')
    elif args == ['version', '--verbose']:
        print('biomcp 1.0.0\nCommit: ' + ('b' if FAIL == 'commit' else 'a') * 40 + '\nTarget: fixture')
    elif args == ['health', '--offline']:
        print(json.dumps({'status': 'complete', 'result': {'sources': [
            {'source': 'synthetic', 'disabled': 'synthetic' in disabled, 'state': 'skipped',
             'reason': 'disabled' if 'synthetic' in disabled else 'offline'}]}}))
    else:
        sys.exit('unexpected CLI arguments')
elif args[0] == 'schema':
    print(json.dumps({'$id': args[1]}))
elif args[0] == 'help':
    print('{"result":{"commands":["get"]}}')
elif args == ['serve']:
    cache = pathlib.Path(os.environ['XDG_CACHE_HOME'])
    cache.mkdir(parents=True, exist_ok=True)
    (cache / 'reply.sqlite').touch()
    for frame in sys.stdin:
        value = json.loads(frame)
        if 'id' not in value:
            continue
        result = ({'serverInfo': {'version': '1.0.0'}} if value['id'] == 1 else
                  {'tools': [{'name': 'biomcp'}]} if value['id'] == 2 else {})
        print(json.dumps({'jsonrpc': '2.0', 'id': value['id'], 'result': result}))
else:
    sys.exit('provider path is forbidden in this fixture')
'''


@unittest.skipIf(os.name == 'nt', 'synthetic executables use POSIX shebangs')
class InstalledCli(unittest.TestCase):
    def test_installed_cli_rows_bind_results_and_fail_on_cli_exit(self):
        for fail in ('', 'version', 'health', 'commit', 'LICENSE', 'THIRD-PARTY-NOTICES.txt', 'unexpected'):
            with self.subTest(fail=fail), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                output = root / 'dist'
                output.mkdir()
                log = root / 'executed.jsonl'
                binary = root / 'biomcp'
                binary.write_text(f'#!{sys.executable}\nLOG = {str(log)!r}\nFAIL = {fail!r}\n' + CANDIDATE)
                binary.chmod(0o755)
                record = {'distribution': 'biomcp-cli', 'version': '1.0.0', 'platform': platform_tag(False),
                          'profile': 'release', 'cargo_profile': {'debug_assertions': False, 'opt_level': '3'},
                          'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(), 'revision': 'a' * 40}
                write_payloads(output)
                wheel(output, binary, record)
                binary_archive(output, binary, record['platform'])
                manifest = {'name': 'biomcp', 'version': '1.0.0', 'tools': [{'name': 'biomcp'}],
                            'server': {'entry_point': 'server/biomcp', 'mcp_config': {'command': '${__dirname}/server/biomcp', 'args': ['serve']}}}
                with zipfile.ZipFile(output / 'synthetic.mcpb', 'w') as bundle:
                    bundle.writestr('manifest.json', json.dumps(manifest))
                    bundle.writestr('server/biomcp', binary.read_bytes())
                    for name in ('LICENSE', 'THIRD-PARTY-NOTICES.txt'):
                        if fail != name:
                            bundle.writestr(name, (output / name).read_bytes())
                    if fail == 'unexpected':
                        bundle.writestr('unexpected.txt', 'unexpected')
                registry = {'name': 'io.github.genomoncology/biomcp', 'version': '1.0.0', 'packages': [
                    {'registryType': 'pypi', 'identifier': 'biomcp-cli', 'version': '1.0.0', 'transport': {'type': 'stdio'},
                     'packageArguments': [{'type': 'positional', 'value': 'serve'}]}]}
                (output / 'server.json').write_text(json.dumps(registry))
                scratch = root / 'install'
                (scratch / 'fixtures').mkdir(parents=True)
                runner = scratch / 'case-runner'
                runner.write_text(f'#!{sys.executable}\nprint(\"test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\")\n')
                runner.chmod(0o755)
                tool = runner.with_name("tool-runner")
                tool.write_text(runner.read_text())
                tool.chmod(0o755)
                env = {**os.environ, 'ONCOKB_API_KEY': 'synthetic-must-not-inherit'}
                # Homebrew has separate installation proof; this fixture uses the shared gate on five channels.
                env['PATH'] = os.defpath
                result = subprocess.run([sys.executable, 'release/gate.py', '--prepared', '--artifacts', str(output),
                                         '--scratch', str(scratch), '--python', sys.executable], env=env,
                                        capture_output=True, text=True, timeout=120)
                if fail:
                    self.assertNotEqual(result.returncode, 0)
                    message = ('unexpected desktop bundle members' if fail in ('LICENSE', 'THIRD-PARTY-NOTICES.txt', 'unexpected')
                               else 'installed CLI commit differs' if fail == 'commit' else 'installed CLI command failed')
                    self.assertIn(message, result.stderr)
                    continue
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                rows = json.loads((scratch / 'report.json').read_text())
                cli = [row for row in rows if 'command' in row]
                self.assertEqual(len(cli), 15)
                executed = [json.loads(line) for line in log.read_text().splitlines()]
                for artifact in ('wheel', 'desktop-extension', 'mcp-registry', 'install-script', 'download-install-script'):
                    selected = [row for row in cli if row['artifact'] == artifact]
                    self.assertEqual([row['command'] for row in selected], [['version'], ['version', '--verbose'], ['health', '--offline']])
                    self.assertEqual(selected[0]['output'], 'biomcp 1.0.0\n')
                    self.assertEqual(selected[2]['output']['result']['sources'][0]['reason'], 'disabled')
                    for row in selected:
                        self.assertEqual(row['revision'], record['revision'])
                        self.assertEqual(row['binary_sha256'], record['binary_sha256'])
                        self.assertEqual(row['platform'], record['platform'])
                        self.assertEqual(row['result'], 'pass')
                        self.assertTrue(any(call['args'] == row['command'] and call['binary'] != str(binary) for call in executed))
                suites = [row for row in rows if row.get('result') == 'pass' and 'checks' in row]
                self.assertTrue(suites)
                self.assertTrue(all('all-cases-and-trials' in row['checks'] for row in suites))
                self.assertTrue(all(row['trials'] == 2 for row in suites))



if __name__ == '__main__':
    unittest.main()
