# 0430 — Count only product-binary coverage toward the floor

Status: COMPLETE.
Landed: 1dd4c6aaea42a47f961596a01673d7102678bf11

Milestone: 5

Depends on: 0429

## Outcome

The coverage gate counts only lines the `biomcp` binary runs while the `cases` target runs. Harness code, test modules and in-process library calls count toward neither side. The PM sets `coverage.floor` from the first measurement.

- Why: `scripts/coverage.sh` counts harness profiles and contract targets, and 51 runner files call library crates in-process, so the floor does not measure outside-in proof.
- Starts from: `scripts/coverage.sh`, `scripts/contract_targets.py`, `coverage.floor`, `crates/biomcp/tests/runner/launch.rs`, the `Contracts:` lines in `sdlc/features.md`, architecture 10.3.
- Rules: the launcher sets `LLVM_PROFILE_FILE=<dir>/bin-%m-%p.profraw` on every child. `coverage.sh` merges only `bin-` profiles with `llvm-profdata merge` and reports with `llvm-cov report --object` on the binary, because `cargo llvm-cov report` cannot select one object. Library functions the binary never links fall outside the denominator. Generated protobuf code is excluded by path. biodata's contract tests do not count (ruling 3). Only the PM raises the floor, and architecture 10.3 changes to say so (ruling 4). `scripts/contract_targets.py` and the `Contracts:` lines go. PM ruling (m14375): the first binary-only measurement at 7b5fdaf77 was 91.81535 percent, so `coverage.floor` is 91.81, truncated rather than rounded so unchanged source cannot fail.
- Proof: a planted trial that calls a query function in the harness process adds no counted line; deleting the only case that reaches one branch lowers the measured figure; the gate fails 0.3 points below the floor.
- Changes: the files above, `Makefile`, `docs/development.md`.
- Defers: converting in-process trials (0431).
