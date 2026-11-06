# 0726 — Cell line search ranks and filters for research

Status: OPEN.

Milestone: 5

## Outcome

Parent cell lines rank above derived lines, a caller can exclude derived lines, and a caller can search by gene or mutation where Cellosaurus records one.

## Why

Item 12: derived lines such as "AsPC-1 SPINK1 #10" crowd the results, and KRAS G12D lines cannot be searched. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then cases for ranking, the derived filter and a mutation search from Cellosaurus fields. If the bundled data lacks sequence variations, report it to the PM for `planning/sources-to-add.md`. No engineering of lines; this is lookup only.

## Starts from

- Starts from: `crates/biomcp-adapters/src/cellosaurus/`, `crates/biomcp-query/src/cell_line/`.

## Keeps

- Keeps: Cellosaurus terms.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-adapters/src/cellosaurus/`, `crates/biomcp-query/src/cell_line/`, `crates/biomcp-types/`, `tests/`.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

Cellosaurus is read from its live API and has no bundle, so this ticket changes no store format and needs no refetch. Replace "the bundled data" in the Proof with "the source".

Checked live. The source can filter both things this ticket needs. `id:AsPC-1 AND NOT hi:*` returned only the parent line, and `var:KRAS AND var:"p.Gly12Asp" AND ox:9606 AND NOT hi:*` returned 163 lines, all holding that variant. The `hi` field names a line's parent, and no reply on main reads it yet. Card replies carry variations; search rows do not.

Filters go in the source query, as with species and disease, and never after fetching, so paging stays correct. Re-sorting a returned window does not meet the Outcome: the exact NB4 line is outside its first ten rows. If a filter does not work at the source, that part stops and goes to the PM. Do not build a local pool.

`var:KRAS` alone also matches lines the source marks as having no reported mutation, so a gene filter alone must say what it matched. Mutation text is matched as the source writes it. Converting `G12D` to `p.Gly12Asp` is allowed only as a typed parse of a marked option. The reply says the match is on source text and that gene and mutation are not tied to one variation, so a short result is not read as complete.

Request the new fields only when a new filter is used, so an unfiltered search stays byte-identical. Add each new filter to `unfiltered()`.

Changes also covers generated help, `docs/reference/commands.md`, `tests/replies/cell-line/` and `tests/cell-line-command-budgets.yaml`.

The default order stays the source's, and callers choose the derived filter, because `decisions/2026-10-24-no-text-heuristics.md` rules out rerankers and allows a sort the caller asks for.

PM ruling: this replaces "Parent cell lines rank above derived lines" in the Outcome. Callers get the filter; the default order stays the source's.
