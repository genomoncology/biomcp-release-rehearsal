# 0482 — Accept synonymous protein changes

Status: COMPLETE.
Landed: 2174a0f2e0f5973926cadbe976a46e6ed984012f

Milestone: 5

## Outcome

A synonymous protein change such as `p.Ala613=` is accepted wherever a protein change is, so `search variant --gene BARD1 --hgvsp p.Ala613=` finds ClinVar VariationID 2905096.

- Why: BioData's experiment 502 compared BioMCP 0.9.3 with an agent writing its own Python and curl on dated cancer questions, and the scripts won (m15051). A code triage against 1.0 at 8466b4700 found this defect still present with no ticket.
- Starts from: `crates/biomcp-types/src/query/protein_change.rs:4-27,49-80`, which accepts only residue codes.
- Rules: `=` means the same residue as the reference, per HGVS; the parser accepts `p.Ala613=` and `p.A613=`, writes back the HGVS form, and each source receives the spelling it supports; nothing else in the grammar widens.
- Proof: a parser case round-trips `p.Ala613=`; a case over a recorded reply finds BARD1 VariationID 2905096 and fails on the base build.
- Changes: the protein change parser, the adapters that send protein changes, cases and baseline.
- Defers: nothing.
