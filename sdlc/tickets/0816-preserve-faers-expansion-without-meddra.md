# 0816 — Preserve FAERS expansion when unrelated MedDRA is excluded

Status: COMPLETE.
Landed: 05d08c0ebec64deb2579f93b31027b79ce7c9dff

Milestone: 5

## Outcome

Plain FAERS report searches retain explicit drug-name expansion and their frozen continuation membership when MedDRA is excluded. Reaction-filtered and aggregate requests still enforce their actual MedDRA dependency. Primary FAERS exclusion remains source-off without reading a saved payload.

## Why

Independent inspection of landed4845 found adverse_event.rs unconditionally bypasses names::search when MedDRA is disabled. Plain reports do not depend on that component, so native search succeeds with the original literal names and silently ignores expansion/name_snapshot. This is a Medium correctness defect in the new integration, distinct from the label recovery edge that prompted the check.

## Proof

- Create an expanded plain-report page, then add MedDRA exclusion to its actual emitted CLI/MCP continuation. Require identical expanded native query, retained aliases/snapshot and correct offset, with zero resolver requests.
- A fresh plain report with MedDRA excluded still expands. Reaction-filtered reports and counts/aggregates with MedDRA excluded refuse without provider requests.
- Primary FAERS exclusion stays source-off without internal missing-restart error, saved-payload reuse or executable blocked-provider action. Preserve existing resolver-dependency exclusion controls.
- Prove the new case fails against landed old behavior and passes fixed, run focused owning cases and routine gate; no live source call or new release build.

## Starts from

- crates/biomcp-query/src/adverse_event.rs search_command routing, adapters faers/search.rs conditional reaction dependency, query adverse_event/counts.rs aggregate dependency and existing faers_names caller-control fixtures. The current primary-source exclusion already short-circuits safely; do not claim its label missing-restart failure also exists here.

## Keeps

- Literal default, opt-in behavior, existing native query limits/counts/offsets, frozen aliases, source/component rights and caller authorization. Gate MedDRA only when the request actually consumes it; do not weaken reaction/aggregate policy or silently fall back to a different query when a required component is excluded.

## Defers

- Label-specific recovery remains in0815. No broader source-policy redesign or automatic provider fallback. Retain original qualification source identities.

## Changes

- FAERS expansion routing condition and owning CLI/MCP caller-control regression. No schema or dependency change expected.
