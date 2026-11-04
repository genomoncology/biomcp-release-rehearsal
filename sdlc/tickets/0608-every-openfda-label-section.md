# Drug labels serve every openFDA section

Status: OPEN.
Milestone: 5

## Outcome

`get drug ... label --label-section` accepts every section field the openFDA label record holds, such as `adverse_reactions`, and its error lists the sections that record has.

- Why: `planning/eval-gap-recheck-2026-11-04.md` item 2. `--label-section adverse_reactions` exits 2 because only five sections are allowed, while openFDA holds 32 fields for the witness label.
- Starts from: the label section list in `crates/biomcp-types/src/drug/query.rs` and `crates/biomcp-query/src/drug/openfda_label.rs`.
- Rules: the accepted sections are openFDA's published SPL section fields, generated from one list, never hand-picked. A section the record lacks gives an empty section, not an error. The text is the source's, unchanged.
- Proof: installed-binary cases: `adverse_reactions` and one table section answer on the witness label; a missing section is empty; the five existing sections are byte-identical.
- Changes: `crates/biomcp-types/src/drug/query.rs`, `crates/biomcp-query/src/drug/openfda_label.rs`, `crates/biomcp-query/src/drug/label_bound.rs`, `crates/biomcp-adapters/src/openfda_label/**`, `tests/cases/drug/**`; the section list, the label section, help and cases.
- Defers: nothing.
