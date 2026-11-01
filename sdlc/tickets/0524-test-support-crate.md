# 0524 — Share test support through one crate and check unused dependencies

Status: COMPLETE.
Landed: 0141bdd1b3ecccd47bd204540f863da78fdd2b79

Milestone: 5

## Outcome

Shared test helpers live in one `publish = false` test-support crate, and no test includes another crate's files by `#[path]`, so the unused-dependency check from 0532 also reads dev-dependencies correctly.

- Why: Jarvis's 1.0 code review (m17823, findings 5 and 6). `#[path]` shares `biomcp-fetch/tests/support/*` and `grpc_worker/support.rs` 148 times across three other crates, for example `crates/biomcp/tests/runner/alphagenome_tls.rs:8`. Tests also compile private product files, as `biomcp-query/tests/collection_refusals.rs:4` does. This forces eight `#[expect(dead_code)]` lines and misleads unused-dependency tools. The workspace decision of 2026-10-20 on outside-in coverage, rule 4, asks the routine gate to run the language's unused-dependency check.
- Starts from: every `#[path]` attribute under `crates/*/tests`, the dev-dependencies and the `lint` target.
- Rules: tests that compiled private product files reach the behavior through a public interface or the CLI, or the PM rules on each remaining one. Same-crate module files that use `#[path]` legitimately stay. Remove any ignore entries 0532 added only for these includes.
- Proof: no `#[path]` under `crates/*/tests` reaches another crate or a product source file; the eight `#[expect(dead_code)]` lines are gone; the landing gate passes with an unchanged case count.
- Changes: a new test-support crate, test imports, manifests and the Makefile.
- Defers: nothing.
