# 0785 — retain-valid-cache-entry-for-narrow-readers

Status: OPEN.

Milestone: 5


## Outcome

A reader with a smaller response limit skips a valid cached reply that exceeds its limit without deleting it. A later reader with a sufficient limit can still use that entry.

## Why

A narrow reader unnecessarily evicts a reusable reply. Ian's November 10 direction prioritizes remaining issues before release timing; see `decisions/2026-11-10-prioritize-remaining-product-issues.md`.

## Proof

- Through the public Fetch API, cache a valid reply, refuse it for a narrower reader, then prove a wider reader still hits the cache without another request. Preserve corruption eviction and prevent oversized stale fallback.

## Starts from

- `crates/biomcp-fetch/src/cache.rs` replaces oversized selected bodies with NULL and the generic invalid-entry branch deletes them. `tests/cache/storage.rs` currently expects deletion after the narrow-reader refusal.

## Keeps

- Reader limits, invalid-entry eviction, cache integrity, stale/fresh policy and the existing refusal/error behavior for the narrow reader.

## Defers

- Nothing; implement with the remaining cache correctness work before selecting the next final source.

## Changes

- Changes: `crates/biomcp-fetch/src/cache.rs`, `crates/biomcp-fetch/tests/cache/storage.rs`.
