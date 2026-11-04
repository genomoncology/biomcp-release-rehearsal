# 0450 — Bound and clean source text

Status: COMPLETE.
Landed: 8a1b105d80801f3c3970e554fca9e6ee39d45ff2

Milestone: 5

## Outcome

`Text` refuses or escapes C0, DEL, C1 and bidirectional control characters and enforces a documented length cap in its constructor. `TrialReference.kind` becomes a closed enum and `TrialDocument.uploaded` a checked timestamp.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 17. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: `crates/biodata/src/text.rs` and the trial types.
- Rules: escape rather than refuse where refusing would drop a whole source record; follow the bad-value rule.
- Proof: `Text` refuses a bidirectional override, and a JSON reply whose source text holds one shows it escaped.
- Changes: the named types, schemas and cases.
- Defers: nothing.
