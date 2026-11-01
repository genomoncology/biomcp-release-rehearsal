# Windows HGNC first snapshot recovery

Status: OPEN.
Milestone: 5

## Outcome

`cache-resilience/hgnc-first-snapshot-recovers` passes on Windows against the installed runner, so a failed first HGNC snapshot recovers there as it does on Linux and macOS.

- Why: private rehearsal run 41358845085 at ab41e914f failed this proof on Windows only with `local.failure` / `local_error database` (Yellow `biomcp-redux-evidence/0294/private-rehearsal-41358845085/windows-job.log` lines 978-979, repeated 1042-1043).
- Starts from: `crates/biomcp/tests/runner/hgnc_sync.rs` (`first`), the HGNC snapshot store and recovery path it exercises, and the raw log.
- Rules: start with a bounded diagnosis that names the failing database operation and its Windows cause, such as a file still open while it is replaced or removed, or a path or lock rule. Fix the product if the product fails on Windows, since a real user would hit it; fix the proof only if the proof alone is wrong. Do not skip the proof on Windows.
- Proof: a reproduction that fails before the fix, on Windows or by a faithful test of the Windows rule, and passes after; the next private rehearsal confirms Windows.
- Changes: the snapshot recovery path or the proof, whichever the diagnosis names.
- Defers: the offline eligibility list and Python invocation failures, each owned by its own Windows ticket.
