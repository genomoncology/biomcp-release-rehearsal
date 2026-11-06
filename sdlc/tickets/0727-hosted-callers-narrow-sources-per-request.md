# 0727 — Hosted callers narrow sources per request

Status: OPEN.

Milestone: 5

## Outcome

The hosted core accepts a per-request list of sources to leave out, set by the wrapper from a user's settings. It can only narrow what the deployment serves, never widen past `hosted_max_use`. Keyed sources keep using the key the wrapper sends for that user.

## Why

Ian, 2026-11-06: "We have to get the hosting server to support API keys and potentially allow people to accept specific sources over others: a custom sources configuration by user." Hosting owns the user settings and key storage; this is the core seam they need. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then an HTTP case where a request leaves out a source and that source makes no call and is reported off, and a case where it tries to add a source and cannot. Agree the header with hosting before building. Cached and pooled results obey the same list.

## Starts from

- Starts from: `crates/biomcp/src/mcp/http/wire.rs`, `crates/biomcp/src/command/hosted_use.rs`, `decisions/2026-11-04-hosted-source-modes.md`.

## Keeps

- Keeps: the three-use ruling, and no key stored or logged by the core.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp/src/mcp/`, `docs/configuration.md`, `tests/`.

## Amendment, 2026-11-06

Hosting agreed the contract (m28791). The header is `x-biomcp-exclude-sources`, comma-separated source names. Hosting's wrapper sets it from account settings and strips client copies. Core also rejects an unknown source name clearly, and gives a machine-readable list of source names users may leave out, in the style of `biomcp key list`.
