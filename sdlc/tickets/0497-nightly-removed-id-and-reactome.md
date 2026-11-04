# 0497 — Refresh the Reactome recording and explain the nightly PubMed and uncertain cases

Status: COMPLETE.
Landed: 81202adc5cd812d273cae85b748eb2ca81be524c

Milestone: 5

## Outcome

The pathways-tp53 recording matches Reactome's current reply, which no longer carries `releaseStatus`, so the nightly check stops failing on it. The two PubMed cases that failed in j22745 and the ten it marked uncertain each get a recorded cause, and the classifier changes only where a cause is proven safe to class as churn.

- Why: scheduled j22745 at dee9172a9 failed on `gene/sections/pathways-tp53.yaml` (`/*/releaseStatus` removed, failing nightly since j21096), `article/edges/no-contexts.yaml` and `article/fold/live-reverse.yaml`, and marked ten cases uncertain; uncertain also fails the job. My first reading called the PubMed changes a single mid-list removal. The developer's check of the retained report showed more: the no-contexts search request identity itself changed, a second PMID left, and live-reverse reordered many IDs (m15987).
- Starts from: `scripts/nightly_live_check.py`, `scripts/test_nightly_live_check.py`, the j22745 report on Yellow, and the pathways-tp53 recording.
- Rules: refresh the Reactome recording from one bounded public capture; keep the strict every-other-ID-in-order rule; for each PubMed and uncertain case, record from the retained report why it changed (for example a date-dependent request parameter or relevance reranking) before proposing any rule; a new churn rule needs a PM ruling and must keep a changed non-ID request parameter a failure; no new framework; nothing is hidden from the report.
- Proof: the replay suite passes on the refreshed Reactome recording; the next scheduled run reports no failure for pathways-tp53 unless the source fails; each investigated case has its cause in progress; any approved rule has a classifier case built from the j22745 shapes that fails on the current script.
- Changes: the Reactome recording, progress notes, and the classifier and its tests only if a rule is approved.
- Defers: nothing.
