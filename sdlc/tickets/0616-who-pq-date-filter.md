# WHO prequalification search filters by date, with the name optional

Status: OPEN.
Milestone: 5

## Outcome

`search drug --source who-pq` accepts `--prequalified-from` and `--prequalified-to` at day, month or year precision, and a search with only a date range lists every product prequalified in that range.

- Why: `planning/eval-gap-recheck-2026-11-04.md` item 9. A date filter exits 2 and a list with no name is refused, though each row carries `date_of_prequalification`. Ian, 2026-11-04 (m24381): "All this stuff should get fixed in 1.0."
- Starts from: `crates/biomcp-query/src/drug/who.rs`, `crates/biomcp-adapters/src/who_pq/`, and the WHO search fields in `crates/biomcp-types/src/drug/query.rs`.
- Rules: WHO PQ is a local bundle, so filtering runs over the bundle; a search needs a name or a date range, never neither. Rows sort newest first when only a date range is given.
- Proof: installed-binary cases: a 2020 range lists products with dates in 2020; name plus range narrows; neither gives a usage error naming both.
- Changes: `crates/biomcp-query/src/drug/who.rs`, `crates/biomcp-adapters/src/who_pq/**`, `crates/biomcp-types/src/drug/query.rs`, `tests/cases/drug/who*/**`; the WHO PQ search, help, cases.
- Defers: nothing.
