# 0006 — Compare schemas with the last public release

Status: WITHDRAWN. Filed too early: nothing is released and no baseline source exists. Milestone 5 now names the release schema baseline; it is refiled when release preparation starts.

Milestone: 5

## Outcome

The release gate detects same-major schema breaks against an independent copy of the last released schemas, even when code and current checked-in schemas change together.

Depends on: 0005

- Starts from: The compatibility helper and schema registry in 0005, the release pipeline when milestone 5 starts, architecture section 3.9 and the PM's 2026-10-24 ruling on the schema compatibility baseline.
- Scope: Select an independent, reproducible source for the last released schemas and compare the candidate contracts before release. Define the first-release case explicitly when no prior public schema contract exists. Reuse compatibility logic and avoid a second schema framework. Review exact file claims when this milestone starts.
- Keeps: Ordinary development uses generation, drift checking and compatibility before update; reviewers inspect schema snapshot changes. This ticket adds no live source calls or unrelated release behavior. Publishing still needs Ian.
- Proof: A release fixture changes a type and its checked-in schema together without raising the major; the independent comparison rejects the break. A compatible addition passes, a major increase permits the documented break, and the initial release case is explicit. Run the release checks through pm job on the actual release candidate.
- PM decision: 2026-10-24. This independent baseline must land before the first public release. No historical baseline is required for the unreleased core-schema foundation.
