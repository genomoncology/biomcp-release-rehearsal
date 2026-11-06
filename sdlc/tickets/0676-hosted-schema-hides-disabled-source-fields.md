# 0676 — Hosted schemas hide fields whose every source is off

Status: OPEN.

Milestone: 5

## Outcome

In hosted mode, a typed tool's schema, help and `next` omit a query field or filter when every source that can answer it is turned off on that server, and say which sources a field needs. Each query owner states which sources answer each of its fields, and one shared projection reads those facts. Fields some served source can answer stay as they are.

## Why

0654's inventory (coordinator m27096) found about nine query owners with dozens of fields that only a disabled source answers: drug label indication, ChEMBL selectors, recall and WHO date fields; adverse-event MedDRA and publisher receipt fields; trial exact condition and biomarker code; diagnostic source groups and recall; disease OLS and SEER; PGx and phenotype fields. There is no shared notion of which sources answer a field today. The hosted evaluation's findings are the operational items 0654 fixes; this is the general form of "never offer what hosted refuses" and is too large for one owner before the cut.

## Proof

- Proof: hosted installed cases at `restricted` assert that, for each owner, a field whose sources are all off is absent from tools/list, help and `next`, and that a field a served source answers stays. Local mode unchanged. Schema tests for the shared projection. Routine gate.

## Starts from

- Starts from: the coordinator's field inventory in m27096, 0654's landed hosted projection, `decisions/2026-11-04-hosted-source-modes.md` and 0619's `hosted_max_use`.

## Keeps

- Keeps: hosted refusals and admission limits unchanged; only what is offered changes. No new service or framework.

## Defers

- Defers: nothing. The PM may split this by owner into disjoint tickets when a lane starts it.

## Changes

- Changes: the query owners' source facts in `crates/biomcp-query/src/`, the shared projection in `crates/biomcp/src/mcp/`, `tests/`.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

Read in code, not run. The shared projection already exists: `argument_allowed` reads `x-required-component`, and it already drives the tool list, help and `next`. This ticket adds facts to it and builds no new projection. Today the fact names one component on 5 fields. Extend it to a list, hiding a field only when every listed source is off.

Facts go on the query types in `crates/biomcp-types/src/`, so Changes covers that crate.

By their terms labels, ChEMBL, FDA recalls, OLS and SEER stay on at `restricted`, so start from the sources `restricted` and the deployment really turn off, confirmed with hosting. Other owners follow only if the check below finds them.

This ticket owns the check moved from 0723: with these facts, the enum walker sends each field with each source that declares it and fails if the command refuses it. One case per owner at `restricted`, not one per field.

The ticket changes what is offered and never what is served, so it does not gate the hosting deployment. It lands before 0640. Order it after 0723. Split by owner when a lane starts it: the first slice extends the fact and covers drug and diagnostic.
