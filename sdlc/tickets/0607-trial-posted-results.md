# Trials show their posted results

Status: OPEN.
Milestone: 5

## Outcome

`get trial` has a `results` section built from ClinicalTrials.gov's posted results (outcome measures with their groups, values, dispersion and analyses, and the adverse-event tables), and the trial card shows the results-first-posted date.

- Why: `planning/eval-gap-recheck-2026-11-04.md` item 1, the evals team's highest-value gap. `get trial NCT01844505 --section outcomes` lists only registered measures, with no values, no adverse events and no results date, while the v2 API returns the PFS medians with confidence intervals and an adverse-event module of 393 serious terms.
- Starts from: the ClinicalTrials.gov v2 adapter and trial sections under `crates/biomcp-adapters/src` and `crates/biomcp-query/src/trial`, and the `resultsSection` and `resultsFirstPostDate` fields of the same study record BioMCP already fetches.
- Rules: no new source or request; the data comes from the study record already fetched. Values keep their reported units, parameter type and dispersion, never recomputed. Large tables page with `--section-limit` and `--section-offset` like other sections. A trial without results says so in the section. The new section and card field are the allowed exception to the command freeze; other replies stay byte-identical, and the schema change is named in the release notes.
- Proof: installed-binary cases on NCT01844505 show the PFS medians and confidence intervals, the adverse-event groups and a page of serious terms, and the posted date; a trial without results shows an empty section; existing trial cases are unchanged apart from the card date.
- Changes: `crates/biomcp-types/src/query/trial/**`, `crates/biomcp-types/src/query/trial_get.rs`, `crates/biomcp-types/src/response/trial*`, `crates/biomcp-adapters/src/ctgov/**`, `crates/biomcp-query/src/trial.rs`, `crates/biomcp-query/src/trial/**`, `crates/biomcp-types/schemas/trial*.json`, `tests/cases/trial/**`, `tests/replies/trial/**`; the trial types and schema, the adapter mapping, the section, help, the map, and cases.
- Defers: nothing.
