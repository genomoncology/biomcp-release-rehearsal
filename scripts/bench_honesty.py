"""Bounded current-source replay probes; retain complete CLI and MCP replies."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(sys.argv[2]).resolve() if len(sys.argv) > 2 else Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from release.smoke import environment


def main():
    binary = Path(sys.argv[1]).resolve()
    output = Path.home() / 'workspace/t' / ('biomcp-0678-' + subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip())
    output.mkdir(parents=True, exist_ok=True)
    binary_digest = hashlib.sha256(binary.read_bytes()).hexdigest()
    version_root = output / 'version-environment'
    version_root.mkdir(exist_ok=True)
    version = subprocess.check_output([str(binary), 'version', '--verbose'], env=environment(version_root), text=True)
    (output / 'version.txt').write_text(version)
    probes = [
        ('trial-search', ['search', 'trial', '--condition', 'melanoma', '--offset', '10'],
         'search_trial', {'condition': ['melanoma'], 'offset': 10}, None),
        ('trial-section', ['get', 'trial', 'NCT01844505', 'results', '--offset', '15'],
         'get_trial', {'input': ['NCT01844505'], 'section': ['results'], 'offset': 15}, None),
        ('faers-or', ['search', 'adverse-event', '--drug', 'imatinib OR nilotinib', '--limit', '1'],
         'search_adverse_event', {'drug': ['imatinib OR nilotinib'], 'limit': 1}, 'adverse-event/faers-several-names'),
        ('resolver', ['get', 'variant', 'CA123643', '--resolver', 'variantvalidator', '--build', 'GRCh38'],
         'get_variant', {'input': 'CA123643', 'resolver': 'variantvalidator', 'build': 'GRCh38'}, 'variant/id-forms'),
        ('population-resolver', ['variant', 'population', '7-117559590-ATCT-A', '--resolver', 'variantvalidator'],
         'variant_population', {'input': '7-117559590-ATCT-A', 'resolver': 'variantvalidator'}, 'variant/id-forms'),
        ('clingen', ['get', 'gene', 'MLH1', 'clingen'],
         'get_gene', {'input': ['MLH1'], 'section': ['clingen']}, 'gene/clingen'),
        ('diagnostic', ['search', 'diagnostic', '--source', 'fda-pma', '--manufacturer', 'Foundation Medicine'],
         'search_diagnostic', {'source': 'fda-pma', 'manufacturer': 'Foundation Medicine'}, 'diagnostic/device-approvals'),
    ]
    with tempfile.TemporaryDirectory(prefix='biomcp-0678-') as temporary:
        for label, args, tool, fields, replay in probes:
            if len(sys.argv) > 3 and label not in sys.argv[3].split(','):
                continue
            root = Path(temporary) / label
            root.mkdir()
            env = environment(root)
            config = root / 'config/biomcp'
            config.mkdir(parents=True)
            globals_ = ['--replay', str(ROOT / 'tests/replies' / replay)] if replay else ['--offline']
            recording = next((arg.split('=', 1)[1] for arg in sys.argv[4:] if arg.startswith('--faers-recording=')), None)
            if label == 'faers-or' and recording:
                globals_ = ['--replay', recording]
            live_faers = label == 'faers-or' and '--live-faers' in sys.argv[4:]
            if live_faers:
                globals_ = ['--record', str(output / 'faers-native')]
            result = subprocess.run([str(binary), *globals_, *args], env=env,
                                    capture_output=True, timeout=60)
            (output / f'{label}-cli.json').write_bytes(result.stdout)
            (output / f'{label}-cli.stderr').write_bytes(result.stderr)
            if live_faers:
                globals_ = ['--replay', str(output / 'faers-native')]
            for mode in ('single', 'typed'):
                (config / 'config.json').write_text(json.dumps({'mcp_tool_mode': mode}))
                params = {'name': tool, 'arguments': fields} if mode == 'typed' else {
                    'name': 'biomcp', 'arguments': {'args': args}}
                frames = [
                    {'jsonrpc': '2.0', 'id': 0, 'method': 'initialize', 'params': {
                        'protocolVersion': '2024-11-05', 'capabilities': {},
                        'clientInfo': {'name': 'bounded-probe', 'version': '1'}}},
                    {'jsonrpc': '2.0', 'method': 'notifications/initialized'},
                    {'jsonrpc': '2.0', 'id': 1, 'method': 'tools/call', 'params': params},
                ]
                result = subprocess.run([str(binary), *globals_, 'serve'], env=env,
                                        input=''.join(json.dumps(f) + '\n' for f in frames).encode(),
                                        capture_output=True, timeout=60)
                (output / f'{label}-{mode}.jsonl').write_bytes(result.stdout)
                (output / f'{label}-{mode}.stderr').write_bytes(result.stderr)
                rows = [json.loads(line) for line in result.stdout.splitlines()]
                reply = next(row for row in rows if row.get('id') == 1)
                body = reply.get('result', {}).get('structuredContent', reply.get('error'))
                summary = {key: body.get(key) for key in ('schema', 'status', 'query', 'page', 'error') if isinstance(body, dict) and key in body}
                if isinstance(body, dict):
                    summary['sources'] = [{key: row.get(key) for key in ('source', 'state', 'reason')} for row in body.get('sources', [])]
                print(json.dumps({'probe': label, 'mode': mode, 'reply': summary}), flush=True)
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == binary_digest, 'probe executable changed during the run'
    (output / 'bindings.json').write_text(json.dumps({
        'source_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'binary_sha256': binary_digest,
        'live_check': 'exclude: bounded diagnostic script, not a case; fixture classes are synthetic variant routing and FAERS header bodies plus retained ClinGen/FDA replies; --live-faers replaces only FAERS with one native CLI capture then replays both MCP modes',
        'live_faers': '--live-faers' in sys.argv[4:],
        'faers_recording': next((arg.split('=', 1)[1] for arg in sys.argv[4:] if arg.startswith('--faers-recording=')), None),
        'faers_origin': 'native public capture or its replay' if any(arg == '--live-faers' or arg.startswith('--faers-recording=') for arg in sys.argv[4:]) else 'synthetic routing fixture',
        'fixture_paths': [replay for *_, replay in probes if replay],
    }, indent=2) + '\n')
    print('Evidence:', output)


if __name__ == '__main__':
    main()
