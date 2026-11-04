# 0505 — Pace each caller's key separately in hosted mode

Status: COMPLETE.
Landed: 0c059c3f7b43bb92c579b4174849b60b3c5c10aa

Milestone: 5

Depends on: 0503

## Outcome

In hosted mode, each source keeps one pacing bucket per caller key and one shared bucket for keyless traffic, the server caps concurrent calls, and each MCP call has a wall-clock deadline that covers the whole call, not only its source legs. A `--rate-share` setting between 0 and 1 scales every per-key and per-source rate, so N instances each take 1/N of a key's limit (m16520).

- Why: Ian approved a shared hosted remote MCP server for 1.0, live before 2026-11-30 (m16469); a code audit at 79883e3ba checked each outcome against this repo. Pacing exists per source but tracks only keyed versus keyless traffic for the whole process (`biomcp-fetch/src/quota.rs`), and the 20 second budget covers source legs, with no timer around an HTTP worker (`mcp/http.rs`).
- Starts from: `biomcp-fetch/src/quota.rs` `Limiters`, `mcp/http.rs` and `config.rs` `BUDGET`.
- Rules: buckets are keyed by the key's one-way fingerprint from 0504, never the key; idle buckets expire; the deadline returns a typed timeout; pacing across several server instances belongs to the front end, which sets `--rate-share`; nothing here coordinates instances.
- Proof: cases with two header keys show one caller's exhausted bucket not slowing the other; keyless calls share one bucket; a call held past the deadline returns the timeout; each fails on the base build.
- Changes: the quota limiter, the HTTP worker, cases and docs.
- Defers: cross-instance pacing.
