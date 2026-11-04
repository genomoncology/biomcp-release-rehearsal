#!/bin/sh
set -eu
export PATH="/Users/ian/.local/share/uv/python/cpython-3.13.17-macos-aarch64-none/bin:$HOME/.cargo/bin:$PATH"
cargo +1.95.0 fmt --all -- --check
cargo +1.95.0 build --locked -p biomcp --bin biomcp
if [ "${1:-}" = gate ]; then
    cargo +1.95.0 test --locked -p biomcp --test cases gene/help-clingen
    cargo +1.95.0 test --locked -p biomcp --test cases trial/help-country
fi
if [ "${1:-}" = refresh ]; then
    python3 scripts/refresh_help_cases.py "${CARGO_TARGET_DIR:-target}/debug/biomcp"
    make docs-generate
    cargo +1.95.0 test --locked -p biomcp --test cases diagnostic/help
    cargo +1.95.0 test --locked -p biomcp --test cases pathway/help
    cargo +1.95.0 test --locked -p biomcp --test cases protein/help
    cargo +1.95.0 test --locked -p biomcp --test cases source-pacing/settings-and-ceilings
fi
cargo +1.95.0 test --locked -p biomcp --test ratchets front_map
python3 scripts/check_front_map.py "${CARGO_TARGET_DIR:-target}/debug/biomcp" --baseline /Users/ian/workspace/t/biomcp-redux-0586-benchmark-baseline-27ead/installed/bin/biomcp --revision "$(git rev-parse HEAD)" --baseline-revision 27eadceca0b0d883f67352745d7f3e6144d9f37d --baseline-sha256 affa8a6637f00702a2d52471a157249e10719ca7e7f28beebcb760ea9431ec2e --output "$HOME/workspace/t/biomcp-redux-0586-proof-$(git rev-parse HEAD)"
if [ "${1:-}" = gate ]; then
    make landing-gate
fi
