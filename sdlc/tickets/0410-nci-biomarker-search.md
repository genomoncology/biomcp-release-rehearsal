# 0410 — Search NCI biomarkers

Status: COMPLETE.
Landed: c9a92e9720b9262ab7dd3a4355b4b08105fb6723

Milestone: 5

Depends on: 0408

## Outcome

`biomcp search biomarker --source nci` finds NCI trial biomarker concepts by name or synonym, and `biomcp get biomarker <NCIt code>` searches NCI by that code. Each header gives the NCIt code as a typed ID, name, native type and NCI's trial count; the card adds synonyms, parent and ancestor codes and the native `eligibility_criterion` values exactly as published. When one code returns several rows, `get` answers with candidates.

- Why: `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. MCP-Bench asks about biomarker criteria such as PD-L1; a 2026-10-28 check returned CD274 Gene, C96022, with PD-L1 among its synonyms.
- Starts from: 0408's NCI paging and refusal work, NCI CTS API v2 `/biomarkers` and `biodata::NcitId`.
- Rules: follow `sdlc/planning/architecture.md` and the shared source rules in `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. NCI returns zero, not an error, for a wrong field name, so every NCI proof includes a positive case that a wrong build would fail. The same snapshot paging, too-broad refusal and bucket-error mapping as 0408 apply. Capture the row shape first and record which documented fields the public tier returns; help states that coverage. Help separates three meanings of biomarker: this NCI concept, `search diagnostic --biomarker` and literal variant text in trial search.
- Proof: replay cases for a synonym search, a get by code, a get with several rows, paging, a missing key and a bad value with a problem; bounded keyed captures.
- Changes: a new biomarker entity, `crates/biomcp-adapters/src/nci`, schemas, help, the MCP command tree and cases.
- Defers: biomarker interpretation, which BioMCP never does.
