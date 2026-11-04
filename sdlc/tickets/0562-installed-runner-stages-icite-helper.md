# Installed runner stages the iCite proof helper

Status: COMPLETE.
Landed: 7b8f2d0417087bb91ebbac51315a03f4ccfa6fe5
Milestone: 5

## Outcome

The installed tool runner finds `icite_discovery_proof.py` on every platform, so `live-source-checks/retained-drift-witnesses` passes against installed artifacts the same way it passes from a checkout.

- Why: private rehearsal run 41358845085 at ab41e914f failed on Apple ARM and Linux ARM because the proof opens `<scratch>/scripts/icite_discovery_proof.py`, and `release/gate.py` stages only `tests/` as `fixtures/`, the runners, `release/` and `LICENSE`. `crates/biomcp/tests/runner/icite_discovery.rs:22-25` resolves the helper as the parent of the fixtures folder plus `scripts/`.
- Starts from: `release/gate.py:110-121` (staging), `crates/biomcp/tests/runner/icite_discovery.rs`, `scripts/icite_discovery_proof.py`, `scripts/check_icite_discovery.sh`.
- Rules: pick the simplest fix of two: stage the helper where the proof looks, or move the helper under the staged fixture tree and update its callers. Do not skip or weaken the proof in installed mode. Check whether any other tool proof resolves a path outside the staged tree, and fix those in the same change if they exist.
- Proof: a fixture test in `release/test_gate.py` fails when the staged tree lacks a file that a tool proof needs; an installed-layout run of `retained-drift-witnesses` from a prepared scratch passes on Linux. The next private rehearsal confirms Apple ARM and Linux ARM.
- Changes: release staging and the proof path only.
- Defers: the Linux evidence permission failures, owned by the Linux installed evidence permissions ticket.
