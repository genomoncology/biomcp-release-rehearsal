# Whole-project code quality review

Date: 2026-10-28

Snapshot: 158243cc2f6ec1b374a48914ffab3089e5bb4a98, clean detached tree. Ian asked for an adversarial review of architecture, code, outside-in tests, coverage, dead code, separation of concerns, locality of behavior and Rust standards. Four fresh read-only reviewers covered architecture, Rust code quality, tests and coverage, and the adapter and query crates. They ran no builds, tests or provider calls. The PM confirmed the trial status gap live: ClinicalTrials.gov reports 38 studies with `TEMPORARILY_NOT_AVAILABLE`, which `biodata::TrialStatus` lacks.

## Verdict

The crate graph, import direction, isolation of test processes, secret handling and lint discipline hold. No unsafe code, stray panics, TODOs or commented-out code. The weaknesses are locality and duplicated truth: the binary re-implements query behavior, each command and each source is described in many hand-kept tables with silent fallbacks, and about half the adapters reject a whole reply for one bad value. The release binary runs one YAML case, the coverage floor counts in-process library execution, and the architecture document no longer describes the code.

## Findings and tickets

| Finding | Severity | Ticket |
|---|---|---|
| `TrialStatus` lacks `TEMPORARILY_NOT_AVAILABLE`; strict ctgov raw types fail a whole page | High | 0417 |
| About 30 adapters decode non-identity fields strictly or pass them through unchecked; audit ratchet sees only `.ok()`; `#[serde(default)]` and 300-line rules check only some files | High | 0418 |
| Status rules copied per entity and disagreeing; next steps built through `json!` with failures dropped; query boilerplate | Medium | 0419 |
| Binary re-implements label bounding, step order, step sources, disease section support and sync; provider query syntax and source mapping in biomcp-query; raw types exported | Medium | 0420 |
| Each command described in about eight hand-kept tables; `QueryBody` erases the request link; usage errors travel as bodies; exit codes hard-coded | Medium | 0421 |
| Each source described in many tables with `_` fallbacks (bundle defaults to HGNC, key help to OncoKB, probes to rejected); registry `expect`; HGNC knowledge in the transport | Medium | 0422 |
| AlphaGenome codec and request building live in the transport and query layers, not the adapter | Medium | 0423 |
| Copy-paste families: bad-reply errors, openFDA envelope, GraphQL alias batching, `received_date`, XML root search, problem helpers, bundle partition misuse; variant sections run serially; two snapshot tables | Medium | 0424 |
| Dead and test-only public items, 8 never-built error messages shipped in schemas, lint settings that differ from the standard, unchecked `schema` feature, split words in strings, dependency duplicates | Medium | 0425 |
| Stray biodata error types and macros, string and unit errors, `non_exhaustive` internal enums, lossy secret reads, 0 meaning unknown time | Medium-low | 0426 |
| Grammar rebuilt three or more times per call, config reread, reply serialized and reparsed | Medium | 0427 |
| 32 `Leg` request methods named by option combination; decoder in the XML module; positional tuples; paging methods copied 19 times | Low | 0428 |
| Release artifact runs 1 of 654 YAML cases and no Rust trials | Medium | 0429 |
| Coverage floor counts in-process library calls and stale contract targets; unchanged since 2026-10-24 | Medium | 0430 |
| Feature gate covers only YAML; docs-generate rewrites help expectations; default config and cache paths untested; two tests that never run; expectations taken from product tables; weak ratchets; proof tooling in the routine target; three budget formats | Medium | 0431 |
| Architecture document stale in transport, dependency table, adapter shape, API sketches and file lists; public query surface wider than designed | Medium | 0432 |

## Strengths

Cargo enforces the documented layer direction with no cycles. Adapters never retry, sleep, read the clock or set quotas. Product code has two reasoned `expect` exceptions and no `unwrap`. Child processes get one isolated environment. Replay uses virtual time. Checkers have planted-failure tests. Every declared dependency is used.
