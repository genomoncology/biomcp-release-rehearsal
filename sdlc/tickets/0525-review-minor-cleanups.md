# 0525 — Clean up disabled-source errors and literal text

Status: COMPLETE.
Landed: 91bcdc494d2038c3ef563eee44d582ec3d8a0e86

Milestone: 5

## Outcome

One `SourceError::disabled(source)` constructor replaces the 14 written-out "source disabled" errors. The static-literal clause moved to 0459, because it needs a BioData constructor (m18414).

- Why: Jarvis's 1.0 code review (m17823, finding 9). `biomcp-query/src` writes the disabled error 14 times, although `biomcp-fetch/src/request.rs:93` already refuses disabled sources. `monarch/mod.rs:145-158` maps static literals through fallible calls to `InputError::Serialization`. The review's hosted connection-error part moved to 0530 (release review weighing, 2026-11-01).
- Starts from: those sites.
- Rules: no reply or problem code changes.
- Proof: the case suite passes unchanged and the schema check passes; `rg` finds one disabled-source constructor.
- Changes: `biomcp-query` source errors.
- Defers: nothing.
