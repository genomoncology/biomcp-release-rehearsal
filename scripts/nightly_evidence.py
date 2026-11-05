"""Transfer the nightly command's evidence with verified durable delivery."""
import hashlib
import json
import shlex
import subprocess
import tarfile

COMMAND = ['cargo', 'test', '--locked', '-p', 'biomcp', '--test', 'tools',
           '--', '--live-check', '--details']
FILES = ('report.json', 'build.log', 'exit-status', 'run.json')

# The receiver creates a unique destination, accepts only our named plain files,
# and verifies the sender's manifest before acknowledging durable delivery.
RECEIVE = r'''
import hashlib, json, os, pathlib, sys, tarfile
root = pathlib.Path(sys.argv[1])
root.mkdir(parents=True, exist_ok=False)
allowed = {'report.json', 'build.log', 'exit-status', 'run.json', 'manifest.json'}
seen = set()
with tarfile.open(fileobj=sys.stdin.buffer, mode='r|') as archive:
    for member in archive:
        if member.name not in allowed or member.name in seen or not member.isfile():
            raise ValueError('unexpected evidence entry')
        seen.add(member.name)
        source = archive.extractfile(member)
        with (root / member.name).open('xb') as target:
            while True:
                chunk = source.read(1024 * 1024)
                if not chunk:
                    break
                target.write(chunk)
            target.flush()
            os.fsync(target.fileno())
if seen != allowed:
    raise ValueError('incomplete evidence')
manifest_bytes = (root / 'manifest.json').read_bytes()
if hashlib.sha256(manifest_bytes).hexdigest() != sys.argv[2]:
    raise ValueError('manifest transfer mismatch')
manifest = json.loads(manifest_bytes)
if set(manifest) != allowed - {'manifest.json'}:
    raise ValueError('unexpected manifest entries')
for name, expected in manifest.items():
    digest = hashlib.sha256()
    with (root / name).open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            digest.update(chunk)
    if digest.hexdigest() != expected:
        raise ValueError('evidence checksum mismatch: ' + name)
fd = os.open(str(root), os.O_RDONLY)
try:
    os.fsync(fd)
finally:
    os.close(fd)
'''


def digest(path):
    checksum = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            checksum.update(chunk)
    return checksum.hexdigest()


def deliver(scratch, host, evidence_dir):
    manifest = {name: digest(scratch / name) for name in FILES}
    (scratch / 'manifest.json').write_text(json.dumps(manifest, sort_keys=True) + '\n')
    archive = scratch / 'transfer.tar'
    with tarfile.open(archive, 'w') as target:
        for name in (*FILES, 'manifest.json'):
            target.add(scratch / name, arcname=name, recursive=False)
    destination = evidence_dir / scratch.name
    command = shlex.join(['python3', '-c', RECEIVE, str(destination),
                          digest(scratch / 'manifest.json')])
    with archive.open('rb') as source:
        subprocess.run(['ssh', '-o', 'BatchMode=yes', '-o', 'ConnectTimeout=30',
                        host, command], stdin=source, check=True)
    print('Verified evidence: ' + host + ':' + str(destination))
