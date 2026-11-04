#!/bin/sh
# Build once at the wheel floor and qualify the same native and container bytes.
set -eu
test -n "$BUILD_IMAGE"
revision=$(python3 release/source_revision.py)
cold=${BIOMCP_RELEASE_COLD:-0}
report=${BIOMCP_RELEASE_REPORT:-$PWD/dist/cold-report.json}
stage() {
  name=$1
  shift
  if "$@"; then code=0; else code=$?; fi
  if [ "$cold" = 1 ]; then
    python3 release/rehearse.py record --report "$report" "$name" "$code"
  elif [ "$code" != 0 ]; then
    exit "$code"
  fi
}
mkdir -p dist
if [ "$cold" = 1 ]; then
  mkdir -p "$(dirname "$report")"
  rm -f "$report"
  set -- "$@" -e BIOMCP_RELEASE_COLD=1
else
  mkdir -p .release-cache/cargo
fi
build_cache=${BIOMCP_RELEASE_COLD:-0}
build() {
  if [ "$build_cache" != 1 ]; then
    operation=$1
    shift
    set -- "$operation" -v "$PWD/.release-cache/cargo:/cargo" "$@"
  fi
  docker "$@"
}
stage build build run --rm "$@" --user "$(id -u):$(id -g)" -e HOME=/tmp \
  -v "$PWD:/source" -w /source \
  -e CARGO_HOME=/cargo \
  -e CARGO_BUILD_JOBS=2 -e "BIOMCP_BUILD_REVISION=$revision" \
  "$BUILD_IMAGE" /bin/bash -c '
    set -euo pipefail
    python_series=$(python3 release/toolchain.py get python | tr -d .)
    export PATH=/cargo/bin:/opt/python/cp$python_series-cp$python_series/bin:$PATH
    if [ "${BIOMCP_RELEASE_COLD:-0}" = 1 ]; then
      export HOME=/tmp/release-home RUSTUP_HOME=/tmp/release-rustup CARGO_HOME=/tmp/release-cargo
      export PATH="$CARGO_HOME/bin:$PATH"
      mkdir "$HOME"
      python release/rehearse.py inside --report /source/dist/cold-build-report.json
    else
      python release/toolchain.py python-check
      python release/toolchain.py install-rust
      python release/artifacts.py build --manylinux --revision "$BIOMCP_BUILD_REVISION" --output dist
      python release/gate.py --prepare-only --artifacts dist --scratch /source/proof
    fi
  '
# The install process sees the artifacts and copied proof, without the checkout.
stage install docker run --rm "$@" --user "$(id -u):$(id -g)" -e HOME=/tmp \
  -v "$PWD/dist:/artifacts:ro" -v "$PWD/proof:/proof:ro" \
  -v "$PWD/dist:/output" \
  "$BUILD_IMAGE" /bin/bash -c '
    set -euo pipefail
    cp -a /proof /tmp/check
    python_series=$(python3 /tmp/check/tools/release/toolchain.py get python | tr -d .)
    export PATH=/opt/python/cp$python_series-cp$python_series/bin:$PATH
    python /tmp/check/tools/release/gate.py --prepared --python python \
      --artifacts /artifacts --scratch /tmp/check --report /output/install-report.json
  ' > install-report.log
cat install-report.log
archive=$(find dist -name 'biomcp-linux-*.tar.gz')
case "$archive" in
  *x86_64*) arch=amd64;;
  *arm64*) arch=arm64;;
  *) echo 'unsupported or missing Linux archive' >&2
     if [ "$cold" != 1 ]; then exit 1; fi
     case "$BUILD_IMAGE" in *aarch64*) arch=arm64;; *) arch=amd64;; esac;;
esac
mkdir -p container
stage container-archive tar -xzf "$archive" -C container
stage container-runner cp proof/case-runner container/case-runner
stage container-tool-runner cp proof/tool-runner container/tool-runner
stage container-fixtures cp -R proof/fixtures container/fixtures
stage container-recipe cp release/Dockerfile container/Dockerfile
image="biomcp-candidate:$arch"
stage container-build docker build --build-arg "runtime_image=$(python3 release/toolchain.py image runtime)" \
  --build-arg "runtime_packages=$(python3 release/toolchain.py get runtime-packages)" --platform "linux/$arch" -t "$image" container
version=$(python3 -c 'import json; print(json.load(open("dist/release.json"))["version"])') || version=unknown
stage container-check python3 release/container_check.py "$image" "$arch" container "$version" "$@"
stage container-save docker save "$image" -o "dist/biomcp-container-$arch.tar"
stage container-cleanup docker image rm "$image"

if [ "$cold" = 1 ]; then
  python3 release/rehearse.py finish --report "$report"
fi
