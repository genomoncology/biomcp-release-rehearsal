# Nightly live check classifies by general cause

Status: COMPLETE.
Landed: 2a7205fc59bc9616f71b10f93a3a0641390eb4f6
Milestone: 5

## Outcome

The nightly live check fails only for a product defect or a source shape change. It classifies by general cause rather than by case: a throttled source (HTTP 429) and everything that follows from it in the same command is `throttled`; a source that gives no reply or a 5xx where the baseline had 200 is `outage`; a command whose baseline was partial or failed and that now completes is `recovered`. Each class stays listed in the report with its cases and sources. A source in `outage` on three nights in a row fails the job.

- Why: the scheduled check has failed every night from 2026-10-27 to 2026-11-03 (j16467 through j27732), and 0389, 0497, 0515 and 0577 each fixed only that night's cases. Run j27732 at c6fe4934c had 82 cases: 71 healthy, 7 churn and 4 failures, none of them a product defect. `article/edges/references.yaml` and `article/evidence/source.yaml` got Semantic Scholar 429s; their `request-plan-removed` and `page continuation changed` drift items are not in the allowed list in `classify_case`, so the throttle path never applies. `drug/native-chembl.yaml` had a ChEMBL request with no reply. `protein/native-sections.yaml` went from partial to complete, a source recovery. A check that fails every night trains people to ignore it.
- Starts from: a short root cause analysis (`skills/rca.md`) of the eight failed nights' reports on Yellow under `data/biomcp-redux/0068-scheduled/`, then `scripts/nightly_live_check.py` (`classify_case`, `command_classification`, `throttled_output`) and `scripts/test_nightly_live_check.py`.
- Rules: no product code changes; this is test infrastructure and does not touch command behaviour. Schema, invariant and unexplained plan changes still fail. Never call KEGG or the licensed LitSense subset. A new class needs a general rule and a test, never a case name.
- Proof: replaying the classifier over every stored report from 2026-10-27 to the latest gives zero failures that the RCA does not name as a product defect or shape change, and the tests show that a synthetic shape change, an unexplained plan change, and three straight nights of outage each still fail.
- Changes: `scripts/nightly_live_check.py`, its tests, and one short RCA record in `sdlc/records/`.
- Defers: nothing.
