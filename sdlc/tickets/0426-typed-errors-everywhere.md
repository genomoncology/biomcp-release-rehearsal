# 0426 — Use typed errors everywhere

Status: COMPLETE.
Landed: 33ff671c8e67196ba4d753851968bfce74f8c2c6

Milestone: 5

Depends on: 0434

## Outcome

biodata declares every error code once, the stray error types fold into it, and one shared checked-ID macro replaces the local ones. No public signature returns a string or unit error. Internal enums drop `non_exhaustive` and callers match exhaustively. Error conversions keep the IO kind. Environment values, keys included, are read as UTF-8 and refused if invalid. Recording times use `Option` instead of 0 for unknown. Library decode errors on `wire_enum!` types leave the input value out of their text, and `FiniteNumberError`, `GwasValueInvalid`, `ReportIdInvalid`, `PathwayIdInvalid`, `AuthorIdInvalid`, `CellLineIdInvalid` and the `&'static str` error in `TrialCoordinates` fold into the declared codes. `License` and `ArticleMatchKind` stop reporting `source.unknown`, and a protein `Bound` with both ends missing stops reporting `coord.span.order`.

- Why: the pre-1.0 code quality review `sdlc/records/2026-10-28-code-quality-review.md`, and the BioData review of BioMCP 1.0's data model (2026-10-28), ask 20.
- Starts from: `crates/biodata/src/error.rs` and the named ID modules, `crates/biodata/src/macros.rs`, `crates/biomcp/src/grammar.rs`, `config.rs`, `exit.rs`, `main.rs`, `crates/biomcp-fetch/src`, `declaration.rs`, `record.rs`, `cache.rs` and every direct consumer of `ReplyMetadata.fetched`: the transport decode, cache and request paths and about 20 stored snapshot query consumers, all moved to `Option` in this ticket so no partly migrated API lands (m13767). A public wire change is reported to the PM before it lands. Approved (m13772): `ArticleResolution::Resolved.fetched` becomes optional and is omitted when the time is unknown; known values keep their form, and the two article callers stop writing 0. Approved (m13780, field names corrected by m13782): `StudyProfiles.acquired`, `StudySampleLists.acquired` and `StudyFiles.acquired` (study-profiles, study-sample-lists and study-files) become optional the same way, and the empty-directory case stops writing 0; `StudySnapshot.acquired` stays required from bundle retrieval time.
- Rules: error code strings stay stable; a test checks every biodata display code against the declaration. Added by the restructure: `non_exhaustive` leaves every workspace enum, `Source`, `SecretName`, `TrialStatus`, `TrialPhase` and `StudyType` included, which settles the choice 0452 defers. The grammar's `Result<_, ()>` helpers and `From<()>` go, and a text ratchet refuses both in product code. The exit literals at `exit.rs:18` and `main.rs:51-52` use `Status::code()`, and a ratchet refuses an integer literal passed to `ExitCode::from`.
- Proof: a library decode of an unknown enum value has error text without the value; the code test fails on a planted stray code; a case refuses an invalid UTF-8 key without printing it; a planted `Result<_, ()>` and a planted `ExitCode::from(5)` each fail their ratchet.
- Changes: the files above and cases.
- Defers: nothing.
