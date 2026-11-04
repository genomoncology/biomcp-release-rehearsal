# 0479 — Find Drugs@FDA approvals by brand name

Status: COMPLETE.
Landed: 8b85789959e225317468f7d41fe8d38fda104a4e

Milestone: 5

## Outcome

`search drug --source drugsfda` matches a brand name such as SARCLISA ESCENA as well as the active ingredient, so an agent can reach the approval record (BLA761445, approved 2026-07-09) from the name a clinician uses.

- Why: BioData's experiment 502 compared BioMCP 0.9.3 with an agent writing its own Python and curl on dated cancer questions, and the scripts won (m15051). A code triage against 1.0 at 8466b4700 found this defect still present with no ticket.
- Starts from: `crates/biomcp-adapters/src/drugsfda/mod.rs:43-53`, which matches the active ingredient only, and the Drugs@FDA search query type.
- Rules: match `products.brand_name` in addition to the active ingredient, using the same exact-then-word order the openFDA label search uses; each result keeps its application number and get step; no name lookup is added to `get drug`.
- Proof: a case over a recorded openFDA reply finds SARCLISA ESCENA's BLA by brand name and fails on the base build; an existing ingredient search reply is unchanged in the reply diff.
- Changes: the Drugs@FDA adapter, query type, cases and baseline.
- Defers: nothing.
