# 0535 — Cut wasted jobs, gate time and leftover temp files

Status: COMPLETE.
Landed: 7c096652175abe0cc8918d097ff9798fe65a5874

Milestone: 5

## Outcome

Cheap checks run before any full job, proofs run through make targets, the gate prints its time per stage and scans only new commits for secrets on landing, and closed tickets leave no temp folders or readable job request copies behind.

- Why: release review C9, gate time and B8, weighed 2026-11-01. Of the last 800 pm jobs, 445 failed (56 percent). Of 165 recent failures, 87 were lint, compile, test or ratchet failures a cheap local step catches, and about 72 were proof machinery: inline Python proof scripts (21), reply-diff noise (15), Python 3.9 `extractall(filter=)` on the M5 (4) and native-proof logs with no error line (about 32). Gate j24209 took 575 seconds, much of it a whole-history secret scan and a forced coverage rebuild. The Beelink's `/tmp` held 1,365 BioMCP entries from 2026-09-03 to 2026-10-31, 613 of them `biomcp-test-*` folders from builds on a machine that never builds BioMCP, and 31 job request copies, 9 readable by any local user (made private on 2026-11-01).
- Starts from: `skills/rca.md` and `skills/proof-spiral.md` in the agents repo, the Makefile, `scripts/coverage.sh`, the secret scan target, the proof scripts that copy request files, and the Beelink `/tmp` listing.
- Rules: run the root cause analysis first and record it under `sdlc/records/`. Add a cheap pre-job step (fmt, clippy, ratchets). Proofs that recur become make targets. Pin or fix the M5 proof Python. Landing scans commits since the last scanned main; the release workflow keeps its full-history scan. Delete Beelink temp leftovers of closed tickets only after checking that no open ticket cites them; ask the 0405 and 0293 owners about theirs. No script copies a job request file, or it writes mode 600.
- Proof: the gate log shows per-stage times and a shorter total; a lint error fails the cheap step in seconds; no Beelink `/tmp` entry of a closed ticket remains; `rg` finds no request-file copy without mode 600.
- Changes: the Makefile, gate scripts, proof scripts, a record, and temp cleanup.
- Defers: tuning the coverage rebuild until the per-stage times exist.
