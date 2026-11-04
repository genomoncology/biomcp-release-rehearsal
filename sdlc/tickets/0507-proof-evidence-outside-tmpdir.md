# 0507 — Keep every intended test evidence folder outside the job's temporary folder

Status: COMPLETE.
Landed: d3b8baed926397824553701ad62426894fbbfa57

Milestone: 5

## Outcome

Every test helper that keeps evidence on purpose writes it under the checkout's `target/` evidence folder from 0500, not under the job's temporary folder. The drug next-step native proof (`drug_next/native.rs` `evidence_root`, logged as `NATIVE_DRUG_NEXT_PROOF`) is the known case; any other helper that keeps a temporary folder for evidence moves the same way.

- Why: in gate j23139 job cleanup deleted 26 files under `TMPDIR/biomcp-native-proof/drug-next`, including replay `mcp.stdout` outputs, before archival (m16537). 0500 fixed only the fixture failure writer, so the same cause repeated.
- Starts from: 0500's default evidence path, `crates/biomcp/tests/runner/drug_next/native.rs`, and a bounded search of the test helpers for temporary folders that are kept (`keep()`, `into_path`, logged evidence paths).
- Rules: one shared evidence path helper, reused from 0500; `BIOMCP_EVIDENCE_DIR` still wins; temporary folders that hold no evidence stay temporary; no watcher or new framework.
- Proof: a runner test deletes the job's temporary tree after a proof finishes and finds the evidence still present; it fails on the current helper. The search's findings are listed in progress.
- Changes: the test helpers that keep evidence, and a test.
- Defers: nothing.
