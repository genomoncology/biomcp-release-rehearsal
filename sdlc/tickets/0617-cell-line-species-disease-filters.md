# Cell-line search filters by species and disease

Status: OPEN.
Milestone: 5

## Outcome

`search cell-line` accepts `--species` and `--disease`, matched against Cellosaurus's own species and disease fields, so "human melanoma lines" excludes mouse B16.

- Why: `planning/eval-gap-recheck-2026-11-04.md` item 7: free text works, but `search cell-line melanoma` returns mouse B16 among human lines and there is no species or disease filter. Ian, 2026-11-04 (m24381): "All this stuff should get fixed in 1.0."
- Starts from: `crates/biomcp-query/src/cell_line/`, `crates/biomcp-adapters/src/cellosaurus/`, and `crates/biomcp-types/src/cell_line/`.
- Rules: filters use Cellosaurus fields (species by NCBI Taxonomy name or ID, disease by NCIt name or ID) in the source query, not after fetching. Text is optional when a filter is given.
- Proof: installed-binary cases: human melanoma excludes B16; a disease ID filter matches its name filter; free text alone is byte-identical.
- Changes: `crates/biomcp-query/src/cell_line/**`, `crates/biomcp-adapters/src/cellosaurus/**`, `crates/biomcp-types/src/cell_line/**`, `tests/cases/cell-line/**`, `tests/cell-line-command-budgets.yaml`; cell-line search, help, cases.
- Defers: nothing.
