# Windows offline eligibility list

Status: OPEN.
Milestone: 5

## Outcome

`live-source-checks/offline-eligibility` passes on Windows: `--live-check --list` reports `article/sections/grammar.yaml` as eligible and `article/sections/failure-500.yaml` as skipped, with the same case names on every platform.

- Why: private rehearsal run 41358845085 at ab41e914f failed on Windows only with "eligibility list omitted public capture or failure skip" (Yellow `biomcp-redux-evidence/0294/private-rehearsal-41358845085/windows-job.log` lines 985-986, repeated 1049-1050; `crates/biomcp/tests/runner/live_proof.rs:174-198`). One likely cause is case names built from native paths, with backslashes, but the diagnosis must show it.
- Starts from: `crates/biomcp/tests/runner/live_proof.rs` (`plan`), the live-check case listing in the runner, and the raw log.
- Rules: bounded diagnosis first: capture what the Windows listing actually reports for these two cases. Case names in reports use forward slashes on every platform if that is the cause. Do not loosen the assertion.
- Proof: a test that fails when a listing builds case names from a native path, and the next private rehearsal confirms Windows.
- Changes: the live-check listing or the proof, whichever the diagnosis names.
- Defers: the HGNC recovery and Python invocation failures, each owned by its own Windows ticket.
