# 0475 — Keep ClinVar haplotypes as one record with their member alleles

Status: COMPLETE.
Landed: 71168479719ef3cc7a7d37f5083412e458fe3d20

Milestone: 5

Depends on: 0468, 0473

## Outcome

A ClinVar variation of type Haplotype comes back as one record that names its kind, its VCV accession and version, its grouping and each member allele's ClinVar identity. It is never flattened into a single allele. VCV000441268 version 5 is the witness. It is an APOE haplotype on one chromosome with SimpleAllele members 17864 and 242765.

- Why: the user-query regression work (0461) found that the ClinVar adapter builds a single allele subject for a haplotype and drops its grouping and members, so two variants on one allele do not stay together (m14609). The 1.0 data model freezes at the tag.
- Starts from: the ClinVar adapter in `crates/biomcp-adapters/src/clinvar`, the BioData variant reference types, and the retained capture at `0461/user-query-regressions/clinvar-441268` on Yellow.
- Rules: keep the native kind, accession, version, `NumberOfChromosomes` grouping and member identities exactly as ClinVar states them. Infer no phase, biology or clinical meaning beyond what ClinVar states. A single-allele variation keeps its current JSON. New cases are listed additions for the BioData switch (0459). This ticket does not widen 0468's admission rewrite.
- Proof: VCV000441268 parses into one haplotype record with both members; a recording with a member removed fails the case; existing single-allele ClinVar replies are unchanged in the reply diff.
- Changes: the BioData reference type, the ClinVar mapping, schemas, cases and the baseline.
- Defers: compound records from sources other than ClinVar.
