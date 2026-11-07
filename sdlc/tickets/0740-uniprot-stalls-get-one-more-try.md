# 0740 — UniProt stalls get one more try

Status: OPEN.

Milestone: 5

## Outcome

`biomcp get protein P00533` succeeds on nearly every live try, despite UniProt's stalls. A stalled UniProt read gets one more try inside the same 15 seconds, if measurement shows that helps. If it does not help, the ticket closes with the measurement and no change.

## Why

0736 cut the transfer to 50 KB, and raising the limit to 15 seconds still left 3 of 12 live `get protein P00533` tries failing. Plain `curl` with compression passed 15 seconds on 3 of 10 fetches, up to 23 seconds, so a longer limit cannot fix it. UniProt starts its reply in about 0.4 seconds and then stalls while sending the body, and a normal fetch takes under a second. EGFR is a common question, so one in four failing is too many.

## Starts from

- Starts from: a live measurement of whether a second request, started after a short stall, returns when the first is stuck. Then the UniProt adapter and `crates/biomcp-fetch/`.

## Keeps

- Keeps: UniProt's rate limits, the 20-second command limit, the honest failure reply and next steps from 0736, and recorded cases. No store format, cache namespace or BioData pin change.

## Changes

- Changes: `crates/biomcp-adapters/src/uniprot/`, and `crates/biomcp-fetch/` only if the retry belongs there.

## Proof

- Proof: 12 or more live `get protein P00533` tries through the built CLI, with their times, against the same run without the change. A test with a stalled recorded reply shows exactly one extra request.

## Defers

- Defers: nothing.

## Depends on

- Depends on: 0736.
