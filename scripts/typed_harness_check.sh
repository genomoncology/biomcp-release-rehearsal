#!/bin/sh
# Synthetic recorder preflight only. No compiler, product, client or provider.
set -eu
export PYTHONDONTWRITEBYTECODE=1
exec python3 scripts/typed_harness_preflight.py check
