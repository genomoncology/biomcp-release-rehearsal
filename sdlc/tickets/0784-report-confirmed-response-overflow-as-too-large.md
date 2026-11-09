# 0784 — report-confirmed-response-overflow-as-too-large

Status: COMPLETE.
Landed: d29e9c78f5e60706c0ceca727c8e9e5eab3659c8

Milestone: 5

## Outcome

Default JSON, XML, probe and byte readers report confirmed response-size overflow as too large, while malformed or damaged responses remain bad reply. Explicit caller overflow overrides and all byte/allocation bounds remain effective.

## Why

The second item in idea 0760 is still unfixed and explicitly due before the cut; known size overflow currently receives a misleading bad-reply diagnostic.

## Proof

- Update the existing public Fetch overflow cases to expect too large and first demonstrate them failing on current main. Cover ordinary body overflow and expanded gzip layers with the existing tests; retain malformed-body and explicit override checks. Inspect replay behavior and keep genuine recording corruption separate from response overflow.

## Starts from

- `crates/biomcp-fetch/src/read.rs` initializes default overflow reasons to BadReply. `request_body.rs` and `request.rs` propagate them for known overflows. Existing `tests/transport.rs`, `tests/double_gzip.rs` and `tests/record/bytes.rs` explicitly assert current diagnostics.

## Keeps

- Existing limits, decompression/allocation protections, explicit `Read::bounded` reason overrides, true malformed-response errors, recording integrity and status/exit mapping. Do not broaden accepted payload sizes or mask a corrupt cache/recording as successful data.

## Defers

- Narrow-reader cache retention and expired offline absence diagnostics are explicitly post-1.0 follow-ups.

## Changes

- Fetch read overflow defaults/mapping and their existing owning public API tests.
