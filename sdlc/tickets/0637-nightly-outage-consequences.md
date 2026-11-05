# Nightly outage covers the requests a failed source drops

Status: COMPLETE.
Landed: 78d1ee710b4207970edc803ff69b897874e59c34
Milestone: 5

## Outcome

The nightly live check classifies a case as `outage` when a source's 5xx or missing reply in one command drops or changes the requests that depend on it, using the same consequence rules `throttled` already uses for a 429 (0583, 0597), instead of failing the job.

- Why: nightly j30151 at f9dc51fd7 (2026-11-05) failed with three `failure` cases, all following a source 5xx: `article/filters/include-retracted.yaml` and `article/filters/since.yaml` (PubMed 200 to 500, then `request-plan-removed` for the dependent request in each command) and `article/recommendations/single.yaml` (a 200 to 500, status complete to partial, then `request-plan-changed`). Two other PubMed and ClinVar 500 cases were classified `outage` correctly because they dropped nothing. 78 cases were healthy, 3 churn, 1 recovered; no product defect. The throttle path already treats dropped dependents as consequences of the 429; the outage path does not.
- Starts from: `classify_case` and `command_classification` in `scripts/nightly_live_check.py`, the throttle consequence rules from 0583 and 0597, its tests, and the j30151 report on Yellow at `data/biomcp-redux/0068-scheduled/20261105T090610Z-wvyook8m`.
- Rules: one shared consequence rule for a failed root request, whether 429 or 5xx/no reply: removed or changed requests count as its consequences only when they depend on that root by existing discovery provenance, or follow from it by the 0597 cache-order rule. Anything else stays `uncertain` or `failure`. The three-nights-in-a-row outage failure stays. A general rule with a test, never a case name. Scripts only; no product change.
- Proof: replaying the classifier over every stored report since 2026-10-27 changes only cases whose drift follows a failed root: the three j30151 failures become `outage`, and older `outage` cases whose dropped requests cannot be linked to the failed root by provenance become `uncertain` or `failure` (2026-10-29 preprint and live-reverse cases, 2026-10-31 litsense licensed subset; m25914); synthetic cases with an unrelated dropped request, a shape change, or an unexplained plan change still fail.
- Changes: `scripts/nightly_live_check.py`, `scripts/test_nightly_live_check.py`; the shared consequence rule and tests.
- Defers: nothing.
