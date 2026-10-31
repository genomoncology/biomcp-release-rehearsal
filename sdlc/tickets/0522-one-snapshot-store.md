# 0522 — Move every source snapshot onto one store

Status: OPEN.

Milestone: 6

Depends on: 0294

## Outcome

Every entity's source snapshot loads and saves through the generic `biomcp-query/src/source_snapshot.rs` store, and the seven hand-written snapshot modules are gone.

- Why: Jarvis's 1.0 code review (m17823, finding 2). Only organization and biomarker search use the generic store. Seven other modules, 815 lines in all, rebuild the same load and save. `author/snapshot.rs` and `author/papers_snapshot.rs` differ only in page type and cache key.
- Starts from: `source_snapshot.rs` and the seven modules, beginning with the two author snapshots.
- Rules: cache keys and stored bytes stay the same, so existing caches keep working. The broken-snapshot problem from 0519 stays.
- Proof: an empty reply diff, an unchanged schema comparison, and a case that reads a snapshot written by the previous build.
- Changes: `biomcp-query` snapshot code and its callers.
- Defers: nothing.
