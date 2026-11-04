# 0440 — Make every schema pattern match its parser

Status: COMPLETE.
Landed: adc6e56c9b905c8e4d1ffcdb0d77430e0de30914

Milestone: 5

## Outcome

Every text value's schema pattern accepts exactly what its parser accepts: `CaId` refuses `CA000000`, `VcvId` refuses `VCV000000000`, `HgvsText::PATTERN` refuses `(123)` references and `.0` versions, and `Doi` refuses C1 control characters, with any other mismatch the test finds.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 4. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: `crates/biodata/src/variant/id.rs:37`, `HgvsText::PATTERN`, `Doi` and every type built with `text_value!`.
- Rules: fix the pattern or the parser, whichever matches the published format.
- Scope (PM ruling 2026-10-28): schema patterns match the parser's grammar and finite bounds exactly. Checksums (such as ORCID), calendar validity (such as 2023-02-29) and relational rules (such as a start not after its end) and UTF-8 byte limits (such as MyVariantQuery, whose schema states a character maximum) stay in the parsers, and each such schema's description names the extra rule. The parity table lists those types with the strings the schema accepts and the parser refuses, so the gap is explicit, never silent.
- Proof: a table in `tests/schemas/values.rs` that asserts `validator.is_valid(s) == s.parse::<T>().is_ok()` on near-miss strings for every `text_value!` type.
- Changes: the patterns, the table and the baseline.
- Defers: nothing.
