# 0517 — Move the tools target off removed transport calls and gate it

Status: COMPLETE.
Landed: 159494d4a4fffae410c463ea543673d92d16fea9

Milestone: 5

## Outcome

The `tools` test target compiles again: `crates/biomcp/tests/runner/live_input_context.rs:51` and `runner/fixture_evidence_proof.rs:56` use the current `call`, `Input` and `Read` API instead of the removed `Leg::json` and `Leg::bytes`, with no behavior change. The landing gate compiles and lints the `tools` target, so a later transport change cannot land while it is broken.

- Why: j24204 on 0432 (m17761) failed to compile `tools` after 0428 landed. The 0428 gate passed because `tools` has `test = false` in `crates/biomcp/Cargo.toml`, and `cargo clippy --all-targets` checks only targets with `test = true`.
- Starts from: the two callers, the `lint` target in `Makefile` and 0428's transport API.
- Rules: the 64-entry endpoint bound, credentials, proxy handling and existing assertions stay; `lint` names `--test tools` explicitly rather than changing the target's `test` flag; batches with 0432.
- Proof: the gate's lint step fails on the base build and passes after; the existing tools proofs that use the two callers pass.
- Changes: the two runner files and `Makefile`.
- Defers: nothing.
