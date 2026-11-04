# 0522 — Move every source snapshot onto one store

Status: COMPLETE.
Landed: 390e9c65deac799173655e4ed9a0d15b9a93981f

Milestone: 5

## Outcome

Every entity's source snapshot loads and saves through the generic `biomcp-query/src/source_snapshot.rs` store, and the seven hand-written snapshot modules are gone.

- Why: Jarvis's 1.0 code review (m17823, finding 2). Only organization and biomarker search use the generic store. Seven other modules, 815 lines in all, rebuild the same load and save. `author/snapshot.rs` and `author/papers_snapshot.rs` differ only in page type and cache key.
- Starts from: `source_snapshot.rs` and the seven modules, beginning with the two author snapshots.
- Rules: cache keys and stored bytes may change, because no released 1.0 cache exists and Ian's no-compatibility stance carries no migration; a store from an older build is rebuilt, never misread. The broken-snapshot problem from 0519 stays.
- Proof: the case suite passes unchanged and the schema check passes, with the reply diff reviewed as a diagnostic; a case shows a snapshot from an older store version rebuilt rather than misread.
- Changes: `biomcp-query` snapshot code and its callers.
- Defers: nothing.
