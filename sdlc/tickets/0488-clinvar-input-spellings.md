# 0488 — Accept ClinVar's own ID and HGVS spellings

Status: COMPLETE.
Landed: 565abbc0cc47b811a25b1099ca913966930238ea

Milestone: 5

## Outcome

`get variant` accepts ClinVar's own spellings: `ClinVar:577152` in any letter case, and the HGVS form with the gene in parentheses, such as `NM_000051.4(ATM):c.6115G>A`.

- Why: BioData's root cause analysis of experiment 502 (m15356, m15395), where an agent writing its own scripts beat BioMCP 0.9.3 on dated cancer questions; the analyst checked this against 1.0 at 19a748042.
- Starts from: the variant ID and HGVS parsers in `crates/biomcp-types/src/query/` and `tests/cases/variant/identity.yaml`.
- Rules: the prefix is matched without regard to case and written back as `clinvar:`; the gene in parentheses is checked against the transcript where the source states one and is dropped from the request; a gene that disagrees with the transcript is refused with a named error; nothing else in the grammar widens.
- Proof: cases accept `ClinVar:577152` and `NM_000051.4(ATM):c.6115G>A` and fail on the base build; a mismatched gene is refused.
- Changes: the variant parsers, cases and baseline.
- Defers: nothing.
