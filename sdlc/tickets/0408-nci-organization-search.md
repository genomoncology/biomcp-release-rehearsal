# 0408 — Search NCI organizations

Status: COMPLETE.
Landed: 5325f69625ecd81c8d39eb0a577f464280621520

Milestone: 5

## Outcome

`biomcp search organization --source nci` finds NCI clinical trial organizations by name (NCI's partial match), city, state or province, country, postal code, organization family or a coordinate radius. Each header gives the exact NCI name, city, state, country and NCI's trial count; the card adds address, coordinates, family and public contact fields as published. There is no `get`, because NCI publishes no organization ID; help says names are not unique and that the count covers trials of every status, not only recruiting ones.

- Why: `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. MCP-Bench asks whether trial sites match Boston's large cancer centers.
- Starts from: the NCI adapter and key handling, NCI CTS API v2 `/organizations` and its documented filters (`name`, `org_city`, `org_state_or_province`, `org_country`, `org_postal_code`, `org_family`, `org_coordinates_lat/lon/dist`).
- Rules: follow `sdlc/planning/architecture.md` and the shared source rules in `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. NCI returns zero, not an error, for a wrong field name, so every NCI proof includes a positive case that a wrong build would fail.
- Paging (PM ruling, the adversarial design review of 2026-10-28): `/organizations` takes only `size` and returns a total only when unsized. BioMCP makes one unsized request under a byte bound, keeps it as a snapshot and pages it locally by offset; above the bound it refuses with a typed "too broad" reason naming the narrowing filters. NCI's `too_many_buckets` error maps to a source problem, never zero results.
- Proof: replay cases for a name search, a city and state search, local paging, the too-broad refusal, a synthetic bucket error, a missing key and a bad value with a problem; one bounded keyed capture.
- Changes: a new organization entity, `crates/biomcp-adapters/src/nci`, schemas, help, the MCP command tree and cases.
- Defers: organizations from other registries.
