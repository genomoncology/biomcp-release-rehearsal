# 0651 — Find FDA-labeled drugs by disease or indication

Status: COMPLETE.
Landed: 18bc84b60ac68dfc1b04a4cc9954e7fbfa75023c

Milestone: 5

## Outcome

`search drug --indication melanoma --source openfda-label` returns the labels whose Indications and Usage section names melanoma, with paging and the source's native total. Each row gives the drug identity that `get drug` accepts, so an agent can go from a disease to FDA-labeled drugs to their label sections.

- Why: Ian, testing hosted BioMCP on 2026-11-05, could not find FDA-approved drugs for a disease (hosting m26471). 0.9 had `--indication`. 1.0 refuses it as not yet supported (`ErrorMessage::DrugUnproved`), and `get disease` has no drugs section.
- Starts from: `crates/biomcp/src/retired/drug.rs` (the refusal table), `crates/biomcp-types/src/drug/query.rs`, the openFDA label search in `crates/biomcp-query/src/drug/`, and `docs/cautions.md` on label search order. openFDA's label endpoint supports field search on `indications_and_usage`.
- Rules: openFDA label only. Drugs@FDA has no indication field, and Open Targets known drugs by disease is a separate later ticket if this one proves out. Indication text is the label's own wording; BioMCP never infers an indication. The flag keeps the CLI, single-tool and typed-tool replies identical, as every command does. Fix general causes; tune nothing to the melanoma example.
- Proof: replayed openFDA cases for `--indication` with a disease word, a two-word phrase and a no-match term; an installed case shows the refusal is gone and paging keeps the query; routine gate.
- Changes: `crates/biomcp/src/retired/drug.rs`, `crates/biomcp-types/src/drug/`, `crates/biomcp-query/src/drug/`, `tests/`, `docs/`; the flag, its query and cases.
- Defers: disease-to-drug through Open Targets, and approvals by indication, until this ships and hosting reports what users ask next.
