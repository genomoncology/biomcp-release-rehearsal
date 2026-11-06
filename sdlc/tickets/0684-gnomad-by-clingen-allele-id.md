# 0684 — Variant lookups accept a ClinGen allele ID

Status: COMPLETE.
Landed: a78e1da71828ce669732397c782f650539189e89

Milestone: 5

## Outcome

`get variant` and `variant population` accept a ClinGen allele ID (CA…) as input, resolve it, and return gnomAD rows when gnomAD has the variant.

## Why

Claude's retest of the live typed server on 2026-11-06 (pin 1bb97e64; hosting m27741 via Jarvis m27744; evidence in `data/biomcp-hosting-evidence/2026-11-05-claude-eval-v2/`): gnomAD matching by ClinGen allele ID was still missing. 0660 resolves other inputs to a CAid; this is the CAid as input. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).

## Proof

- Proof: Investigate first: reproduce on current main in typed mode (the retest ran an older pin). If main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. Then replayed cases for an SNV and an indel given as CAids, and an unknown CAid that returns empty, not an error. Routine gate.

## Starts from

- Starts from: 0660's resolver and 0677's population normalization in `crates/biomcp-query/src/variant/`.

## Keeps

- Keeps: existing input forms and results.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-query/src/variant/`, `tests/`.
