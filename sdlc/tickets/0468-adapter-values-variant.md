# 0468 — Apply the bad-value rule in the variant annotation adapters

Status: COMPLETE.
Landed: 3d60fd908c9c0fc4cc334fe43865b12bfa284711

Milestone: 5

Depends on: 0467, 0442

## Outcome

The adapters `allele_registry`, `alphagenome.rs`, `clingen`, `clinvar`, `gnomad`, `litvar`, `mutalyzer`, `myvariant` and `variantvalidator` keep a reply when one non-identity value is bad, and none of their audit or file-rule entries stays pending.

- Why: the pre-1.0 code quality review, split from 0418 into whole family landings (m13852).
- Starts from: 0467's pending list for these adapters, each adapter's `raw.rs` and `map.rs`, and 0417's helpers in `mapped.rs` and `raw.rs`.
- Rules: the field table in restructure (d) and 0418's rule: omit one bad non-identity value with one problem naming the field, refuse only a record with a bad identity, fail only on a bad protocol value. Use 0417's helpers and add none unless two adapters need the same new one. A reply property that leaves a required set takes a major under 0437's gate. Leave `nci`, `fda_*`, `maude`, `drugsfda`, `openfda_label`, `faers`, the recall adapters and `ctgov` alone. Every pending site is fixed or classified with a reason; no site stays unclassified. The lane rebases on 0436's variant ID forms. A bundle row that lacks every usable query key is disclosed through 0474's shared rejection channel, not in this family; this ticket fixes rows with a checked native key locally (m14304 ask 3). LitVar (m14582): candidate `name`, `rsid` and the publication counts become optional metadata and native `_id` stays the identity; a bad name or rsid only blocks selections that need that field, and a malformed or incoherent `pmids_count` is omitted with a named problem while valid PMID rows stay. No inferred zero, repaired count or broadened selection. The `ArticlePage` successor schema and its consumers change together.
- Proof: per adapter, a replay case edits one non-identity value in a real recording and asserts the record kept, the field absent and the problem named, and it fails on the base build; the family's pending audit and cap entries are gone; the reply diff lists only the named cases.
- Changes: these adapter folders, their audit entries, `tests/cases/`, schemas and the baseline.
- Defers: helper copies (0424).
