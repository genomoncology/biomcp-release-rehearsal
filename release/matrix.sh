#!/bin/sh
# Execute both container architectures at the declared platform floor.
set -eu
revision=$(python3 release/source_revision.py)
scratch=$(mktemp -d)
image_name="biomcp-local-$$"
cleanup() {
  docker image rm "$image_name:amd64" "$image_name:arm64" >/dev/null 2>&1 || true
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
python3 release/toolchain.py check
version=$(cargo metadata --locked --no-deps --format-version 1 | python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"] == "biomcp"))')
mkdir "$scratch/source"
git archive HEAD | tar -x -C "$scratch/source"
python3 release/artifacts.py build --revision "$revision" --output "$scratch/native"
python3 release/gate.py --artifacts "$scratch/native" --scratch "$scratch/native-check"
BIOMCP_CASE_RUNNER="$scratch/native-check/case-runner" \
  BIOMCP_INSTALLED_BINARY="$scratch/native-check/installer-bin/biomcp" \
  BIOMCP_INSTALLED_VERSION="$version" \
  python3 -m unittest release.test_artifacts release.test_installer release.test_smoke
for arch in amd64 arm64; do
  case "$arch" in amd64) wheel_arch=x86_64; archive_arch=x86_64;; arm64) wheel_arch=aarch64; archive_arch=arm64;; esac
  image=$(python3 release/toolchain.py image "manylinux-$wheel_arch")
  docker run --rm --memory 6g --cpus 2 "$@" \
    --user "$(id -u):$(id -g)" -e HOME=/tmp \
    --platform "linux/$arch" -v "$scratch/source:/source:ro" -v "$scratch:/output" \
    -e "BIOMCP_BUILD_REVISION=$revision" -e "BIOMCP_BUILD_ARCH=$arch" \
    "$image" /bin/bash -c '
      set -euo pipefail
      cp -a /source /tmp/build
      cd /tmp/build
      export HOME=/tmp/build-home CARGO_HOME=/tmp/cargo RUSTUP_HOME=/tmp/rustup CARGO_BUILD_JOBS=2
      mkdir -p "$HOME"
      python_series=$(python3 release/toolchain.py get python | tr -d .)
      export PATH=/opt/python/cp$python_series-cp$python_series/bin:$PATH
      python release/toolchain.py python-check
      python release/toolchain.py install-rust
      export PATH=/tmp/cargo/bin:$PATH
      python release/artifacts.py build --manylinux --revision "$BIOMCP_BUILD_REVISION" --output "/output/linux-$BIOMCP_BUILD_ARCH"
      python release/gate.py --prepare-only --artifacts "/output/linux-$BIOMCP_BUILD_ARCH" --scratch /tmp/proof
      cp -a /tmp/proof "/output/proof-$BIOMCP_BUILD_ARCH"
    '
  # The installation container has only artifacts, a runner and public fixtures.
  # It has no checkout, build tree, host configuration or credential mount.
  docker run --rm --memory 6g --cpus 2 "$@" \
    --user "$(id -u):$(id -g)" -e HOME=/tmp \
    --platform "linux/$arch" -v "$scratch/linux-$arch:/artifacts:ro" \
    -v "$scratch/proof-$arch:/proof:ro" "$image" /bin/bash -c '
      set -euo pipefail
      cp -a /proof /tmp/check
      python_series=$(python3 /tmp/check/tools/release/toolchain.py get python | tr -d .)
      python=/opt/python/cp$python_series-cp$python_series/bin/python
      "$python" /tmp/check/tools/release/gate.py --prepared \
        --python "$python" --artifacts /artifacts --scratch /tmp/check
    '
  mkdir "$scratch/container-$arch"
  tar -xzf "$scratch/linux-$arch/biomcp-linux-$archive_arch.tar.gz" -C "$scratch/container-$arch"
  cp "$scratch/proof-$arch/case-runner" "$scratch/container-$arch/case-runner"
  cp "$scratch/proof-$arch/tool-runner" "$scratch/container-$arch/tool-runner"
  cp -R "$scratch/proof-$arch/fixtures" "$scratch/container-$arch/fixtures"
  cp release/Dockerfile "$scratch/container-$arch/Dockerfile"
  docker build --build-arg "runtime_image=$(python3 release/toolchain.py image runtime)" \
    --build-arg "runtime_packages=$(python3 release/toolchain.py get runtime-packages)" --platform "linux/$arch" -t "$image_name:$arch" "$scratch/container-$arch"
  python3 release/container_check.py "$image_name:$arch" "$arch" "$scratch/container-$arch" "$version" "$@"
  docker image rm "$image_name:$arch"
done

python3 - <<'PYTHON'
import json
import platform
for native in ["macOS arm64", "macOS x86_64", "Windows x86_64"]:
    if native == "macOS " + platform.machine() and platform.system() == "Darwin":
        continue
    print(json.dumps({"platform": native, "result": "deferred", "reason": "Native platform execution requires its release rehearsal machine"}))
PYTHON
