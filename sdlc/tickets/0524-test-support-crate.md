# 0524 — Share test support through one crate and check unused dependencies

Status: OPEN.

Milestone: 6

Depends on: 0294

## Outcome

Shared test helpers live in one `publish = false` test-support crate, no test includes another crate's files by `#[path]`, and `make lint` runs `cargo machete` in its default mode.

- Why: Jarvis's 1.0 code review (m17823, findings 5 and 6). `#[path]` shares `biomcp-fetch/tests/support/*` and `grpc_worker/support.rs` 148 times across three other crates, for example `crates/biomcp/tests/runner/alphagenome_tls.rs:8`. Tests also compile private product files, as `biomcp-query/tests/collection_refusals.rs:4` does. This forces eight `#[expect(dead_code)]` lines and misleads unused-dependency tools. The workspace decision of 2026-10-20 on outside-in coverage, rule 4, asks the routine gate to run the language's unused-dependency check.
- Starts from: every `#[path]` attribute under `crates/*/tests`, the dev-dependencies and the `lint` target.
- Rules: tests that compiled private product files reach the behavior through a public interface or the CLI, or the PM rules on each remaining one. Never use `cargo machete --with-metadata`. The lint fails on an unused dependency.
- Proof: `rg '#\[path' crates/*/tests` is empty; the eight `#[expect(dead_code)]` lines are gone; `make lint` fails when an unused dependency is added on purpose; the landing gate passes with an unchanged case count.
- Changes: a new test-support crate, test imports, manifests and the Makefile.
- Defers: nothing.
