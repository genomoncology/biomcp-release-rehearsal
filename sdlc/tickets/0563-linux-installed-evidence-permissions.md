# Linux installed runner evidence permissions

Status: OPEN.
Milestone: 5

## Outcome

The Linux install stage can write and retain proof evidence inside its copied check tree, so `schema-discovery/retains-failed-local-peer-state`, `schema-discovery/persistent-fixture-evidence` and `schema-discovery/persistent-native-evidence` pass in the container.

- Why: private rehearsal run 41358845085 at ab41e914f failed on Linux ARM with Permission denied in `retains-failed-local-peer-state` and missing directories after evidence retention hit Permission denied. The install stage runs as the host user and does `cp -a /proof /tmp/check` (`release/linux.sh:56-65`), which keeps the copied modes from the read-only proof mount.
- Starts from: `release/linux.sh`, `release/gate.py` staging, `crates/biomcp/tests/runner/fixture_evidence_proof.rs`, `crates/biomcp/tests/runner/proof.rs:50`, and the raw witnesses on Yellow at `biomcp-redux-evidence/0294/private-rehearsal-41358845085`.
- Rules: start with a bounded diagnosis from the raw witness: name the path that refused the write and why. Then fix the cause, for example by copying without preserved read-only modes or by making the proof write under its own scratch. Do not run the container as root and do not loosen what the proofs assert.
- Proof: a reproduction of the install stage on Linux that failed before the fix and passes after, plus a fixture test that catches a read-only staged tree. The next private rehearsal confirms Linux ARM and Linux Intel.
- Changes: the Linux install stage or proof scratch paths only.
- Defers: the iCite helper staging, owned by the installed runner iCite helper ticket.
