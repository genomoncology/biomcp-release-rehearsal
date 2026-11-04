# MAUDE search filters by received date

Status: COMPLETE.
Landed: cd2ae3bc1ac6e3e36cb8a5cae18db65a783cb5da
Milestone: 5

## Outcome

`search adverse-event --source maude` accepts `--received-from` and `--received-to` as it does for FAERS, at day or month precision, and sends them as an openFDA `date_received` range.

- Why: `planning/eval-gap-recheck-2026-11-04.md` item 4. The MAUDE witness for insulin pumps in January 2020 exits 2, while openFDA returns 886 reports for the same range. Help promises month precision, such as `2020-01`, and it is refused.
- Starts from: `crates/biomcp-types/src/adverse_event/query.rs`, the MAUDE request builder in `crates/biomcp-adapters/src`, and the FAERS date filter as the model.
- Rules: the same date type and precision rules for both sources; a month means its first to last day. The request shown in the reply carries the range.
- Proof: installed-binary cases: the witness answers with a total in line with openFDA's; month and day precision give the same request for a whole month; FAERS cases are unchanged.
- Changes: `crates/biomcp-types/src/adverse_event/**`, `crates/biomcp-adapters/src/maude/**`, `crates/biomcp-query/src/adverse_event_maude.rs`, `crates/biomcp-query/src/adverse_event_maude/**`, `tests/cases/adverse-event/**`, `tests/replies/adverse-event/**`; the adverse-event query, the MAUDE request, help and cases.
- Defers: nothing.
