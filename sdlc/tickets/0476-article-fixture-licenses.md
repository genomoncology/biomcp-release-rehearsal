# 0476 — Keep only openly licensed article text in public fixtures

Status: COMPLETE.
Landed: 6be90cda35b1b9d97e37db8df3eb71052a5f173e

Milestone: 5

## Outcome

Every PMC full text, supplement and PDF capture under `tests/replies/article/` comes from an article licensed CC0, CC BY or public domain, and a check fails when a capture's recorded license is anything else.

- Why: the licensed-data audit for Ian's rule (m14819) found the repository clean of GENIE, UMLS, TCGA, JAX-CKB, OncoKB, COSMIC, OMIM and ISCN content, but some live PMC open-access captures may be CC BY-NC or carry no reuse license, and the repository is public.
- Starts from: `tests/replies/article/**` (the `live-*` and `saves/pdf` captures) and the license each PMC article declares.
- Rules: replace a non-qualifying capture with a CC0 or CC BY article that exercises the same behavior, or with synthetic text where no real article is needed. Keep each case's purpose. Add no new framework; the check reads the license the capture already records, or a small license list beside the captures.
- Proof: a listing of every article capture with its license; a planted CC BY-NC capture fails the check; the reply diff lists only the replaced cases.
- Changes: article captures, their cases and the check.
- Defers: nothing.
