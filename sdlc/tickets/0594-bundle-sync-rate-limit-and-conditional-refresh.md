# Bundle sync reports rate limits honestly and refreshes conditionally

Status: OPEN.
Milestone: 5

## Outcome

A bundle sync that gets HTTP 429 reports reason `rate-limited` with the source's retry time, and stops at once when that time is past the deadline. Refreshing a stored bundle first asks the source whether it changed, with a HEAD request or a conditional GET using the stored ETag or modified date, and keeps the stored copy fresh on "not modified" without downloading it again.

- Why: hosting j28110 on 20bc8a9 (m23066). `biomcp sync gencc` failed twice after about 120 s with state `failed` and reason `syncing`, exit 4 and empty stderr. GenCC was answering HTTP 429 with retry-after 16433 and the body "Daily download limit reached (20 per day per IP) … use a HEAD request and If-None-Match or If-Modified-Since headers to check for changes without counting against this limit." The bundle store already saves each bundle's `etag` and `modified` (`crates/biomcp-fetch/src/bundle/publish.rs`), but refresh never sends them.
- Starts from: `crates/biomcp-fetch/src/bundle/download.rs` and `read.rs`, the retry-after parsing in `crates/biomcp-fetch/src/retry.rs`, the sync reply in `crates/biomcp/src/command/sync.rs`, and the GenCC adapter in `crates/biomcp-adapters/src/gencc.rs`.
- Rules:
  - This is general to every bundle source; GenCC is the witness.
  - A 429 with a retry time beyond the remaining deadline ends the sync at once. A short retry time may be waited out within the deadline.
  - The reply and stderr name `rate-limited` and the retry time; exit status stays the documented source-failure code.
  - The conditional check runs only when the source supports ETag or Last-Modified. A 304 or unchanged HEAD extends `fresh_until` without rewriting rows. A changed or unknown answer downloads as today.
  - A failed refresh keeps the stored copy.
- Proof: installed-binary cases against a local fixture server. A 429 with a long retry time ends quickly, reporting `rate-limited` and the retry time. A stored bundle with an ETag gets a 304 and stays fresh with no body download. A changed ETag downloads the new rows.
- Changes: the bundle download and refresh path, the sync reply detail, and cases.
- Defers: nothing.
