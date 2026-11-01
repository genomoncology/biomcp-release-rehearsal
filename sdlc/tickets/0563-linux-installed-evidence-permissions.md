# Linux installed runner evidence permissions

Status: OPEN.
Milestone: 5

## Outcome

The Linux install stage can write and retain proof evidence inside its copied check tree, so `schema-discovery/retains-failed-local-peer-state`, `schema-discovery/persistent-fixture-evidence` and `schema-discovery/persistent-native-evidence` pass in the container.

- Why: private rehearsal run 41358845085 at ab41e914f failed on Linux ARM with EACCES and ENOENT (linux-arm-job.log lines 1046-1054 and 1070-1074). Diagnosis (m19565): the shared default evidence folder is a compile-time path, `/source/crates/biomcp-test-support/../../target/fixture-evidence`, and the persistence proofs canonicalize the compiled `/source` checkout for their expected paths. The install container has no `/source` and runs as the host user, so creating `/source` fails with EACCES and the expected paths fail with ENOENT. An earlier guess that `cp -a` kept read-only modes was wrong.
- Starts from: `release/linux.sh`, `release/gate.py` staging, `crates/biomcp/tests/runner/fixture_evidence_proof.rs`, `crates/biomcp/tests/runner/proof.rs:50`, and the raw witnesses on Yellow at `biomcp-redux-evidence/0294/private-rehearsal-41358845085`.
- Rules: start with a bounded diagnosis from the raw witness: name the path that refused the write and why. Then fix the cause: installed runs resolve evidence and expected paths from the runtime scratch or fixture root, never from a compile-time checkout path. Do not chmod, do not run the container as root and do not loosen what the proofs assert.
- Proof: a reproduction of the install stage on Linux that failed before the fix and passes after, plus a test that fails when an installed run resolves any evidence path under the compiled source checkout. The next private rehearsal confirms Linux ARM and Linux Intel.
- Changes: the shared evidence and proof scratch path resolution only.
- Defers: the iCite helper staging, owned by the installed runner iCite helper ticket.
