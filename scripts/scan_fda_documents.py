"""Compare scoped FDA product and test duplication with a pinned cached scanner."""
import json
from pathlib import Path
import subprocess
import sys


def main():
    baseline = subprocess.check_output(['git', 'rev-parse', sys.argv[1]], text=True).strip()
    current = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    scanner = next((p.parent.parent / '.bin/jscpd'
                    for p in Path.home().glob('.npm/_npx/*/node_modules/jscpd/package.json')
                    if json.loads(p.read_text()).get('version') == '4.0.5'), None)
    if scanner is None:
        raise SystemExit('cached jscpd 4.0.5 is required')
    subprocess.run([str(scanner), '--version'], check=True)
    product = ['crates/biomcp-adapters/src/' + name for name in
               ['fda_pma', 'fda_510k', 'drugsfda', 'pdf', 'device_search.rs']]
    product += ['crates/biomcp-query/src/diagnostic',
                'crates/biomcp-query/src/drug/drugsfda',
                'crates/biomcp-query/src/drug/drugsfda.rs',
                'crates/biomcp-types/src/query/diagnostic',
                'crates/biomcp-types/src/drug/query.rs',
                'crates/biomcp-types/src/drug/fda.rs',
                'crates/biomcp-types/src/response/diagnostic.rs',
                'crates/biomcp-types/src/response/pdf.rs',
                'crates/biomcp/src/retired/diagnostic.rs']
    tests = ['tests/cases/diagnostic', 'crates/biomcp-types/tests/diagnostic_contracts.rs',
             'crates/biomcp/tests/ratchets/dependencies.rs',
             'scripts/check_fda_documents.sh', 'scripts/check_fda_document_live.py',
             'scripts/scan_fda_documents.py']
    output = Path('target/fda-document-scans')
    for side, revision in [('baseline', baseline), ('current', current)]:
        for group, roots in [('product', product), ('tests', tests)]:
            names = subprocess.check_output(
                ['git', 'ls-tree', '-r', '--name-only', revision, '--', *roots], text=True).splitlines()
            names = [name for name in names if Path(name).suffix in {'.rs', '.yaml', '.py', '.sh'}]
            destination = output / side / group
            source = destination / 'source'
            for name in names:
                path = source / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(subprocess.check_output(['git', 'show', revision + ':' + name]))
            result = subprocess.run([str(scanner), str(source), '--format', 'rust,yaml,python,bash',
                                     '--max-size', '10mb', '--reporters', 'json',
                                     '--output', str(destination / 'report')],
                                    capture_output=True, text=True)
            (destination / 'scanner.log').write_text(result.stdout + result.stderr)
            result.check_returncode()
            statistics = json.loads((destination / 'report/jscpd-report.json').read_text())['statistics']['total']
            print(side, group, 'files', len(names), json.dumps(statistics, sort_keys=True))
    print('FDA scan baseline:', baseline, 'current:', current, 'reports:', output)


if __name__ == '__main__':
    main()
