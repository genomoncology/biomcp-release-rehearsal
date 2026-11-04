# Nightly counts a throttled replanned request as throttled

Status: OPEN.
Milestone: 5

## Outcome

When the request that got HTTP 429 is itself new in tonight's plan, and the only other changes in that command are dependent requests that were dropped because of it, the nightly live check classifies the case as `throttled`, as 0583 promised, not as `uncertain`.

- Why: nightly j28820 at d349f2489 (2026-11-04) failed on `article/recommendations/single.yaml`. Semantic Scholar answered 429 to request 3635cb8d…, which was `request-plan-added`. Its two dependent requests were `request-plan-removed`, and the output went from complete to failed. 0583's rule covers a 429 "and everything that follows from it in the same command", but the classifier only applies it when the throttled request was also in the baseline. The same run classified three other throttled cases and one recovered case correctly. The only other failure, `article/preprint/all.yaml`, was a one-night Europe PMC full-text reply missing every JATS field. The same URL returns the full 91 KB JATS again, so it stays a failure and is watched rather than ticketed.
- Starts from: `command_classification`, `dependent_change` and `throttled_output` in `scripts/nightly_live_check.py`, its tests, and the j28820 report on Yellow at `data/biomcp-redux/0068-scheduled/20261104T090107Z-6jtslq_f`.
- Rules: a general rule with a test, never a case name. The throttled request may be new only when its provenance shows it came from the same discovery as the baseline request it replaced. Every removed request must depend on it. Anything else stays `uncertain` or `failure`.
- Proof: replaying the classifier over every stored report since 2026-10-27 changes only this case's class. A synthetic case where an unrelated request is also removed stays `uncertain`.
- Changes: `scripts/nightly_live_check.py` and its tests.
- Defers: nothing.
