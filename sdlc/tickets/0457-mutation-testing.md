# 0457 — Run mutation testing once per release candidate

Status: COMPLETE.
Landed: 0dd91ad71fdcc37d81b6f092cd1073524433372d

Milestone: 5

Depends on: 0429, 0484

## Outcome

`cargo-mutants` is a pinned release tool in `release/toolchain.json`, installed by the existing owned installer into the native job environment under its own CARGO_HOME (m16827 ask 5); the published CLI runtime image gains no Rust tools. Each release candidate runs it as sharded `pm job`s, with `cargo test -p biomcp --test cases` as the test command, under a 90-minute total wall-time cap. It writes one survivor list attached to the candidate. Survivors do not block (ruling 5).

- Why: coverage shows which lines run, not whether a case fails when they break. Ruling 5 adopts mutation testing per candidate.
- Starts from: `release/toolchain.json`, the existing owned installer, `release/rehearse.py`.
- Rules: first, measure one shard of 20 mutants on the release build machine and record the cost per mutant. Then set the candidate's scope so its mutant count times that cost fits 90 minutes. Scope is the product files changed since the previous candidate, then `biomcp-query/src/status.rs`, `next.rs`, `biomcp-adapters/src/mapped.rs`, `crates/biomcp/src/spell.rs`, `grammar.rs` and the transport request path. A run that reaches the cap stops and lists the unrun mutants. The release owner turns at most ten survivors per candidate into outside-in cases, rule modules first, and lists the rest. No crate dependency is added.
- Proof: the measured shard's cost is recorded with its raw log; a planted survivor, a deleted assertion in the one case that covers a branch in `status.rs`, appears in the survivor list; restoring the assertion removes it.
- Changes: the files above and `docs/development.md`.
- Defers: nothing.
