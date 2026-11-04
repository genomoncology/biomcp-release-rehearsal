"""Exercise binary object and child profile selection through the coverage command."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
TOOLS = r'''#!/usr/bin/env python3
import os
from pathlib import Path
import shutil
import sys
args = sys.argv[1:]
profiles = Path(os.environ['PROOF_PROFILES'])
name = Path(sys.argv[0]).name
with Path(os.environ['PROOF_CALLS']).open('a') as calls:
    calls.write(name + ' ' + ' '.join(args) + '\n')
if name == 'rustc':
    print(os.environ['PROOF_LLVM'] + '/lib')
elif name == 'llvm-profdata':
    names = sys.stdin.read().splitlines()
    assert names and all(Path(n).name.startswith('bin-') for n in names), names
    Path(args[args.index('-o')+1]).write_text('\n'.join(names))
elif name == 'llvm-cov':
    assert '--object' in args and args[args.index('--object')+1].endswith('/debug/biomcp'), args
    assert '--ignore-filename-regex' in args and 'generated' in args[args.index('--ignore-filename-regex')+1], args
    assert '--sources' in args and all('/crates/' in p and '/src/' in p for p in args[args.index('--sources')+1:]), args
    merged = Path(args[args.index('--instr-profile')+1]).read_text()
    assert 'harness-' not in merged and 'stale-' not in merged, merged
    value = os.environ.get('PROOF_COVERAGE', '60')
    print('TOTAL 100 0 100% 100 0 100% 100 40', value + '%')
elif args[:2] == ['llvm-cov', 'show-env']:
    print('export LLVM_PROFILE_FILE="' + str(profiles / 'default-%m-%p.profraw') + '"')
    print('export CARGO_LLVM_COV_TARGET_DIR="' + str(profiles / 'target') + '"')
elif args[:2] == ['llvm-cov', 'clean']:
    if profiles.exists(): shutil.rmtree(profiles)
    profiles.mkdir()
    (profiles / 'unrelated.keep').write_text('keep')
elif args[0] == 'test' and '--no-run' not in args:
    if 'ratchets' in args and os.environ.get('PROOF_REJECT_RATCHETS'):
        raise SystemExit(1)
    profile = os.environ['LLVM_PROFILE_FILE'].replace('%m','fake').replace('%p','1')
    Path(profile).write_text('harness execution including library calls')
    if 'cases' in args and not os.environ.get('PROOF_NO_CHILDREN'):
        (profiles / 'bin-fake-2.profraw').write_text('product child')
'''


def invoke(root, env, *args, **extra):
    return subprocess.run(['sh', 'scripts/coverage.sh', *args], cwd=root,
                          env=dict(env, **extra), capture_output=True, text=True)


def main():
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        for name in ('scripts/coverage.sh', 'sdlc/features.md'):
            target = root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, target)
        (root / 'coverage.floor').write_text('60\n')
        source = root / 'crates/product/src/main.rs'
        source.parent.mkdir(parents=True)
        source.write_text('fn main() {}\n')
        skill = root / 'skills/biomcp/SKILL.md'
        skill.parent.mkdir(parents=True)
        skill.write_text('Use biomcp to retrieve evidence.\n')
        subprocess.run(['git', 'init', '-q', str(root)], check=True)
        tools = root / 'fake-tools/bin'
        tools.mkdir(parents=True)
        (tools.parent / 'lib').mkdir()
        for name in ('cargo', 'rustc', 'llvm-profdata', 'llvm-cov'):
            tool = tools / name
            tool.write_text(TOOLS)
            tool.chmod(0o755)
        env = dict(os.environ, PATH=str(tools) + os.pathsep + os.environ['PATH'],
                   PROOF_PROFILES=str(root / 'profiles'), PROOF_CALLS=str(root / 'calls'),
                   PROOF_LLVM=str(tools.parent), RUST_TEST_THREADS='3')
        result = invoke(root, env)
        assert result.returncode == 0, result.stdout + result.stderr
        assert 'cases; product binary source' in result.stdout, result.stdout
        calls = (root / 'calls').read_text().splitlines()
        ratchets = 'cargo test --locked -p biomcp --test ratchets'
        cases = 'cargo test --locked -p biomcp --test cases -- --test-threads 3'
        assert calls.index(ratchets) < calls.index(cases), calls
        assert not any('contracts' in call or '--lib' in call for call in calls), calls
        assert list((root / 'profiles').glob('harness-*.profraw'))
        report = invoke(root, env, 'report')
        assert report.returncode == 0, report.stdout + report.stderr
        unreached = invoke(root, env, 'unreached')
        assert unreached.returncode == 0, unreached.stdout + unreached.stderr
        below = invoke(root, env, 'report', PROOF_COVERAGE='59.7')
        assert below.returncode != 0 and 'below floor' in below.stderr, below
        skill.write_text('Use biomcp to retrieve updated evidence.\n')
        changed = invoke(root, env, 'report')
        assert changed.returncode != 0 and 'inputs changed since collection' in changed.stderr, changed
        rejected = invoke(root, env, PROOF_REJECT_RATCHETS='1')
        assert rejected.returncode != 0 and 'ratchets failed' in rejected.stderr, rejected
        assert not (root / 'profiles/coverage-complete').exists()
        missing = invoke(root, env, PROOF_NO_CHILDREN='1')
        assert missing.returncode != 0 and 'no product binary profiles' in missing.stderr, missing
    print('Coverage selects product children and binary object; refuses incomplete collections and low coverage')


if __name__ == '__main__':
    main()
