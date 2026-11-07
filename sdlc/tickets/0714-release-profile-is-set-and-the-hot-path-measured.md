# 0714 — Release profile is set and the hot path measured

Status: COMPLETE.
Landed: f636524bbb3b3ef6b3fbd5d2875fec06d143d0a5

Milestone: 5

## Outcome

The workspace states its release build settings, and one recorded measurement shows where a warm and a cold call spend time, including connection setup and store open.

## Why

Row W26 of `records/2026-11-06-pre-release-review.md`, first deferred. `Cargo.toml` has no `[profile.release]`. Ian, 2026-11-06, wants the project "as clean and as free of bugs and issues as possible".

## Proof

- Proof: a recorded-reply timing run before and after, under `pm job`, with numbers in the ticket. Any fix follows the measurement. If nothing dominates, say so and change only the profile.

## Starts from

- Starts from: `Cargo.toml`, the serve path in `crates/biomcp/src/mcp/`.

## Keeps

- Keeps: behavior and binary channels. No live provider load.

## Defers

- Defers: nothing.

## Changes

- Changes: `Cargo.toml`, and code only where the numbers point.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

Keep `opt-level = 3` and unwinding. `release/artifacts.py inspect()` refuses any other opt level, and `catch_unwind` in `crates/biomcp/src/mcp/worker.rs`, `crates/biomcp-fetch/src/batch.rs` and `crates/biomcp-fetch/src/cache/contention.rs` needs unwinding.

Each release section builds three binaries in this profile, so link-time optimization is paid three times per platform, and the Intel build has already used 119m44s of its 120 minute limit. Land this after 0697 raises the limit, and record the build time next to the size.

Recorded replies open no connection, so they cannot measure connection setup. Measure it against a local TLS stub, or state it as unmeasured. This is one measurement: no harness stays behind.

## Evidence

Job j34553, at 98e8a6276: recorded trial reply `get trial NCT02563002 --section outcomes`, 4 booked cores, release builds with no compiler cache. Medians, p95 in brackets.

| | Default profile | Thin LTO, one codegen unit |
|---|---|---|
| Build time | 149 s | 314 s |
| Binary size | 50.6 MB | 39.3 MB |
| Process start (`version`) | 8.3 ms | 7.9 ms |
| CLI call, new store | 21.9 ms (30.8) | 20.2 ms (42.0) |
| CLI call, store exists | 15.8 ms | 14.6 ms |
| `serve` start to initialize reply | 9.4 ms | 7.8 ms |
| First `serve` call | 5.6 ms | 4.5 ms |
| Later `serve` calls | 2.8 ms (4.1) | 2.4 ms (4.0) |

Nothing dominates. A new store costs about 6 ms over an existing one, which is the store open. Connection setup is unmeasured: recorded replies open no connection, and no TLS stub was built. Ruling, 2026-11-07: option A, the default settings written out, because twice the build time for 1 to 2 ms per call is not worth it.
