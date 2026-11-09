# 0785 — retain-valid-cache-entry-for-narrow-readers

Status: OPEN.

Milestone: 6

Depends on: 0294

## Outcome

A reader with a smaller response limit skips a valid cached reply that exceeds its limit without deleting it. A later reader with a sufficient limit can still use that entry.

## Why

Ian's November 9 review found that a narrow reader unnecessarily evicts a reusable reply and explicitly allowed this minor fix after release.

## Proof

- Through the public Fetch API, cache a valid reply, refuse it for a narrower reader, then prove a wider reader still hits the cache without another request. Preserve corruption eviction and prevent oversized stale fallback.

## Starts from

- `crates/biomcp-fetch/src/cache.rs` replaces oversized selected bodies with NULL and the generic invalid-entry branch deletes them. `tests/cache/storage.rs` currently expects deletion after the narrow-reader refusal.

## Keeps

- Reader limits, invalid-entry eviction, cache integrity, stale/fresh policy and the existing refusal/error behavior for the narrow reader.

## Defers

- Implement after 1.0 publication as Ian allowed. This ticket does not block the release.

## Changes

- Fetch cache lookup and its owning two-reader storage test.
