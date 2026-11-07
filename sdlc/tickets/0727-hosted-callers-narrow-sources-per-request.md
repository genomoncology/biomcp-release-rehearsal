# 0727 — Hosted callers narrow sources per request

Status: COMPLETE.
Landed: d5416721c9c26582e1a8b2d2dcaddca8d311b609

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

From the ticket review, `records/2026-11-06-ticket-review.md`:

The seam exists: `Fetch::restrict_sources` only adds exclusions, so a request cannot widen what the deployment serves. Parse `x-biomcp-exclude-sources` beside the other per-request headers, carry it on the hosted call, and apply it where `hosted_use::restrict` applies the use limit. Feed the same list to `catalog::excluded`, so the tool list, help and `next` agree with the reply.

Names are source names only, not groups. The list of names users may leave out is CLI JSON in the style of `biomcp key list`.

Stored results: source snapshots already check disabled sources on read. Article pools need 0691's key, so this lands after 0691. Follow-up steps must not turn the header into a `--disable-sources` flag.

Proof adds a case where two callers with different lists run the same article search and neither receives the other's stored pool.

Ian confirmed on 2026-11-06: OncoKB and AlphaGenome stay off at `restricted`, even with a user's key.
