# 0525 — Clean up disabled-source errors, literal text and dropped connection errors

Status: OPEN.

Milestone: 6

Depends on: 0294

## Outcome

One `SourceError::disabled(source)` constructor replaces the 14 written-out "source disabled" errors, static text literals become `Text` without fallible calls, and the hosted HTTP server reports a failed connection instead of discarding it.

- Why: Jarvis's 1.0 code review (m17823, finding 9). `biomcp-query/src` writes the disabled error 14 times, although `biomcp-fetch/src/request.rs:93` already refuses disabled sources. `monarch/mod.rs:145-158` maps static literals through fallible calls to `InputError::Serialization`. `biomcp/src/mcp/http.rs:145` drops connection errors with `let _ = service.handle(stream)`.
- Starts from: those sites and the hosted server's existing status and health reporting.
- Rules: no reply or problem code changes. A connection error goes to the hosted server's existing diagnostics channel without printing secrets or request bodies, and it never stops the server.
- Proof: an empty reply diff and an unchanged schema comparison; a hosted-mode test where a client drops mid-request shows the error recorded and the server still serving.
- Changes: `biomcp-query` source errors, the Monarch adapter and the hosted HTTP server.
- Defers: nothing.
