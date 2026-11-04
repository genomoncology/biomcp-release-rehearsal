# openFDA label search fills its page and names every row

Status: OPEN.
Milestone: 5

## Outcome

`search drug --source openfda-label` returns as many rows as its limit allows, up to the total, and every row has a name; help describes the query as the openFDA label search it is.

- Why: `planning/eval-gap-recheck-2026-11-04.md` item 2. `biomcp search drug imatinib --source openfda-label --limit 3` returns 0 of 18 (1 of 18 at limit 10), so rows are being dropped after the page is fetched. In an indication search, 4 of 10 rows had no name. Help calls the query an exact name phrase, but it also works as an indication search.
- Starts from: `crates/biomcp-query/src/drug/openfda_label.rs` and the openFDA label adapter in `crates/biomcp-adapters/src`.
- Rules: find the general cause of the dropped rows (likely filtering or deduplication after a fixed-size fetch) and fix it there: either fetch until the page fills or stop filtering. A row with no brand or generic name takes its name from the label's own fields in a stated order, and is never dropped for lacking one. Help text says what the query matches.
- Proof: installed-binary cases: the witness at limits 3 and 10 return 3 and 10 rows with consistent totals; the indication search has a name on every row; paging covers the total with no repeats.
- Changes: `crates/biomcp-query/src/drug/openfda_label.rs`, `crates/biomcp-adapters/src/openfda_label/**`, `tests/cases/drug/openfda-label*/**`; the label search, help text and cases.
- Defers: nothing.
