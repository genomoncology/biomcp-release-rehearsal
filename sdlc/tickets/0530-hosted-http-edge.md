# 0530 — Keep the hosted HTTP edge answering under load

Status: COMPLETE.
Landed: 34532a947459f709fafdc2a4b07cc67061504e90

Milestone: 5

## Outcome

The hosted HTTP server answers `/health` before its capacity check, refuses an over-capacity connection with a fixed 503 without reading it, and records a failed connection instead of discarding it. A measured load test with `biomcp sync` running shows whether cache waits need an admission probe.

- Why: release review A8 and A4 and code review finding 9's hosted part (moved from 0525), weighed 2026-11-01. At capacity the accept thread reads the refused socket for up to 2 seconds and writes for up to 2 more (`crates/biomcp/src/mcp/http.rs:128-138`), and `/health` counts against `MAX_CONNECTIONS = 8`, so a load balancer can kill a busy task. `http.rs:145` drops connection errors with `let _ = service.handle(stream)`. `Cache::write` waits on the shared mutex with no admission probe (`cache.rs:160-170,286`). The hosted service goes live by 2026-11-30.
- Starts from: `mcp/http.rs`, `mcp/http/call.rs`, `wire.rs`, `cache.rs` and the hosted mode from 0502 to 0505.
- Rules: no reply changes. Connection errors go to the hosted server's existing diagnostics without request bodies or secrets, and never stop the server. The load test runs through a pm job with sync and a common-word gene search running together. If p95 latency of other calls exceeds 10 seconds, give `Cache::write` the admission probe that `try_sql` uses in this ticket. Otherwise record the numbers and stop.
- Proof: a test with eight long calls in flight gets 200 from `/health`; a connection that sends nothing at capacity gets 503 at once and does not delay the next accept; a client dropped mid-request is recorded and the server keeps serving; the load-test numbers are on the ticket.
- Changes: the hosted HTTP server and its tests, and the cache write path if the measurement requires it.
- Defers: scaling the write budget to body size, which the load balancer's buffering covers.
