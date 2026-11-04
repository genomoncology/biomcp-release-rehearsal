# 0416 — Search FDA drug recalls

Status: COMPLETE.
Landed: c84507ec63a9b7ceaa5a1ed160b1d159c775d908

Milestone: 5

Depends on: 0412

## Outcome

`biomcp search drug --source fda-drug-recall` searches openFDA `drug/enforcement.json` by drug name (phrase on product description and the published brand and generic names), recalling firm, classification, status, a phrase in the published reason for recall and a recall initiation date range, and a recall can be fetched by its native recall number. Each header gives the recall number, classification, status, firm, product description, initiation date and report date; the card adds reason, distribution pattern, code information and voluntary or mandated status as published.

- Why: `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. Ian chose both device and drug recalls (2026-10-28). 0.4.6 offered `openfda_recall_searcher` for drugs and 0.9 searched `drug/enforcement.json` through `--type recall`, so this restores a dropped capability.
- Starts from: 0412's recall identity, typed ID and paging work, the drug entity's source pattern, openFDA `drug/enforcement.json` and the 0.9 implementation of `--type recall` for the fields it used.
- Rules: follow `sdlc/planning/architecture.md` and the shared source rules in `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. Classification is the FDA's own Class I, II or III label, never re-ranked. Counts are recall records, not patients harmed. The reason search and the initiation date range restore what 0.4.6 offered, where the reason searched `reason_for_recall` and the since date mapped to `recall_initiation_date`; the date range uses the same flag spelling as 0412. Use `openfda` harmonized fields only for matching, and show a drug name only from fields the recall record itself publishes.
- Proof: replay cases for a drug name, a firm, a classification, a reason phrase, an initiation date range, a fetch by recall number, paging and a bad value with a problem; bounded captures.
- Changes: a new drug source, its typed ID, schemas, help, the upgrade table and cases.
- Defers: food recalls.
