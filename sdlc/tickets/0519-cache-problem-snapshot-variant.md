# 0519 — Report broken snapshots and author lookups truthfully

Status: OPEN.

Milestone: 5

## Outcome

A broken stored snapshot reports its own cache problem instead of `fetch.cache_database`, and the author papers path keeps the author ID that the dispatcher already matched, so it never returns a database error for a lookup that succeeded.

- Why: Jarvis's 1.0 code review (m17823, finding 4) found about 116 sites that map JSON encode and decode failures, missing pool state and logic mismatches to `CacheError::Database`. In the worst case `author/papers.rs:7` drops the matched ID, then `author/semantic_scholar.rs:77` matches it again and returns a database error. `CacheProblem` is public; the generated schemas change with it, and 0336 re-exports the baseline from final source.
- Starts from: `CacheError` and `CacheProblem` in the fetch layer, the snapshot load and save paths, `author/papers.rs`, `author/semantic_scholar.rs` and the ORCID branch that already passes the ID.
- Rules: add exactly one variant for a stored snapshot that fails to decode or encode, with a dotted code that follows the existing pattern. The author papers path takes the ID from the dispatcher, as the ORCID branch does. Real SQLite failures stay `fetch.cache_database`. Do not reclassify the other sites in this ticket. No reply changes except the corrected problem code for a broken snapshot. Do not touch `crates/biodata`.
- Proof: a CLI case where a corrupted stored snapshot reports the new code and exit status, and a case where an author papers lookup by name returns papers instead of a database error. `make schema-generate` regenerates the checked-in schemas, and the only schema change is the added variant.
- Changes: the fetch layer's cache error types, the snapshot paths, the author papers path, the cases and the generated schemas.
- Defers: reclassifying the remaining catch-all sites, which needs a measured list first.
