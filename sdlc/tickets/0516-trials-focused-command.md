# 0516 — Run focused proofs through one committed trials command

Status: COMPLETE.
Landed: c363db9d062d597a3eb62bf0ded8839e9f98b022

Milestone: 5

## Outcome

`scripts/trials.py` gains one focused command that every focused proof job uses instead of a hand-written wrapper. It builds through `release/artifacts.py` cargo paths, runs from the current checkout, accepts literal trial names discovered from `--list`, passes cases to the Rust runner to interpret, selects with the existing `run_selected`, and exits with the runner's own exit status.

- Why: an RCA of five failed setups (m17714): j24103 used an unsupported archive extraction, j24114 guessed a private helper, j24147 lost compiler stderr, j24148 ran from the wrong directory and j24163 parsed YAML as JSON. Each came from a one-off wrapper that owned build identity, executable discovery, working directory or case parsing.
- Starts from: `scripts/trials.py` `run_selected`, `release/artifacts.py` and 0484's `--list` selection.
- Rules: no new input format, manifest or proof framework; the command refuses an empty selection before any build, builds only the Rust test runner to validate names through `--list`, and refuses an unknown name before any product preparation, as `run_selected` does (m17716); assignments name this command for focused proofs from now on.
- Proof: one existing YAML case, one custom trial, one deliberately failing trial whose exit passes through unchanged, and an ordinary structural run, each through the new command; the existing selection tests still pass.
- Changes: `scripts/trials.py` and its tests.
- Defers: nothing.
