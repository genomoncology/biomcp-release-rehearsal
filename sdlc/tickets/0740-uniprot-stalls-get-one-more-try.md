# 0740 — UniProt stalls get one more try

Status: COMPLETE.
Landed: 7cbde6ad6a13277b42b724235596a57be4605af8

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
- 2026-11-07 next: Allowance stop. Branch ticket/0740-uniprot-stalls-get-one-more-try at 3f279bf68; j34537 passed. Next: 12 live tries each way, keeping the first request open; close with the measurement if no gain. Remove the other 0740 branch on origin before landing; land refuses two.

## Evidence

- Evidence: j34244 cut the first request at 3 seconds and sent a second: 10 of 12 live tries passed against 9 of 12 without the change, within noise. j34142 had shown a second curl returning while the first stayed open.
- Evidence: j34537 keeps the first request open while the second runs, first finished reply wins: 12 of 12 live `get protein P00533` tries passed, slowest 13.42 seconds, against 8 of 12 on main in the same interleaved run, where all four failures were 15-second deadlines. Twelve tries each way is a small sample, but the gain is in the direction the ticket asked for and the kept-open design replaces the cut-first one.

## Amendment, 2026-11-07

Review note: a losing request may keep its connection open after the winner returns, for at most the time left in the 15-second budget, and its quota slot is already released. On a long-lived server the number of requests in flight to UniProt can briefly exceed its quota of two. The one-shot CLI is unaffected. Accepted for 1.0 as bounded.
