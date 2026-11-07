# 0723 — Schemas state each filter truthfully

Status: COMPLETE.
Landed: 1bcc68d5b24f921e49fe68422eda8b361f116799

Milestone: 5

## Outcome

Each MCP and CLI filter takes what its schema and help say. `product_code` works for 510(k) devices, the NCI trial search states or resolves its disease input, the Open Targets section error lists only what can be sent, and source names match across schema, help and server.

## Why

Items 5 and 6: author tool schema allowed `semanticscholar` while help says `semantic-scholar`; `product_code` rejected for 510(k); NCI search needs an NCIt code though the schema says "Disease or NCIt"; Open Targets takes one section but the error lists both. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then a case per named filter, plus a generated check that every schema enum and every filter a command advertises is accepted by that command, so this class fails in the gate. `crates/biomcp-adapters/src/fda_510k/` already reads `product_code`.

## Starts from

- Starts from: `crates/biomcp-types/src/`, `crates/biomcp-adapters/src/fda_510k/`, typed catalog `crates/biomcp/src/mcp/typed/catalog.rs`.

## Keeps

- Keeps: 0683 tool budget.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-types/`, `crates/biomcp-adapters/`, `crates/biomcp-query/`, `crates/biomcp/`, `tests/`.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

Read in code, not run. All four items still exist on main.

`product_code`: `fda_510k/` reads this field from replies, not as a search filter. Add it to `device_search.rs` `expression`, which PMA shares, and stop `validate_device` refusing it.

Author source names: the hosted schema filter keeps any word that is not a source name, so it removes `semantic-scholar` and leaves the alias `semanticscholar`. Stop advertising the alias, keep accepting it, and check the hosted tool list. Land after 0722, which edits the same list.

Open Targets: `get drug` takes one section per call for every source, and the error lists all of the source's sections without saying so. State the limit in the schema and in the error, for every source.

NCI: state the input and do not resolve names. The field says an NCIt code is needed for NCI, and points to `--condition-id` for MONDO or DOID identifiers.

The generated check is cut down. Extend the existing `query-validation/enumerated-letter-case` walker from one value per enum field to every value, sampling `country`, and add a pass over the hosted tool list. Build no new checker. It covers argument checking only. It does not reach adapters, so each named filter still has its own recorded case.

"Every filter a command advertises is accepted by that command" moves to 0676, because it needs the per-field source facts that ticket adds.

Land after the bench scored build, because this ticket shares files with 0693, 0717, 0720 and 0722.
