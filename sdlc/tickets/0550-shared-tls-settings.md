# 0550 — Build TLS settings once for hosted calls

Status: COMPLETE.
Landed: c9394cc5fb17ede1fc40a6703ebf3de125f750cc

Milestone: 5

## Outcome

A hosted call reuses one process-wide TLS configuration and platform verifier instead of building new ones, so per-call agents no longer rebuild certificate settings and can resume TLS sessions with source hosts.

- Why: the hosted team's capacity review (m18618, Ian approved the request). `Fetch::with_cancellation` (`crates/biomcp-fetch/src/cancellation.rs:79-98`) builds new `Agents` for each hosted call through `network.rs:161`, with a new TLS configuration, platform verifier and empty pool, so every call opens fresh TCP and TLS connections to every source host with no session resumption.
- Starts from: `cancellation.rs`, `network.rs`, `tls.rs` and the cancellation behavior from 0502.
- Rules: build the TLS configuration, including additive roots and the TLS session store, once per `Fetch` configuration and share it by `Arc` across per-call agents. Cancellation keeps shutting down only the call's own agents, and a cancelled call still sends nothing more. Proxy and extra-root settings behave as today. No reply changes.
- Proof: a test counts TLS configuration builds across ten hosted calls and finds one; a local TLS stub sees the second call resume the first call's session; the existing cancellation tests pass.
- Changes: the fetch layer's agent construction and its tests.
- Defers: one process-wide connection pool with each call's remaining time as a per-request timeout, after launch when traffic asks for it.
