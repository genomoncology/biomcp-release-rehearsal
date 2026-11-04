# 0512 — Pair ClinPGx snapshot references in the reply diff

Status: COMPLETE.
Landed: 275d9101ad8569c404866e1ad0bafb17eb154e4a

Milestone: 5

## Outcome

The reply diff pairs the opaque ClinPGx snapshot that `biomcp.pgx-annotations/5` declares through its root `next[kind=page]` arguments, with every reference to it and its lifetime, the same way it pairs the phenotype card's declarations. Any other difference still fails.

- Why: the 0419 comparison (m17041). The normalizer recognizes only phenotype-card snapshot declarations, so four ClinPGx calls per side stay unmatched and their references differ.
- Starts from: the 0501 and 0509 normalizer and the retained 18-command ClinPGx captures per side from the 0419 comparison.
- Rules: bind the actual typed declaration, references and lifetime; no data mask; no recapture.
- Proof: the retained ClinPGx captures pair after the change and fail before it; a broken reference and a changed lifetime each still fail.
- Changes: the reply diff normalizer and its tests.
- Defers: nothing.
