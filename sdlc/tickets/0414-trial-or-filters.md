# 0414 — Join repeated trial conditions, interventions and terms with OR

Status: COMPLETE.
Landed: 953f2f943877dd15f167ee63967160cb26545d91

Milestone: 5

## Outcome

On ClinicalTrials.gov, `search trial` accepts `--condition`, `--intervention` and `--keyword` more than once, and joins repeated values of one option with OR; different options still combine with AND. `--keyword` maps to ClinicalTrials.gov's general term query, which it refuses today. On NCI, repeated `--condition` NCIt codes become repeated disease concept parameters, which NCI documents as OR; NCI still refuses `--intervention`. The query echo lists every value.

- Why: `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. 0.4.6 accepted lists for conditions, interventions and other terms, and a benchmark task passes two terms.
- Starts from: `crates/biomcp-types/src/query/trial.rs`, `crates/biomcp-adapters/src/ctgov/mod.rs` and `raw.rs`, and `crates/biomcp-adapters/src/nci`.
- Rules: follow `sdlc/planning/architecture.md` and the shared source rules in `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. NCI returns zero, not an error, for a wrong field name, so every NCI proof includes a positive case that a wrong build would fail. Quoting (PM ruling, the adversarial design review of 2026-10-28): one value and many values follow one rule. Each value is sent as it is sent today and wrapped in parentheses, then joined with OR, so a multi-value search never loses hits that each single value finds. The condition-word mismatch notice checks each value on its own. `--exact-condition` with several values builds one `AREA[Condition]` clause per value, joined with OR. At most 5 values per option (PM number). Changing these fields to lists takes a major bump of the trial search and request schemas. `--condition-id` stays single.
- Proof: for a multi-word value, the OR total is at least each single-value total; cases for two conditions, two interventions, two keywords, the mismatch notice per value, exact condition with two values, the cap and NCI's intervention refusal; bounded captures.
- Changes: the files above, schemas, help and cases.
- Defers: OR across different options.
