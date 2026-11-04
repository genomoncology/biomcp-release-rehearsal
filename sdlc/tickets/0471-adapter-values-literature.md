# 0471 — Apply the bad-value rule in the literature adapters

Status: COMPLETE.
Landed: c5f90cf5914ee3088fe00858366edf69ddaec779

Milestone: 5

Depends on: 0467

## Outcome

The adapters `pubmed`, `pubtator`, `europepmc`, `pmc_oa`, `jats`, `pdf`, `litsense`, `semantic_scholar`, `icite` and `orcid` keep a reply when one non-identity value is bad, and none of their audit or file-rule entries stays pending.

- Why: the pre-1.0 code quality review, split from 0418 into whole family landings (m13852).
- Starts from: 0467's pending list for these adapters, each adapter's `raw.rs` and `map.rs`, and 0417's helpers in `mapped.rs` and `raw.rs`.
- Rules: the field table in restructure (d) and 0418's rule: omit one bad non-identity value with one problem naming the field, refuse only a record with a bad identity, fail only on a bad protocol value. Use 0417's helpers and add none unless two adapters need the same new one. A reply property that leaves a required set takes a major under 0437's gate. Leave `nci`, `fda_*`, `maude`, `drugsfda`, `openfda_label`, `faers`, the recall adapters and `ctgov` alone. Every pending site is fixed or classified with a reason; no site stays unclassified. License (m15228): a malformed license shape is a permission failure, so the permission-dependent content (full text, supplements, article text, evidence passages) is withheld with one named license problem, while unrelated valid metadata stays; a well-formed but unrecognized license string stays `License::Other` as today. Public license fields stay required.
- Proof: per adapter, a replay case edits one non-identity value in a real recording and asserts the record kept, the field absent and the problem named, and it fails on the base build; the family's pending audit and cap entries are gone; the reply diff lists only the named cases.
- Changes: these adapter folders, their audit entries, `tests/cases/`, schemas and the baseline.
- Defers: helper copies (0424).
