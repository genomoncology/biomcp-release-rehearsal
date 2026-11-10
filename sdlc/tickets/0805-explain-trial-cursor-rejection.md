# 0805 — Explain rejected trial continuations and offer a fresh restart

Status: COMPLETE.
Landed: 8fe91273cac6582b9bc14ab09caf48a5b8118cf1

Milestone: 6

## Outcome

When ClinicalTrials.gov rejects a paged request with HTTP 400, CLI and stdio MCP identify the continuation request and --next-page, with explicit fresh-first-page recovery preserving the search filters. The response does not misdescribe that provider rejection as unsupported command grammar or claim the cursor expired.

## Why

The correction supplied in chat on November 10 confirms fresh-token paging works but reports a misleading generic usage error for stale/fake tokens. Code inspection confirms the provider-400 conversion and recovery gap. The supplied report calls this nonblocking. Ian subsequently asked developers to work on these bugs during the qualification wait, so prepare implementation and focused proof now; retain existing candidate evidence under its actual source and assess affected checks before selecting a changed release candidate.

## Proof

- Use controlled HTTP-400 continuation responses through CLI and stdio MCP. Require specific fixed guidance naming --next-page and ClinicalTrials.gov's rejected request, without asserting expiry or echoing the token/provider body.
- Execute the emitted fresh restart: retain source, filters, configuration and source exclusions; clear both next_page and offset; bypass the cached first page. Follow the new successful continuation.
- Cover ordinary native cursors and broader-name wrapped cursors. Preserve successful trial continuation, native-consumption/refusal cases and unrelated usage/source errors.
- Run focused checks and routine gate. No live expired-token dependency or assumed registry TTL.

## Starts from

- crates/biomcp-types/src/query/trial/continuation.rs accepts ordinary opaque strings and validates only owned prefixes specially; ctgov/filters.rs forwards pageToken. crates/biomcp-query/src/trial.rs can convert HTTP 400 to TextPositionError; response/error.rs renders generic checked-command usage and command/reply.rs drops accumulated source metadata on that path. Broader-name notices currently take a different rejection path.
- Successful CTGov search replies retain cursors for configured one-day search freshness, but no registry cursor lifetime or exact reason for the reported rejection is established. Raw cache admits 2xx positives and 404 negatives, not HTTP 400. Fast rejection alone cannot prove local validation or a cached 400.

## Keeps

- Existing caller isolation, filters, exclusions, budgets, cache admission and freshness policy. No automatic retry, silent fresh fetch, inferred cursor TTL, expiry classification, provider-body disclosure or arbitrary command rewriting. Use explicit actionable recovery through existing typed guidance/next-step structures.
- Preserve current qualification source identities. The broad trial grammar allegation was disproved by fresh native Mac/Linux CLI/MCP follows; this ticket addresses the confirmed narrower diagnostic path. The coordinator decides when accepted fixes enter an ending candidate and repeats only affected qualification.

## Defers

- Registry cursor lifetime research or cache policy changes need separate evidence and scope if later warranted. Implementation can proceed during qualification; publication remains separately approved.

## Changes

- Trial rejection/recovery guidance and owning typed error/next-step definitions as necessary, with controlled CLI/MCP regression fixtures. No general parser or transport redesign.
