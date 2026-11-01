# 0528 — Recover a complete bundle part instead of wedging on HTTP 416

Status: COMPLETE.
Landed: 08c9239ad94cb5b357d349fe7040f55d5b7e9273

Milestone: 5

## Outcome

A bundle sync whose kept part already holds the whole body recovers on the next sync: an HTTP 416 to its resume request resets the part and downloads again, and a part whose parse or publish failed is dropped rather than resumed.

- Why: release review A1 (2026-10-31, weighed 2026-11-01). When `consume` fails after the whole body is on disk (parse error, budget spent after parsing, or a fenced publish), `crates/biomcp-fetch/src/bundle/download.rs:102-107` keeps the part with bytes equal to total. The next sync sends `Range: bytes=<total>-` with `If-Match`, and the status loop at lines 169-181 handles only 412, 304, 200 and 206, so a 416 returns Rejected and keeps the part again until the publisher's ETag changes. It reaches HGNC, whose TSV host honours Range. ClinGen is CSV and is never resumed.
- Starts from: `bundle/download.rs`, `bundle/store.rs` (`retain`, `claim`, `reset`) and `crates/biomcp-fetch/tests/bundle/download.rs`.
- Rules: the first commit is the failing test. Treat 416 like 412 when the part holds bytes. Reset a part whose stream finished before `retain`. No other status handling changes.
- Proof: a test where sync 1 serves a full TSV body with a strong ETag and a header that fails parsing, and sync 2 asserts the resume request carries `range` and `if-match`, answers 416, and then publishes from a plain 200. It fails on the base and passes after.
- Changes: the bundle download code and its test.
- Defers: nothing.
