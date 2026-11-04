# Nightly counts cache-order effects of a throttled command as throttled

Status: COMPLETE.
Landed: 2e6d77240346ff0d306d4d4f225363dde631b543
Milestone: 5

## Outcome

When a 429 in one command of a case means a later command in the same case must fetch what it would otherwise have read from the cache, the nightly live check classifies the case as `throttled`, as 0583 promised, not as `uncertain`. This applies when the later command had no requests in the baseline, its added root request is on the throttled source, any added request to another source depends on that root by discovery provenance, and every added request succeeds.

- Why: nightly j28820 at d349f2489 (2026-11-04) left `article/recommendations/single.yaml` uncertain. Command 1's Semantic Scholar request 3635cb8d… went from 200 to 429 and dropped two dependents. That part already classifies as throttled. Command 2 had no requests in the baseline, because it read command 1's cached replies, and tonight it made successful live requests, because nothing was cached. That cross-command effect is what makes the case uncertain (coordinator diagnosis m24045, correcting the PM's first reading). The same run classified three other throttled cases and one recovered case correctly. The only other failure, `article/preprint/all.yaml`, was a one-night Europe PMC full-text reply missing every JATS field. The same URL returns the full 91 KB JATS again, so it stays a failure and is watched rather than ticketed.
- Starts from: `classify_case` and `command_classification` in `scripts/nightly_live_check.py`, its tests, and the j28820 report on Yellow at `data/biomcp-redux/0068-scheduled/20261104T090107Z-6jtslq_f`.
- Rules: a general rule with a test, never a case name. A later command counts as a cache-order effect only when an earlier command in the same case got a 429, the later command had no baseline requests, its added root is on the throttled source, every added request to another source depends on that root by existing discovery provenance, and every added request succeeded (m24045). Anything else stays `uncertain` or `failure`.
- Proof: replaying the classifier over every stored report since 2026-10-27 changes only this case's class. Synthetic cases where the later command has a failed request, a different source, or baseline requests stay `uncertain`.
- Changes: `scripts/nightly_live_check.py` and its tests.
- Defers: nothing.
