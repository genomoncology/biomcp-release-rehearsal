"""Run a fixed bounded source plan; retain replies and verify its tar archive.

make source-capture PLAN=/absolute/plan.json OUT=/absolute/new-directory
Plan: deadline_seconds, max_bytes, spacing_seconds, requests. Each request has
id, source, url, response_format (json or bytes), optional method/body/headers,
and optional key {env, header, prefix}. URLs contain the complete selectors;
this command adds none. Keys enter curl through stdin, never arguments/evidence.
A stopped plan can continue with --continue-from DIR into a new --out directory.
Every attempted request is retained and never automatically retried. Verify an
archive with --verify FILE. --cleanup removes scratch only after archive proof.
Default stdout is one summary line; bodies and headers stay in the output folder.
Response headers have a separate 64 KiB bound; oversized replies stop the plan.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
from urllib.parse import urlsplit

from capture_outcome import classify

HEADER_BYTES = 64 * 1024


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_digest(path):
    result = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            result.update(chunk)
    return result.hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, sort_keys=True, indent=2) + '\n')


def require_environment():
    names = ['PATH', 'HOME', 'TMPDIR']
    if os.name == 'nt':
        names.append('SystemRoot')
    for name in names:
        if not os.environ.get(name):
            raise ValueError('missing required OS variable: ' + name)
    if not Path(os.environ['TMPDIR']).is_dir():
        raise ValueError('TMPDIR must name an existing directory')
    for tool in ('curl', 'tar'):
        if not shutil.which(tool):
            raise ValueError('missing installed tool: ' + tool)


def outside_git(path):
    if not path.is_absolute():
        raise ValueError('output must be absolute')
    for parent in (path.resolve(), *path.resolve().parents):
        if (parent / '.git').exists():
            raise ValueError('raw capture output must stay outside Git')


def load_plan(path):
    plan = json.loads(path.read_text())
    for name, maximum in [('deadline_seconds', 300), ('max_bytes', 64 * 1024 * 1024),
                          ('spacing_seconds', 3600)]:
        value = plan[name]
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            raise ValueError('plan bound must be numeric: ' + name)
        if not 0 <= value <= maximum or (name != 'spacing_seconds' and value == 0):
            raise ValueError('invalid plan bound: ' + name)
    if not isinstance(plan['max_bytes'], int):
        raise ValueError('max_bytes must be an integer')
    seen, secrets = set(), []
    for request in plan['requests']:
        if not re.fullmatch(r'[A-Za-z0-9_.-]+', request['id']) or request['id'] in seen:
            raise ValueError('request IDs must be unique safe names')
        seen.add(request['id'])
        url = urlsplit(request['url'])
        if url.scheme not in ('http', 'https') or not url.hostname or url.username or url.password:
            raise ValueError('request URL must be HTTP(S) without credentials')
        if request['response_format'] not in ('json', 'bytes') or not request['source']:
            raise ValueError('request needs source and explicit response_format')
        if request.get('method', 'GET') not in ('GET', 'POST', 'HEAD'):
            raise ValueError('unsupported request method')
        if 'key' in request:
            key = request['key']
            if not re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', key['env']):
                raise ValueError('invalid key environment name')
            secret = os.environ.get(key['env'])
            if not secret:
                raise ValueError('missing named key variable: ' + key['env'])
            secrets.append(secret)
        for name, value in request.get('headers', {}).items():
            validate_header(name, value)
        if 'key' in request:
            validate_header(request['key']['header'], request['key'].get('prefix', ''))
    serialized = json.dumps(plan)
    if any(secret in serialized for secret in secrets):
        raise ValueError('plan contains a key value')
    return plan, secrets


def validate_header(name, value):
    if not re.fullmatch(r'[A-Za-z0-9-]+', name) or '\n' in value or '\r' in value:
        raise ValueError('invalid request header')


def files_in(directory):
    files = {}
    for path in directory.iterdir():
        if path.is_symlink() or not path.is_file():
            raise ValueError('evidence must contain only regular files')
        if path.name != 'manifest.json':
            files[path.name] = file_digest(path)
    return files


def manifest(directory):
    write_json(directory / 'manifest.json', files_in(directory))


def verify_directory(directory):
    expected = json.loads((directory / 'manifest.json').read_text())
    if expected != files_in(directory):
        raise ValueError('evidence manifest mismatch')


def verify_archive(archive):
    with tarfile.open(archive) as packed:
        members = packed.getmembers()
        names = [member.name for member in members]
        for member in members:
            if not member.isfile() or '/' in member.name or member.name in ('.', '..'):
                raise ValueError('archive contains an unsafe member')
        if len(names) != len(set(names)) or 'manifest.json' not in names:
            raise ValueError('archive contains duplicate members or lacks its manifest')
        with packed.extractfile('manifest.json') as stream:
            expected = json.load(stream)
        if not isinstance(expected, dict) or set(names) != set(expected) | {'manifest.json'}:
            raise ValueError('archive members differ from evidence manifest')
    with tempfile.TemporaryDirectory(prefix='source-capture-verify-') as scratch:
        subprocess.run(['tar', '-xzf', str(archive), '-C', scratch], check=True,
                       stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        verify_directory(Path(scratch))


def initialize(out, plan, previous):
    plan_hash = digest(json.dumps(plan, sort_keys=True).encode())
    receipts = []
    if previous:
        verify_directory(previous)
        state = json.loads((previous / 'state.json').read_text())
        if state['plan_sha256'] != plan_hash:
            raise ValueError('continuation plan differs from retained plan')
        receipts = state['receipts']
        if [row['index'] for row in receipts] != list(range(len(receipts))):
            raise ValueError('continuation receipts are not a delivered prefix')
    out.mkdir(parents=True)
    if previous:
        for path in previous.iterdir():
            shutil.copyfile(path, out / path.name)
    state = {'plan_sha256': plan_hash, 'receipts': receipts}
    write_json(out / 'plan.json', plan)
    write_json(out / 'state.json', state)
    manifest(out)
    return state


def bounded_curl(command, config, plan):
    limit = plan['max_bytes'] + HEADER_BYTES
    with subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                          stderr=subprocess.DEVNULL) as process:
        try:
            process.stdin.write(('\n'.join(config) + '\n').encode())
            process.stdin.close()
            raw = process.stdout.read(limit + 1)
            if len(raw) > limit:
                process.kill()
                return 63, raw[:limit]
            return process.wait(timeout=plan['deadline_seconds'] + 2), raw
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()


def curl_reply(request, plan, witness):
    headers = dict(request.get('headers', {}))
    if 'key' in request:
        key = request['key']
        headers[key['header']] = key.get('prefix', '') + os.environ[key['env']]
        validate_header(key['header'], headers[key['header']])
    config = ['url = ' + json.dumps(request['url'], ensure_ascii=False),
              'request = ' + json.dumps(request.get('method', 'GET'))]
    config.extend('header = ' + json.dumps(name + ': ' + value, ensure_ascii=False)
                  for name, value in headers.items())
    if 'body' in request:
        config.append('data-raw = ' + json.dumps(request['body'], ensure_ascii=False))
    command = ['curl', '--disable', '--config', '-', '--silent', '--show-error',
               '--include', '--globoff', '--max-time', str(plan['deadline_seconds']),
               '--max-filesize', str(plan['max_bytes']), '--retry', '0',
               '--proto', '=http,https']
    if request.get('method') == 'HEAD':
        command.append('--head')
    key_name = request.get('key', {}).get('header')
    witness['header_name'] = key_name
    witness['attached'] = bool(key_name) and any(
        line.startswith('header = "' + key_name + ': ') for line in config)
    if key_name and not witness['attached']:
        raise ValueError('named credential header missing at curl boundary')
    returncode, raw = bounded_curl(command, config, plan)
    status, headers, body = 0, b'', raw
    header_bytes = 0
    while body.startswith(b'HTTP/'):
        headers, separator, body = body.partition(b'\r\n\r\n')
        header_bytes += len(headers) + len(separator)
        if not separator or header_bytes > HEADER_BYTES:
            return 0, b'', b'', True, False
        status = int(headers.split(b' ', 2)[1])
        if status >= 200:
            break
    truncated = len(body) > plan['max_bytes'] or returncode == 63
    body = body[:plan['max_bytes']]
    transport = returncode not in (0, 63)
    return status, headers, body, truncated, transport


def reflects_key(headers, body, secrets):
    if any(secret.encode() in headers + body for secret in secrets):
        return True
    # Scan JSON string tokens independently, retaining duplicate/deep fields.
    for match in re.finditer(rb'"(?:[^"\\]|\\.)*"', body):
        try:
            value = json.loads(match.group())
        except (ValueError, UnicodeError):
            continue
        if any(secret in value for secret in secrets):
            return True
    return False



def sanitize_headers(headers):
    sensitive_names = {b'authorization', b'proxy-authorization', b'x-api-key', b'set-cookie'}
    kept, sensitive = [], False
    for line in headers.splitlines():
        if not line.startswith((b' ', b'\t')):
            sensitive = line.split(b':', 1)[0].lower() in sensitive_names
        if not sensitive:
            kept.append(line)
    return b'\r\n'.join(kept)

def capture_one(index, request, plan, secrets, out):
    started = time.monotonic()
    witness = {}
    try:
        status, headers, body, truncated, transport = curl_reply(request, plan, witness)
    except subprocess.TimeoutExpired:
        status, headers, body, truncated, transport = 0, b'', b'', False, True
    if reflects_key(headers, body, secrets):
        status, headers, body, truncated, transport = 0, b'', b'', False, True
    if request['response_format'] == 'json':
        outcome = classify(request['source'], request['url'], status, body,
                           truncated=truncated, transport_failure=transport).kind.value
    else:
        outcome = ('transport_failure' if transport else 'truncated' if truncated
                   else 'success' if 200 <= status < 300 else 'provider_failure')
    receipt = {'index': index, 'id': request['id'], 'source': request['source'],
               'url': request['url'], 'status': status, 'outcome': outcome,
               'elapsed_seconds': round(time.monotonic() - started, 3),
               'body_bytes': len(body), 'body_sha256': digest(body),
               'credential_header': witness.get('header_name'),
               'credential_header_attached': witness.get('attached', False)}
    headers = sanitize_headers(headers)
    prefix = f'{index:04d}'
    (out / (prefix + '.body')).write_bytes(body)
    (out / (prefix + '.headers')).write_bytes(headers)
    write_json(out / (prefix + '.receipt.json'), receipt)
    return receipt


def run_capture(args):
    outside_git(args.out)
    output, temporary = args.out.resolve(), Path(os.environ['TMPDIR']).resolve()
    if output == temporary or temporary in output.parents:
        raise ValueError('choose a durable location outside TMPDIR for capture output')
    archive = Path(str(args.out) + '.tar.gz')
    if args.out.exists() or archive.exists():
        raise ValueError('output directory and archive must not exist')
    plan, secrets = load_plan(args.plan)
    state = initialize(args.out, plan, args.continue_from)
    stopped = False
    for index in range(len(state['receipts']), len(plan['requests'])):
        if state['receipts']:
            time.sleep(plan['spacing_seconds'])
        row = capture_one(index, plan['requests'][index], plan, secrets, args.out)
        state['receipts'].append(row)
        write_json(args.out / 'state.json', state)
        manifest(args.out)
        if row['outcome'] not in ('success', 'empty'):
            stopped = True
            break
    members = sorted(path.name for path in args.out.iterdir())
    subprocess.run(['tar', '-czf', str(archive), '-C', str(args.out), *members],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE,
                   env=dict(os.environ, COPYFILE_DISABLE='1'))
    verify_archive(archive)
    if args.cleanup:
        shutil.rmtree(args.out)
    print(json.dumps({'attempted': len(state['receipts']), 'stopped': stopped,
                      'archive': str(archive), 'sha256': file_digest(archive)}))
    return 2 if stopped else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--plan', type=Path)
    parser.add_argument('--out', type=Path)
    parser.add_argument('--continue-from', type=Path)
    parser.add_argument('--cleanup', action='store_true')
    parser.add_argument('--verify', type=Path)
    args = parser.parse_args()
    try:
        require_environment()
        if args.verify:
            verify_archive(args.verify.resolve())
            print('Archive manifest verified')
            return 0
        if not args.plan or not args.out:
            raise ValueError('--plan and --out are required')
        return run_capture(args)
    except (ValueError, KeyError, OSError, subprocess.SubprocessError, tarfile.TarError) as error:
        detail = str(error) if type(error) is ValueError else type(error).__name__
        print('source capture refused: ' + detail, file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
