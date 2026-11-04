# 0472 — Apply the bad-value rule in the disease, drug, study and remaining adapters

Status: COMPLETE.
Landed: 0e9d541be551d6bff42bd017613ff4277e8e674b

Milestone: 5

Depends on: 0467

## Outcome

The adapters `mydisease`, `disease_sources`, `monarch`, `hpo`, `ols`, `mychem`, `chembl`, `drug_sources`, `ddinter`, `ema`, `who_ivd`, `who_pq`, `seer`, `gwas`, `cbioportal`, `cellosaurus`, `vaers`, `adverse_event_sources` (its non-FAERS paths), `registry`, `guidance.rs` and `sheet`, plus any in-scope adapter the audit lists that no other family names keep a reply when one non-identity value is bad, and none of their audit or file-rule entries stays pending.

- Why: the pre-1.0 code quality review, split from 0418 into whole family landings (m13852).
- Starts from: 0467's pending list for these adapters, each adapter's `raw.rs` and `map.rs`, and 0417's helpers in `mapped.rs` and `raw.rs`.
- Rules: the field table in restructure (d) and 0418's rule: omit one bad non-identity value with one problem naming the field, refuse only a record with a bad identity, fail only on a bad protocol value. Use 0417's helpers and add none unless two adapters need the same new one. A reply property that leaves a required set takes a major under 0437's gate. Leave `nci`, `fda_*`, `maude`, `drugsfda`, `openfda_label`, `faers`, the recall adapters and `ctgov` alone. Every pending site is fixed or classified with a reason; no site stays unclassified.
- Proof: per adapter, a replay case edits one non-identity value in a real recording and asserts the record kept, the field absent and the problem named, and it fails on the base build; the family's pending audit and cap entries are gone; the reply diff lists only the named cases.
- Changes: these adapter folders, their audit entries, `tests/cases/`, schemas and the baseline.
- Defers: helper copies (0424).
