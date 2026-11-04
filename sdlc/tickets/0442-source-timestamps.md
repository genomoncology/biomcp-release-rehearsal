# 0442 — Check source timestamps and give each property name one type

Status: COMPLETE.
Landed: 3806e07aae77b4fae503355d5fd340df9734c1fc

Milestone: 5

## Outcome

`SourceTimestamp.value` is checked RFC 3339 text that keeps its stated precision and offset. Every `source_updated` uses that one type, and `updated` and `fetched` each have one meaning across replies.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 6. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: `crates/biomcp-types/src/response/source_timestamp.rs:10-12`, `trial_header.rs:22`, `AdverseEventPage`, `DrugApprovals`, `DrugFdaPage`, `TrialCards` and `TrialPage`.
- Rules: a source date with no time stays a date; nothing invents precision or a timezone. Scope (m14375 batch, m14304 ask 5): the check enforces one type for each temporal property meaning (source dates, source timestamps, retrieval timestamps and `source_updated`) across all schemas. A universal one-type-per-property-name rule is not adopted: names such as `id`, `status`, `data` and `query` legitimately mean different things in different objects. `SourceOutcome.fetched` is a count and is renamed to say so. ClinicalTrials.gov `dataTimestamp` is UTC by its published API specification, so reading it as UTC invents nothing.
- Proof: `2024-05-03T10:00:00+02:00` reads and writes back unchanged, and a schema check fails when one temporal property maps to two types in any schema.
- Changes: the named types, schemas with a major bump, cases and the baseline.
- Defers: nothing.
