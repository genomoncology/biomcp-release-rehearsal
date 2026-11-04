# 0435 — Refuse malformed HGVS text before any source call

Status: COMPLETE.
Landed: 4762723f34bf9af13deb0046a82f63283471c8d3

Milestone: 5

## Outcome

`HgvsText` and `Expression` refuse text that is not plausible HGVS before any source call: whitespace, a body with no digit except the positionless forms the HGVS specification allows (such as `p.?`, `p.0`, `p.=` and `r.spl`), and a protein reference with a nucleotide body. `Expression` is described as a reference and syntax check only, with no full parser.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 1. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: `crates/biodata/src/variant/notation.rs:48-60` and `:126-175`.
- Rules: stays within the 2026-10-24 variant rulings: no full HGVS parser.
- Proof: CLI cases where `get variant NM_005228.5:c.???` and `variant normalize` with a trailing word exit 2 with no source call, plus a parse table for the same strings.
- Changes: notation.rs, its schema pattern if it changes, help and cases.
- Defers: nothing.
