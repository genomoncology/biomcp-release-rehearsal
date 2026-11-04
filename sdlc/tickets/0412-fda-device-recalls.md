# 0412 — Search FDA device recalls

Status: COMPLETE.
Landed: 430a08d342fc013bda9ace39e986b2def4c4945a

Milestone: 5

Depends on: 0413

## Outcome

`biomcp search diagnostic --source fda-device-recall` searches openFDA `device/recall.json` by recalling firm (phrase, reusing `--manufacturer`), exact `--product-code` and an initiation date range, and `get diagnostic` returns one recall by its typed `Z-NNNN-YYYY` product recall number. Each header gives the recall number, event number, firm, product code, status and initiation date; the card adds reason, root cause, product description, distribution pattern and only the top-level 510(k) and PMA numbers. `get` adds the classification from one exact enforcement lookup when one exists, and leaves it absent otherwise. A recall names the 0413 MAUDE search for its product code, from any manufacturer, as a next step. The retired adverse-event recall message points here and says what it covers.

- Why: `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. The 0162 recall drop is replaced.
- Starts from: openFDA `device/recall.json` and `device/enforcement.json` (`product_res_number` equals `recall_number`; enforcement held 1 of 3 Illumina recalls), `crates/biomcp-adapters/src/fda_510k`, `fda_pma`, `crates/biomcp-types/src/query/diagnostic/mod.rs` and `crates/biomcp/src/retired/adverse_event.rs`.
- Rules: follow `sdlc/planning/architecture.md` and the shared source rules in `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. NCI returns zero, not an error, for a wrong field name, so every NCI proof includes a positive case that a wrong build would fail. Never project `openfda` fields such as `openfda.k_number`, which describe the product code, not the recalled product. Counts are product recall records, not products harmed or events. Help says the source covers every medical device. Drug recalls are 0416, after Ian chose both on 2026-10-28.
- Proof: replay cases for firm, product code and date searches, a get with and without an enforcement match, paging, an `openfda` field never shown and a bad value with a problem; bounded captures.
- Changes: a new source, its typed ID, `crates/biomcp-types/src/query/diagnostic/mod.rs`, the retired recall message, schemas, help and cases.
- Defers: food recalls.
