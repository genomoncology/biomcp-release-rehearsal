# 0498 — Keep enough nightly evidence to classify every live change

Status: COMPLETE.
Landed: a1dfb5a5738f80bbb72f776fca6888e3a38330c8

Milestone: 5

Depends on: 0497

## Outcome

The nightly live check keeps a complete witness for every changed case: no cut witnesses and no omitted drift details, and each derived request records the discovery IDs and input context it came from, including the DOI groups that Semantic Scholar edges pass to PubMed. A case is uncertain only when the source reply itself leaves the cause open, never because the check threw evidence away.

- Why: in scheduled j22745 all ten uncertain cases lacked diagnostics (cut witnesses or omitted drift details), and the no-contexts PubMed search changed through DOI terms that `RawEdges` did not record, so no cause could be proven (m15988). Uncertain fails the job, so the check stays red on evidence gaps alone, and 0485 will add every real-data case to it.
- Starts from: the live-check witness and drift output in the cases runner, `scripts/nightly_live_check.py`, the `RawEdges` provenance in the Semantic Scholar edge path and the j22745 report on Yellow.
- Rules: bound evidence size per case rather than cutting it; record provenance where requests are built, not by reparsing; keep the strict classifier rules; no new provider calls while building; no new framework.
- Proof: replaying the j22745 shapes yields no cut witness and no omitted drift; a classifier case shows the no-contexts DOI change traced to its discovery input; a planted case with a cut witness fails a check.
- Changes: the cases runner's live-check output, edge provenance, the nightly script and its tests.
- Defers: any new churn rule, which needs a PM ruling on the retained evidence.
