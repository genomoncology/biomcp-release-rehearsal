# 0485 — Put every real-data case in the nightly live check

Status: COMPLETE.
Landed: 18186021724d467bf7357356e3af0d5af72dfa48

Milestone: 5

Depends on: 0456, 0497, 0498

## Outcome

Every kept case that uses real recorded replies runs in the nightly live check, so a saved reply that no longer matches its source is caught within a day. Only synthetic failure cases and local-only cases stay out, and each says why.

- Why: Ian's ruling in `decisions/2026-10-29-one-small-outside-in-suite.md`. Today 98 of 680 cases are live-checked, and 182 cases never had their source provenance reviewed.
- Starts from: the `live_check` field in each case, `scripts/nightly_live_check.py` and `runner/live*.rs`.
- Rules: a case with real recorded replies is live-checked unless it needs a key the nightly job lacks or it is local-only; either exclusion is stated in the case. Review provenance for each kept real-data case before marking it public. The nightly job stays within its existing source limits.
- Proof: the nightly report's checked count equals the number of eligible kept cases. A planted real-data case with no `live_check` field fails discovery.
- Changes: case `live_check` fields and the selection check.
- Defers: nothing.
