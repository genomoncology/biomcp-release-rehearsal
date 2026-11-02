#!/bin/sh
# Inventory source changes and check-only disk use in the pinned Linux image.
set -eu
cd "$(dirname "$0")/.."
case "${1:-container}" in
  container|focused)
    mode=checks
    if [ "${1:-container}" = focused ]; then
      export PATH="$HOME/.local/bin:$PATH"
      ruff check --no-cache --select F401,F841,ERA,BLE,E722,S110 \
        release/source_revision.py release/test_source_revision.py release/test_workflow_contracts.py
      actionlint
      python3.13 scripts/check_shell.py
      mode=focused-checks
    fi
    image=$(python3.13 release/toolchain.py image manylinux-aarch64)
    mkdir -p target/linux-source-check/build
    checkout=$(mktemp -d "$PWD/target/linux-source-check/source.XXXXXX")
    trap 'python3.13 -c "import shutil,sys; shutil.rmtree(sys.argv[1])" "$checkout"' EXIT
    git clone --quiet --depth 1 --no-local "$PWD" "$checkout"
    docker run --rm ${JOB_LABEL_FILE:+--label-file "$JOB_LABEL_FILE"} \
      --platform linux/arm64 --memory 8g --cpus 2 \
      -v "$checkout:/source" -v "$PWD/target/linux-source-check/build:/source/target" -w /source \
      -e HOME=/source/target/home \
      -e CARGO_BUILD_JOBS=2 -e RUST_TEST_THREADS=2 \
      "$image" /bin/bash scripts/check_linux_release_source.sh "$mode"
    ;;
  checks|focused-checks)
    export PATH="$HOME/.cargo/bin:/opt/python/cp313-cp313/bin:$PATH"
    mkdir -p "$HOME"
    # These fixtures retain only the OS command path, as on the release runner.
    if [ ! -x /usr/bin/git ]; then ln -s "$(command -v git)" /usr/bin/git; fi
    git config --global --add safe.directory /source
    python3.13 release/toolchain.py install-rust
    cargo --version
    python3.13 --version
    uname -sm
    if [ "$1" = checks ]; then
      cargo install cargo-deny --version "$(python3.13 release/toolchain.py get cargo-deny)" --locked
    else
      cargo deny --version
    fi
    snapshot() {
      echo "SOURCE STATUS $1"
      git status --porcelain=v1 --untracked-files=all
      echo "DISK $1"
      df -k . /tmp
      du -sk target
    }
    snapshot before
    test -z "$(git status --porcelain=v1 --untracked-files=all)"
    if [ "$1" = focused-checks ]; then
      python3.13 -m unittest release.test_source_revision release.test_workflow_contracts
      make docs-check
      snapshot docs-check
      test -z "$(git status --porcelain=v1 --untracked-files=all)"
      python3.13 release/source_revision.py
      du -sk target/debug
      cargo clean --profile dev
      snapshot debug-cleanup
      test ! -e target/debug
      test -z "$(git status --porcelain=v1 --untracked-files=all)"
      exit 0
    fi
    status=0
    for stage in fmt lint test docs-check; do
      if make "$stage"; then code=0; else code=$?; fi
      echo "CHECK RESULT $stage $code"
      snapshot "$stage"
      if [ "$code" != 0 ]; then status=$code; fi
    done
    echo 'CHECK BUILD OUTPUT'
    du -sk target/* target/debug/* 2>/dev/null || true
    exit "$status"
    ;;
  *) exit 2 ;;
esac
