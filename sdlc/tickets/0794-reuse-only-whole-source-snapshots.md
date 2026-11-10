# 0794 — Reuse only whole source snapshots

Status: COMPLETE.
Landed: ddefacff3a2e2c1f080446567e9f36bcb5e44190

Milestone: 5

Depends on: 0793

## Outcome

New searches never select a stored source snapshot with refused records, stale observations or failed work as their current answer. Its explicit pool remains addressable under existing expiry, ownership and source-policy rules.

## Why

Article pools enforce the whole-answer stance, while the shared source-pool path unconditionally publishes current mappings. Evidence: `records/2026-11-10-product-review.md`, F2.

## Starts from

- `crates/biomcp-query/src/source_snapshot.rs`, `store/pool.rs`, and every `find`, `publish`, `intern`, `publish_source_pool` and `extend_source_pool` caller: protein search and relations, GWAS threshold, pathway relations, ORCID, and study paging. Article current-pool publication is the existing comparison, not a second implementation to copy.

- PM decision, 2026-11-10: eligibility describes the retained observation, not whether every upstream page has been downloaded. Healthy lazy native windows remain reusable. Entity owners supply refusal/freshness/completeness facts from their saved state before rendering; empty source lists in local study paging must not be mistaken for failure.
- Make payload/metadata/current-mapping updates atomic. A new incomplete pool leaves an existing healthy current mapping intact. When an existing current pool becomes incomplete, remove only the mapping still naming that key and pool. A losing concurrent extension must not overwrite the retained winner's eligibility. Cover identical-payload early returns and `intern` reuse. Old partial current mappings must be refused or invalidated without deleting unrelated stored data or invalidating existing explicit pool IDs. Reading or extending an older explicit pool must not republish it over a newer healthy default.
- Reconstruct a new query using normal transport-cache policy; this does not promise a new upstream request or bypass a still-valid raw response cache. Preserve existing statuses for explicit failed continuations.

- Approved implementation plan: preserve snapshot identity/version and use a separate `biomcp.source_current/2` current-key namespace. Legacy defaults become unreachable while explicit IDs remain valid. Read the pointer, pool metadata and payload consistently in one database observation; recheck decoded eligibility for implicit reads. Eligibility updates also apply to identical-payload and intern reuse.

## Proof

- Routine public query/installed cases: initial refused member, a failed later native page followed by a new query after recovery, stale observations, and explicit reads of the older pool. Assert current eligibility and retained diagnostics separately from network traffic. Failed uncached pages may prove a new request; refusal cases must isolate raw cache or inspect the public pool identity.
- Exercise a healthy lazy window and local study paging; an incomplete publisher must not displace a healthy current answer. One controlled concurrent extension proves the accepted payload and eligibility agree. Include a legacy incomplete current mapping and a delayed extension whose pool is no longer current. Reuse existing cache/query fixtures.

## Keeps

- Caller/source exclusion isolation, fixed expiry, explicit continuation identity, received-count pagination, immutable article pools and normal raw-response caching. Follow `decisions/2026-11-06-review-stances.md`.

## Changes

- Changes: `crates/biomcp-query/src/source_snapshot.rs`, `crates/biomcp-query/src/store/pool.rs`, `crates/biomcp-query/src/protein/`, `crates/biomcp-query/src/gwas/threshold.rs`, `crates/biomcp-query/src/pathway/relation.rs`, `crates/biomcp-query/src/author/orcid.rs`, `crates/biomcp-query/src/study/page.rs`, `tests/cases/protein/`, `tests/cases/gwas/`, `tests/cases/pathway/`, `tests/cases/author/`, `tests/cases/study/`, `crates/biomcp-query/tests/`.

## Defers

- Changes to raw-response cache admission or failed-result presentation require their own demonstrated defect. No new store version or general caching framework by default.
