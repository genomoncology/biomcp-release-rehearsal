#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=1.95.0
export PATH="$HOME/.local/bin:$PATH"
python3.13 --version
case "${1:-linux}" in
  linux)
    image=rust:1.95.0-slim
    scratch=$(mktemp -d)
    trap 'chmod -R u+w "$scratch"; rm -rf "$scratch"' EXIT HUP INT TERM
    docker run --rm ${JOB_LABEL_FILE:+--label-file "$JOB_LABEL_FILE"} \
      --user "$(id -u):$(id -g)" -e CARGO_MANIFEST_DIR=/source/crates/biomcp-test-support \
      -v "$PWD:/input:ro" -v "$scratch:/output" "$image" \
      rustc --edition 2024 /input/release/evidence_probe.rs -o /output/evidence-probe
    mkdir "$scratch/fixtures"
    chmod 555 "$scratch/fixtures"
    docker run --rm ${JOB_LABEL_FILE:+--label-file "$JOB_LABEL_FILE"} \
      --user "$(id -u):$(id -g)" \
      -v "$scratch:/proof:ro" "$image" /bin/sh -eu -c '
        cp -a /proof /tmp/check
        test ! -e /source
        test ! -w /tmp/check/fixtures
        mkdir /tmp/fixture-scratch
        export BIOMCP_TEST_FIXTURES=/tmp/check/fixtures TMPDIR=/tmp/fixture-scratch
        /tmp/check/evidence-probe /tmp/check/target/fixture-evidence
      '
    cp "$scratch/evidence-probe" "$scratch/case-runner"
    cp "$scratch/evidence-probe" "$scratch/tool-runner"
    chmod 555 "$scratch"
    setup=$(python3.13 -c 'import sys; sys.path.insert(0, "release"); from container_check import RUNNER_SETUP; print(RUNNER_SETUP)')
    docker run --rm ${JOB_LABEL_FILE:+--label-file "$JOB_LABEL_FILE"} \
      --user "$(id -u):$(id -g)" -v "$scratch:/proof:ro" \
      -e BIOMCP_TEST_FIXTURES=/tmp/check/fixtures -e TMPDIR=/tmp/fixture-scratch \
      "$image" /bin/sh -ec "mkdir /tmp/fixture-scratch; $setup" installed-runner \
      /tmp/check/tool-runner /tmp/check/target/fixture-evidence
    ;;
  focused)
    cargo fmt --all -- --check
    python3.13 scripts/check_shell.py
    python3.13 -m unittest release.test_linux
    cargo test --locked -p biomcp --test tools -- --proofs --exact schema-discovery/retains-failed-local-peer-state
    cargo test --locked -p biomcp --test tools -- --proofs --exact schema-discovery/persistent-fixture-evidence
    cargo test --locked -p biomcp --test tools -- --proofs --exact schema-discovery/persistent-native-evidence
    cargo clippy --locked -p biomcp --test tools -- -D warnings
    ;;
  boundary)
    ruff check --select F401,F841,ERA,BLE,E722,S110 release/container_check.py release/test_linux.py
    python3.13 scripts/check_shell.py
    python3.13 -m unittest release.test_linux
    ;;
  *) exit 2 ;;
esac
