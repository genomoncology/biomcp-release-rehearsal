# FAERS search takes several drug names at once

Status: OPEN.
Milestone: 5

## Outcome

`search adverse-event --source faers` accepts a repeatable drug name and joins the names with OR in one openFDA query, so one call answers "reports for imatinib or nilotinib".

- Why: `planning/eval-gap-recheck-2026-11-04.md` item 4. `"imatinib OR nilotinib"` is sent as one quoted phrase and returns 0 rows, and two positional names exit 2, while openFDA returns 56,301 reports for the OR of the two generic names. Ian, 2026-11-04 (m24381): "All this stuff should get fixed in 1.0."
- Starts from: `crates/biomcp-types/src/adverse_event/` (the FAERS query) and the FAERS request builder in `crates/biomcp-adapters/src/faers/`.
- Rules: a repeated `--drug` (or repeated positional name, whichever the grammar already supports for other repeatables) joins with OR; a literal `OR` inside one quoted value is still one phrase, and help says to repeat the option instead. Capped at the openFDA query length, with a usage error naming the cap. Other filters AND with the name group. The request in the reply shows the query.
- Proof: installed-binary cases: two names give a total close to openFDA's OR count; one name is byte-identical to today; too many names give the cap error.
- Changes: `crates/biomcp-types/src/adverse_event/**`, `crates/biomcp-adapters/src/faers/**`, `crates/biomcp-query/src/adverse_event.rs`, `tests/cases/adverse-event/faers*/**`; the FAERS query and request, help, cases.
- Defers: nothing.
