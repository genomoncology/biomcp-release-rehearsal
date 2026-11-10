# 0814 — Bind trial continuations to their native position and selectors

Status: COMPLETE.
Landed: 8278dd1a2d6723c7bcf73f044ff991e3f163c643

Milestone: 5

## Outcome

ClinicalTrials.gov continuations retain a checked association between the actual provider cursor, expected consumed-row offset, matching mode and search selectors. Changing the public offset or selectors refuses before transport instead of relabeling duplicate rows or suppressing the next page. Exact-condition recovery always starts at page zero without a cursor.

## Why

CTGov receives only pageToken while BioMCP trusts an independent caller offset for page metadata and end detection. Code inspection confirms mismatched offsets can mislabel pages. A retained third-page response also emits exact-condition recovery with offset40 but no token, which cannot execute. Ian asked to address both continuation issues.

## Proof

- Obtain an actual emitted continuation through CLI/MCP; change offset, filters or matching mode and require specific recovery guidance with zero provider requests. Follow the unmodified continuation successfully, preserving native-consumed counts rather than admitted-row counts.
- Cover partially/all-refused native pages, unknown total, broader-name matching, stable native cursor forwarding, caller exclusions and explicit fresh restart.
- On a later page execute the emitted exact-condition action; require cleared token/offset and preserved other selectors. Preserve 0805 rejected-provider-continuation recovery cases after merging its landed fix.
- Exercise legacy/unbound and missing/disallowed retained bindings: specific fresh-first-page guidance, no guessed offset, hidden network restart or arbitrary native request. Prove old behavior fails focused public tests and fixed behavior passes.

## Starts from

- biomcp-types query/trial/continuation.rs and condition.rs; adapters ctgov/filters.rs; query/trial.rs page metadata/end detection. Raw benchmark q2-competitors-p3.json has the broken exact-condition action; trial-ops log describes token/offset misuse.
- Existing first_page() resets cursor/offset and existing authorized retained snapshot/capability infrastructure can retain bindings. Use that shared store rather than designing a separate secret/signature or unbounded encoded-expression framework.

## Keeps

- Keep provider cursor bytes opaque/unchanged at transport; native source paging, exact/broader mode, query filters, source/component exclusions and existing count/rights/budget policies. Reuse checked stored capabilities to bind selectors/offset, including caller authorization before payload access.
- Compatibility choice: arbitrary legacy native tokens cannot prove offset locally. Accept them only when a known retained binding validates the association; otherwise return explicit fresh restart guidance. Do not silently accept unverifiable offset metadata, invent a cursor lifetime or perform automatic retries. Preserve other trial registries unless an independently demonstrated defect requires change.

## Defers

- Registry expiry research and automatic cache refresh are separate. Existing rehearsal retains its source identity; qualify affected paging/schema/installed paths on the ending candidate without restarting unrelated platform investigations.

## Changes

- Trial checked continuation/first-page actions, shared retained binding integration and owning CLI/MCP fixtures. Report any public-shape conflict before broadening the design.
