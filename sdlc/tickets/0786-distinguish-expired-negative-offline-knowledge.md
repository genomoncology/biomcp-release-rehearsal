# 0786 — distinguish-expired-negative-offline-knowledge

Status: OPEN.

Milestone: 6

Depends on: 0294

## Outcome

Offline replies distinguish a missing cached answer from a previously absent result whose freshness has expired, while refusing to claim stale absence as current fact.

## Why

Ian's November 9 review found that expired negative knowledge becomes a bare offline diagnostic and explicitly allowed this minor improvement after release.

## Proof

- Through the public Fetch API, contrast fresh cached absence, expired cached absence and no cached entry in offline mode. The expired case states its historical uncertainty, performs no network request and never becomes a successful current absence result.

## Starts from

- `crates/biomcp-fetch/src/cache_request.rs` discards expired cached 404 entries into the same path as a miss; `tests/offline.rs` currently expects plain Offline after negative expiry.

## Keeps

- Offline mode performs no network calls, negative-entry freshness rules remain intact, current absence requires valid evidence and callers still receive a truthful non-success result when current knowledge is unavailable.

## Defers

- Implement after 1.0 publication as Ian allowed. Choose the smallest compatible diagnostic shape during implementation; this ticket does not block the release.

## Changes

- Fetch negative-cache/offline diagnostic mapping and its existing owning tests.
