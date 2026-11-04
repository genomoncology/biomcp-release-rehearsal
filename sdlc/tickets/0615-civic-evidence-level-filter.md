# CIViC drug evidence filters by level and status

Status: OPEN.
Milestone: 5

## Outcome

`get drug <id> civic --source civic --civic-kind evidence` accepts `--evidence-level` (A to E, repeatable) and `--evidence-status` (accepted, submitted, rejected), sent to CIViC's own query.

- Why: `planning/eval-gap-recheck-2026-11-04.md` item 8. The witness returns 25 of 169 imatinib evidence items with no filter; CIViC GraphQL returns 7 accepted level-A items for therapy 5. Ian, 2026-11-04 (m24381): "All this stuff should get fixed in 1.0."
- Starts from: `crates/biomcp-query/src/drug/civic.rs`, `crates/biomcp-adapters/src/civic/`, and `civic_kind` in `crates/biomcp-types/src/drug/query.rs`.
- Rules: filters go to CIViC's GraphQL arguments, never filtered after fetching; totals reflect the filter. The same filters apply wherever else BioMCP reads CIViC evidence, through one shared filter type.
- Proof: installed-binary cases: accepted level A for therapy 5 gives 7; no filter is byte-identical.
- Changes: `crates/biomcp-adapters/src/civic/**`, `crates/biomcp-query/src/drug/civic.rs`, `crates/biomcp-query/src/variant/civic.rs`, `crates/biomcp-types/src/drug/query.rs`, `tests/cases/drug/civic*/**`; cIViC filters, help, cases.
- Defers: nothing.
