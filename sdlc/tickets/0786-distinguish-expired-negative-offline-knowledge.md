# 0786 — distinguish-expired-negative-offline-knowledge

Status: OPEN.

Milestone: 5


## Outcome

Offline replies distinguish a missing cached answer from a previously absent result whose freshness has expired, while refusing to claim stale absence as current fact.

## Why

Expired negative knowledge becomes a bare offline diagnostic. Ian's November 10 direction prioritizes remaining issues before release timing; see `decisions/2026-11-10-prioritize-remaining-product-issues.md`.

- PM decision, 2026-11-10: retain the existing Offline failure category and prefer a fixed historical-absence diagnostic over a new public error category. Carry the smallest internal distinction needed to preserve that knowledge; never return stale absence as current fact.

## Proof

- Through the public Fetch API, contrast fresh cached absence, expired cached absence and no cached entry in offline mode. The expired case states its historical uncertainty, performs no network request and never becomes a successful current absence result.

## Starts from

- `crates/biomcp-fetch/src/cache_request.rs` discards expired cached 404 entries into the same path as a miss; `tests/offline.rs` currently expects plain Offline after negative expiry.

## Keeps

- Offline mode performs no network calls, negative-entry freshness rules remain intact, current absence requires valid evidence and callers still receive a truthful non-success result when current knowledge is unavailable.

## Defers

- Choose the smallest diagnostic shape that distinguishes expired knowledge without asserting current absence. No general cache redesign.

## Changes

- Changes: `crates/biomcp-fetch/src/cache_request.rs`, `crates/biomcp-fetch/src/error.rs`, `crates/biomcp-fetch/tests/offline.rs`, `crates/biomcp-query/src/status.rs`.
