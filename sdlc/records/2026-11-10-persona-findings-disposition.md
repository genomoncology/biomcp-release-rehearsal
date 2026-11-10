# Persona finding dispositions

Review of the merged biomed-agent-bench suite at 94160cd6529f4ce2a228a0f87ba81795c75c869b against core source and retained reproductions. This records evidence and decisions; live ticket state remains in pm.

| Report | Evidence-based disposition | Owning work |
| --- | --- | --- |
| Recurrence unknown sample set returns zero | Confirmed: invalid list admission could still produce zero observations. Existing source failure/partial diagnostics mean the report's “no error” wording overstates the finding. | Admit recurrence sample sets before counting mutations, 0813. |
| Trial token with mismatched offset | Confirmed metadata/end-detection defect. Also found a generated exact-condition action retaining offset without its cursor. | Bind trial continuations to native position/selectors, 0814. |
| Rejected trial cursor looks like grammar failure | Confirmed provider HTTP-400 mapping defect; ordinary tokens are not rejected locally merely because they are arbitrary strings. Cursor expiry/lifetime remains unproved. | Rejected-continuation recovery, 0805. |
| Protein starts off by one | Numeric allegation rejected: native one-based inclusive to zero-based interbase conversion is correct. Text discoverability was inadequate. | Protein coordinate presentation, 0811; numeric values unchanged. |
| FAERS generic/brand coverage | Explicit discovery feature gap relative to truthful literal default, rather than broken native count. | Bounded opt-in expansion, 0812. |
| FAERS expansion with MedDRA excluded | Confirmed integration defect: unrelated component exclusion could silently bypass expansion for plain reports. | Preserve expansion under unrelated exclusions, 0816. |
| Label generic/brand coverage | Explicit discovery feature gap. Old revision does not prove stale cache or obsolete label; revisions cannot rank different SPL sets globally. | Bounded label-name expansion, 0815. |
| Multi-allelic population silently picks one | Not reproduced. Same-build raw get/population calls retained all three rs121918623 alleles and three population actions, with no CAR/gnomAD request before selection. Explicit clinvar:12883 resolved G>A successfully. No speculative fix ticket. | Retained witness below; reopen only with contradictory raw source identities/envelopes. |
| BRAF V600E versus V600 normalization | Claimed superset contradicted by retained rows. Specific query has four trials, broader query ten, only one shared; V600E-only NCT06887088 is absent from V600 rows. V600 lacks the substitution and is not an equivalent spelling. | Idea0817, explicitly labeled broader discovery. |
| Nonvalvular versus non-valvular indications | Different literal openFDA phrase matches, not evidence of the same failed variant-normalization mechanism. | Separate phrase-discovery design if pursued; no silent semantic expansion. |
| Recall classification missing from search | Class II capture is from device/enforcement; search uses device/recall RES. Existing detail already exact-joins enforcement. No dropped RES field or upstream repair established. | Idea0818, bounded exact enforcement enrichment; filtering separate. |
| Get-trial limit ignored | Not established. Limit bounds whole records; section byte budgets can return fewer. Text sections use text budgets. Current collection renderer enforces the limit. | Reopen with a collection exceeding its requested limit; do not rename it to section-limit. |
| Expired pool remedy needs omitted ID | Not established. Current remedy is an executable token-free restart preserving selectors, clearing pool/offset. | Reopen with the actual contrary remedy/envelope. |
| GTR test-name search | New feature: current unsupported selector is rejected honestly. Names already exist in test_version.gz. | Idea0819, whole pinned-catalog matching before counts/paging, including name-only orphan tests. |
| FDA CDx date filter/sort | New feature requiring date-contract inventory. Native device cells can have missing/multiple dates; table currently preserves physical rows. | Idea0820, explicit global snapshot ordering/bounds with original cells, ties and unparseable-date disclosure. |

## Evidence pointers

- The merged suite is under `repos/biomed-agent-bench/personas/`. Findings were checked against raw envelopes where available, not accepted from rollup prose alone.
- Valid recurrence: `tumor-board/evidence/q5-recurrence-pik3ca.json`, 128 observed/355 examined. Invalid-list persona observation is log-only; controlled old-runtime job j37148 supplies the actual regression witness.
- Allele reproduction: Yellow `data/biomcp-redux-evidence/0788/variant-allele-witness-ca945a56a`, manifest `b40145668cffd513c864f77977d4b217d15d0bf6397bd9f6da300f3a805e6ffa`. Genuine selected-source binary, shared isolated configuration/cache, three bounded calls, raw envelopes/records/observations retained.
- Variant spelling: `patient-navigator/evidence/q7-rows-p1.json` and `q7-rows-v600.json`; log synthesis is not a raw superset or eligibility proof.
- Recall: `dx-lab-director/evidence/cmp-openfda-recall-illumina.json` is enforcement data; the retained pv-scientist RES capture has no classification. Compare feed identity before proposing a field mapping.
- Label: `pt-committee` retains distinct apixaban/ELIQUIS search and exact-set replies; trimmed envelopes lack native harmonized name fields, so the precise provider annotation cause is unproved.

No new provider calls were needed for the merged-suite triage. Earlier bounded allele reproduction and controlled regressions retain their actual source/job identities. No public issue or message was sent, and no biomedical equivalence was inferred from search wording.
