# 0660 — Variant joins bridge builds and IDs

Status: COMPLETE. Current main b4117cdd5 already resolves the investigated GRCh37 variant to GRCh38 and ClinGen CAid before CIViC/gnomAD joins in both modes; fresh independent evidence ACCEPT at Yellow0660/current-main-b4117cdd5-j30638 manifestd5e05c2a32b6211b5055d9e1362c693d90464e6c20368011a7c038b19af111a1. Original evaluation request unavailable; reconstructed case disclosed, j30638 harness RED preserved, no product change or test probes landed.

Milestone: 5

## Outcome

`get variant` resolves a GRCh37 or HGVS input to GRCh38 and a ClinGen allele ID before joining gnomAD and CIViC, so `KRAS G12D` returns their rows instead of `no-join-id`, and echoes the resolved IDs.

- Why: the hosted evaluation of 2026-11-05 (hosting m26488; evidence under `data/biomcp-hosting-evidence/2026-11-05-claude-eval/`), capability 4: search returned GRCh37 HGVS, and gnomAD and CIViC answered `no-join-id`. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).
- Starts from: the variant join planning in `crates/biomcp-query/src/variant/` and 0588's variant ID forms.
- Rules: Investigate first: reproduce on current main in single and typed mode with the evaluation's command; if main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. Use public resolvers BioMCP already calls (ClinGen Allele Registry, Ensembl); add no new source without its license check.
- Proof: replayed `get variant` for KRAS G12D with gnomAD and CIViC rows present; routine gate.
- Changes: `crates/biomcp-query/src/variant/`, `tests/`; the fix and its cases.
- Defers: nothing.
