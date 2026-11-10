# 0798 — Retain diagnostics for each live check attempt

Status: OPEN.

Milestone: 5

## Outcome

A failed nightly command leaves bounded structured diagnostics for both attempts, sufficient to distinguish the CLI's source status, HTTP/TLS details when available, local error and timeout. Later success never replaces the failed attempt's evidence.

## Why

The November 10 nightly lost the original replies and first-attempt details for three failed commands; a later successful probe could not explain them. Evidence: `records/2026-11-10-product-review.md`, F8.

## Starts from

- `scripts/nightly_live_check.py` (`reason`, `check`, retry loop) and `scripts/test_nightly_live_check.py`. The script currently reduces a complete reply to a 200-character reason, and on two failures reports only the second reason. The existing PM job log is the durable sink.

- PM decision, 2026-11-10: emit and flush each bounded structured attempt record in the existing log before retrying and before the final summary, using the CLI envelope's selected diagnostic fields and process result. Keep actual first/second outcomes, elapsed time and attempt number. Do not claim HTTP details absent from the CLI reply; HTTP status is generally absent today. Retain fixed TLS guidance when present. Select fixed diagnostic fields, not arbitrary notices or malformed stdout, and mark truncation while preserving valid JSON. Avoid full biomedical bodies, credentials, environment dumps, raw headers and a new reporting framework.

- Approved implementation plan: each serialized attempt record has an 8 KiB UTF-8 ceiling; truncation retains valid JSON and an explicit marker.

## Proof

- Existing Python harness with synthetic first-fail/second-success, different failures across both attempts, malformed JSON and subprocess timeout. Assert each original diagnostic survives, output bounds hold, key-isolated environment stays intact and final exit/summary semantics are unchanged. Include a synthetic body/credential sentinel proving omitted payload fields never enter the attempt records. No real provider calls are needed.

## Keeps

- Current one-retry policy, public-keyless source set, pacing, result checks and the distinction between a failed provider check and a proven provider outage or product defect.

## Changes

- Changes: `scripts/nightly_live_check.py`, `scripts/test_nightly_live_check.py`.

## Defers

- Diagnosing the original transient failures remains part of final source qualification; do not invent their cause or reclassify the old failed job.
