# 0812 — Add disclosed bounded FAERS drug-name expansion

Status: COMPLETE.
Landed: 2167e618c7fb567659448e1488c7e6eaf3028597

Milestone: 5

## Outcome

FAERS report and reaction-aggregate commands offer explicit drug-name expansion using the existing bounded drug resolver. Existing literal default and repeated --drug semantics remain unchanged. Replies disclose actual searched aliases and incomplete/ambiguous resolution, and continuations preserve the resolved query membership.

## Why

Ian asked to address the persona drug-filter report. Retained FAERS evidence shows literal pembrolizumab/myocarditis/death returns292 reports while the explicit pembrolizumab/KEYTRUDA union returns703 under identical filters. Literal source completion is currently truthful but easy to mistake for drug-wide coverage. VAERS has no equivalent drug-name interface.

## Proof

- Controlled CLI and MCP cases distinguish literal default from explicit expansion across generic/brand/medicinalproduct spellings. Overlapping names must yield one native OR query and native totals, never summed counts.
- Cover complete, ambiguous, incomplete and failed resolver evidence; caller source/component exclusions; disclosure of searched spellings and omissions; reaction/seriousness/date filter preservation; escaping and existing4096-byte query bound.
- Follow actual emitted continuations after changing resolver fixture availability; they must retain the exact original resolved spellings/native query. Report and aggregate modes use the same resolution policy.
- Run focused owning cases and routine gate. No new live data acquisition or entire release rebuild during development proof.

## Starts from

- Evidence: worktrees/bench-persona-pv-scientist/personas/pv-scientist retained JSON envelopes. Existing adverse_event/query.rs documents literal terms; adverse_event/faers.rs ORs supplied spellings across native generic_name/brand_name/medicinalproduct and exposes source_query. Reuse drug/search_names.rs rather than copying resolution logic.
- Current FAERS paging is native offset paging, without a retained resolver pool. Design the smallest typed continuation representation that freezes spellings and preserves caller controls; do not resolve afresh per page or accept unbounded opaque source expressions.

## Keeps

- Literal default, caller spelling, existing explicit OR, native filters/counts/offsets, source rights/exclusions, configured name limits and resolver budgets/completeness checks. Explicit expansion must not silently turn failed/incomplete name discovery into complete drug-wide coverage.
- VAERS remains its separate exact native vaccine-code aggregate surface. Do not add MyChem vaccine-code mappings, accept --drug there or merge independent tables. No undocumented default-semantics change, automatic external tool use, new dependency or wholesale query framework.

## Defers

- Vaccine-name discovery needs a separate native-catalog mapping design if requested. Preserve current rehearsal identity; assess affected schema/query/installed proofs before selecting a changed release candidate.

## Changes

- FAERS typed expansion option/continuation state, shared existing resolver integration, disclosure/help and owning outside-in fixtures. Report a design question before broadening the public contract.
