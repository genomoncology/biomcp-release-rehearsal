# 0687 — Ignore ledger binds to landed main

Status: OPEN.

Milestone: 5

## Outcome

Only the four commit fields in `release/secrets.ignore.jsonl` rows 178–181 bind to published trial main `6c9e62b770ac237902c9073de6e61c2f0a78eeae`, after exact path, blob and line verification. PM authorized this change in m28084.

## Why

Later landing jobs cannot reach the completed trial ticket's original source commit, so their unchanged history scan refuses these rows.

## Proof

- Proof: Native job j31795 fails with `history-ignore-unreachable-commit` at row 178. Independently verify each target commit/path/blob/line and that only the four commit fields change; the next combined native gate must pass the unchanged history scanner.

## Starts from

- Starts from: `release/secrets.ignore.jsonl` rows 178–181, the existing validator in `release/secrets.py`, and published main `6c9e62b770ac237902c9073de6e61c2f0a78eeae`. Its four named files have the recorded blobs. The original source object exists but is outside the next job's named history scope.

## Keeps

- Keeps: Every scanner constraint, path, blob, line, pattern, rationale and other ledger row. No scanner change, extra job refs, future commit binding or other ledger edit. Fresh review is required by PM m28084.

## Defers

- Defers: Nothing; this bounded input repair unblocks the pending landing batch.

## Changes

- Changes: `release/secrets.ignore.jsonl` only.
