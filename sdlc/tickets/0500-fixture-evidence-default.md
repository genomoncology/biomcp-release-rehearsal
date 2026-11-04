# 0500 — Keep fixture failure evidence after a gate job exits

Status: COMPLETE.
Landed: 7b3ecf2941006b4d1bb8919ae949e50f081f8abe

Milestone: 5

## Outcome

When `BIOMCP_EVIDENCE_DIR` is unset, the case runner writes fixture failure folders under the checkout's `target/fixture-evidence` instead of the job's temporary folder, so a full gate's evidence survives job cleanup until it is archived. An explicit `BIOMCP_EVIDENCE_DIR` still wins.

- Why: in gate j23024 three diagnostic folders were deleted by job cleanup before the live copy reached them, and that copy races the cleanup (m16372). The landing command forwards only thread variables, so the existing override never reaches a normal gate.
- Starts from: the fixture evidence writer in the case runner's test helpers.
- Rules: change only the default path; no environment hook, pm change or new config; the folder stays git-ignored under `target/`.
- Proof: a runner test with the variable unset shows a failing fixture writing under `target/fixture-evidence`, and fails on the current helper; a test with the variable set keeps the explicit path.
- Changes: the test helper and its test.
- Defers: nothing.
