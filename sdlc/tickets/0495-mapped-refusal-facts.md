# 0495 — Carry each refused record as a fact on the adapter's mapped reply

Status: COMPLETE.
Landed: 7909f06ba7da0ca213189a120121a5c59130fc93

Milestone: 5

Depends on: 0418, 0474, 0494

## Outcome

`Mapped<T>` carries a refusal fact that each adapter sets where it actually refuses a record, and the query owners receive it. 0419 then decides "a dropped record makes the reply partial" from that fact instead of from `problems.is_empty()`, which today stands in for it in 39 query files.

- Why: the 0419 developer found that `Mapped<T>` has no refusal fact, so removing the `problems.is_empty()` predicates would mark replies with refused records complete (m15847 ask 3). The work spans 127 construction sites in 81 adapter files, too large to hide inside 0419.
- Starts from: `crates/biomcp-adapters/src/mapped.rs`, every `Mapped` construction site, the refusal sites the 0418 audit lists, the 39 query files that read `problems.is_empty()`, and the native window values from 0494.
- Rules: the fact is set only where a record is refused, never inferred from problem text or counts; 0494's families reuse its window value and add nothing beside it; `Mapped<SyncResult>` from 0474 carries the same fact; statuses do not change in this ticket, so the reply diff is empty.
- Proof: a planted refusal that skips the fact fails a focused adapter test in each adapter family; a check lists every `Mapped` construction site and fails on one without a refusal fact decision; the reply diff is empty.
- Changes: `mapped.rs`, the adapter construction and refusal sites, the query owners that receive the fact, and tests.
- Defers: status changes, which stay in 0419.
