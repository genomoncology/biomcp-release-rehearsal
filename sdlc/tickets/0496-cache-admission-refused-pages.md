# 0496 — Keep pages whose every record was refused out of the cache

Status: COMPLETE.
Landed: 1b33e09b80c4d9f58d8734c4cfdacfc96aaeb92d

Milestone: 5

Depends on: 0494, 0495

## Outcome

The cache admits a source page only when it is a valid native empty page or holds at least one accepted identity. A nonempty native page whose records were all refused still returns its problems and native counts, but it never enters the cache, so a later call asks the source again instead of replaying a page BioMCP could not read.

- Why: the coordinator's root cause analysis (m15928). GWAS (fixed at 458fd5032) and Monarch phenotype search made raw identity fields optional so mapping could refuse single records, while transport admission still accepted an all-refused page. A bounded inventory found the same gap in ChEMBL search, mechanisms and indications; MyChem search; Cellosaurus candidates; cBioPortal studies and sections; SEER statistics; and Monarch associations.
- Starts from: GWAS's `identities_admitted`, the cache admission check in `crates/biomcp-fetch`, the sites named above, 0494's window values and 0495's refusal facts.
- Rules: admission reads 0494's window value and 0495's refusal fact; no pagination or cache framework; strict scope and protocol checks and native paging stay; optional metadata on accepted identities still caches.
- Proof: for each named family, a case over recorded replies whose native page has records that are all refused shows a second call making a fresh request; it fails on the base build. A valid native empty page still caches.
- Changes: cache admission, the named adapter families, cases and baseline.
- Defers: nothing.
