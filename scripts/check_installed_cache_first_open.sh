#!/bin/sh
# Install the qualified frozen AMD wheel offline and witness native first opens.
set -eu
cd "$(dirname "$0")/.."
source_base=/home/ian/workspace/data/biomcp-redux-evidence/0293/candidate-015aea0a704b7/producer-custody-j30268-0639
root="$PWD/target/installed-cache-first-open"
wheel=biomcp_cli-1.0.0-py3-none-manylinux_2_28_x86_64.whl
mkdir -p "$root/artifact"
scp "yellow:$source_base/artifacts/$wheel" "$root/artifact/$wheel"
scp "yellow:$source_base/artifacts/release.json" "$root/artifact/release.json"
scp "yellow:$source_base/proof/bootstrap.json" "$root/artifact/bootstrap.json"
scp "yellow:$source_base/custody-receipt.json" "$root/artifact/custody-receipt.json"
scp "yellow:$source_base/wheel-pass-projection-snapshot.json" "$root/artifact/wheel-pass-projection-snapshot.json"
python3.13 - "$root/artifact" "$wheel" <<'PY'
import hashlib
import json
from pathlib import Path
import platform
import sys
import zipfile
root = Path(sys.argv[1])
assert platform.system() == 'Linux' and platform.machine() == 'x86_64'
release = json.loads((root / 'release.json').read_text())
bootstrap = json.loads((root / 'bootstrap.json').read_text())
source = '015aea0a704b7b6176b6988bc6fae707ac0feb38'
binary_sha = 'ee51cf73879b228e2b4814056e682ac4d870fa9222628c0b6cfc7f974f740ca9'
assert release['revision'] == source
assert bootstrap['product_revision'] == bootstrap['tooling_revision'] == source
assert release['binary_sha256'] == binary_sha
wheel = root / sys.argv[2]
assert hashlib.sha256(wheel.read_bytes()).hexdigest() == 'f7511ad9a4f819dbf248ebd1228e4a73ab2807ebfae075d47b95dfb25d7d7797'
with zipfile.ZipFile(wheel) as archive:
    manifests = [name for name in archive.namelist() if name.endswith('.dist-info/release.json')]
    assert len(manifests) == 1 and json.loads(archive.read(manifests[0])) == release
    binaries = [name for name in archive.namelist() if '.data/scripts/' in name]
    assert len(binaries) == 2
    assert all(hashlib.sha256(archive.read(name)).hexdigest() == binary_sha for name in binaries)
(root / 'verified-binding.json').write_text(json.dumps({
    'candidate': source, 'binary_sha256': binary_sha,
    'wheel_sha256': hashlib.sha256(wheel.read_bytes()).hexdigest(),
    'platform': platform.platform(),
}, indent=2) + '\n')
PY
python3.13 -m venv "$root/installed"
"$root/installed/bin/python" -m pip install --no-index --no-deps "$root/artifact/$wheel"
python3.13 scripts/cache_installed_first_open.py "$root/installed/bin/biomcp" \
    ee51cf73879b228e2b4814056e682ac4d870fa9222628c0b6cfc7f974f740ca9 \
    "$root/results" --artifact "yellow:$source_base/artifacts/$wheel"
