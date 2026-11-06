# 0678 — Small honesty gaps from the bench recheck

Status: COMPLETE.
Landed: fbcc6ad710d1227d249ca06a2b946116e60c775f

Milestone: 5

## Outcome

1. `search trial --offset` without `--pool` says a pool is needed and shows the corrected call.
2. A FAERS `--drug` value holding several names joined by `OR` is either searched as those names or refused with advice to pass `--drug` once per name; it never returns 0 rows silently.
3. `--resolver variantvalidator` is recorded only when VariantValidator answered; otherwise provenance names the resolver that did.
4. ClinGen's specification version comes from a ClinGen source field, or is omitted; it is never shown without a source.
5. Diagnostic search pages report a total, or say plainly that the source gives none.

## Why

The bench recheck of the 1.0 candidate (Jarvis m27105, report `experiments/513-biomcp-cli-codesign/climb/gaps-on-rc-ed9c36a96.md` in the bench repo) found no regressions and these open gaps. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).

## Proof

- Proof: Investigate first: reproduce each item on current main in single and typed mode; record evidence and drop any item main already satisfies. Replayed or installed cases for each remaining item. Fix general causes; tune nothing to the examples. Routine gate.

## Starts from

- Starts from: trial paging in `crates/biomcp-query/src/trial/`, FAERS query building, variant resolver provenance, the ClinGen adapter and the diagnostic search reply.

## Keeps

- Keeps: existing results and refusals other than these.

## Defers

- Defers: nothing. Split by item if a lane collides.

## Changes

- Changes: the owning query modules and `tests/`.
