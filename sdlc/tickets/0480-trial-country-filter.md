# 0480 — Filter trial search by country

Status: COMPLETE.
Landed: ea95d704bfccb3f2633964ff3ad372eaa57d77d3

Milestone: 5

## Outcome

`search trial --country` limits results to trials with a site in the named country, so a question about US trials no longer returns every country's trials.

- Why: BioData's experiment 502 compared BioMCP 0.9.3 with an agent writing its own Python and curl on dated cancer questions, and the scripts won (m15051). A code triage against 1.0 at 8466b4700 found this defect still present with no ticket.
- Starts from: `search trial` options in `crates/biomcp-types/src/query/trial.rs`, the ClinicalTrials.gov adapter in `crates/biomcp-adapters/src/ctgov`, and `docs/reference/commands.md`.
- Rules: map to ClinicalTrials.gov's location country field; accept the country names ClinicalTrials.gov publishes and refuse anything else with a named error; repeated values combine as OR like other repeated trial options; the reply echoes the filter; NCI's own country filter is used where the NCI source takes one, and a source that cannot filter by country refuses the option rather than ignoring it.
- Proof: a case over a recorded reply with `--country "United States"` asserts the request carries the country and every returned trial has a US site, and fails on the base build; an unknown country is refused.
- Changes: the trial query type, the ctgov adapter, help, cases and baseline.
- Defers: nothing.
