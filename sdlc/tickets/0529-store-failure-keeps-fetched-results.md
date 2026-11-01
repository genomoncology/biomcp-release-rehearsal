# 0529 — Keep fetched results when the store fails, and keep stale replies

Status: COMPLETE.
Landed: 019a93e884b5aa5396a6e80e5eb7d56ab6a37c7b

Milestone: 5

## Outcome

When a snapshot or pool publish fails, a command returns the results it already fetched, with the cache problem recorded, no next-page token and a retry step, as transport replies already do. Store open writes only when its schema version is missing or old. Expired replies stay available as the stale and offline fallback for a grace period instead of being deleted on every write.

- Why: release review A3, A5 and A6 (weighed 2026-11-01). `store.publish(...)?` at `crates/biomcp-query/src/article/mod.rs:128` turns a late store failure into a local failure with no items, and the same `?` sits at about 15 more sites (`article/edges.rs:77`, `article/recommend.rs:80`, `author/orcid.rs:117`, `author/snapshot.rs:95,97`, `author/papers_snapshot.rs:89,91`, `protein/search.rs:138-139`, `protein/source_relations.rs:217`, `protein/source_relations/saved.rs:128,136`, `protein/relations/retained.rs:75`, `gwas/threshold.rs:168-169`, `study/page.rs:36`, `pathway/relation_snapshot.rs:87`). `Store::open` takes the write lock and commits on every open (`store/mod.rs:78-96`). `Cache::write` runs `DELETE FROM reply WHERE fresh_until<=now` on every insert (`cache.rs:292`), while `cache_request.rs:78-112` serves expired replies as a fallback, and the comment in `store/cleanup.rs:1` says the opposite.
- Starts from: those sites, the transport's `local_write_busy_keeps_the_checked_live_reply_and_deadline` behavior, `store/mod.rs`, `cache.rs` and `store/cleanup.rs`.
- Rules: the first commit is a failing CLI case for article search with the store held busy late in the budget. One helper serves every site; no site keeps the bare `?`. Expired replies are pruned only once they are 30 days past `fresh_until`, and by `store clean`. Store open reads `meta` first and writes only when needed. Do not touch `crates/biodata`.
- Proof: the article search case returns its fetched items with the cache problem and retry step; a store-open trial shows no write transaction when the version matches; a cache trial shows an expired reply still served stale after an unrelated live write, and gone after the grace period.
- Changes: `biomcp-query` publish sites and one helper, the store and cache code, their cases and the cleanup comment.
- Defers: nothing.
