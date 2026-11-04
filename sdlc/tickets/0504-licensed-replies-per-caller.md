# 0504 — Never serve one caller's licensed reply to another

Status: COMPLETE.
Landed: 480d41e71d53d71fc43d4cc5256073b928d8cd8c

Milestone: 5

Depends on: 0503

## Outcome

A reply fetched with a no-fallback key is never returned to a caller without that same key: not from the request cache, a query pool, a snapshot or any in-memory memo. Public sources keep sharing cached replies with the key left out of the cache identity, as they do today.

- Why: Ian approved a shared hosted remote MCP server for 1.0, live before 2026-11-30 (m16469); a code audit at 79883e3ba checked each outcome against this repo. The cache identity is the credential-free URL (`biomcp-fetch/src/retry.rs`, `prepared.rs`), and OncoKB and NCI replies cache for a day, so with per-request keys a reply cached for one caller would serve the next. 0.9 leaked UMLS-enriched results this way.
- Starts from: `biomcp-fetch/src/cache_request.rs`, `prepared.rs`, the OncoKB and NCI adapters' freshness, query pools and snapshots in `biomcp-query/src/store`, and 0503's no-fallback tier.
- Rules: in hosted mode, mix a one-way fingerprint of the caller's key into the identity for no-fallback sources, or skip caching them, whichever is smaller; the fingerprint is never reversible to the key; outside hosted mode nothing changes.
- Proof: a case where caller A's OncoKB reply is cached and caller B, with a different key or none, gets a fresh request or a refusal, fails on the base build; a public-source reply is still shared across callers.
- Changes: the fetch cache identity, the store owners for pools and snapshots, and cases.
- Defers: nothing.
