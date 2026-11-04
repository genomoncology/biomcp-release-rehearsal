# 0469 — Apply the bad-value rule in the clinical interpretation adapters

Status: COMPLETE.
Landed: 3bc574c4282e7a6b1edc56eaf1c9c3f5cfe02f0f

Milestone: 5

Depends on: 0467

## Outcome

The adapters `civic`, `oncokb`, `hotspots`, `cpic`, `clinpgx`, `pgx_sources.rs`, `gencc.rs`, `gtr`, `erepo` and `assertion.rs` keep a reply when one non-identity value is bad, and none of their audit or file-rule entries stays pending.

- Why: the pre-1.0 code quality review, split from 0418 into whole family landings (m13852).
- Starts from: 0467's pending list for these adapters, each adapter's `raw.rs` and `map.rs`, and 0417's helpers in `mapped.rs` and `raw.rs`.
- Rules: the field table in restructure (d) and 0418's rule: omit one bad non-identity value with one problem naming the field, refuse only a record with a bad identity, fail only on a bad protocol value. Use 0417's helpers and add none unless two adapters need the same new one. A reply property that leaves a required set takes a major under 0437's gate. Leave `nci`, `fda_*`, `maude`, `drugsfda`, `openfda_label`, `faers`, the recall adapters and `ctgov` alone. Every pending site is fixed or classified with a reason; no site stays unclassified. A bundle row that lacks every usable query key is disclosed through 0474's shared rejection channel, not in this family; this ticket fixes rows with a checked native key locally (m14304 ask 3). OncoKB `gene_type` (m14304 ask 4): keep the exact native text, add the canonical coding only for the five published literals, and for an unknown literal omit the canonical coding with one problem naming the field; never refuse the gene and never map an unknown value to another category.
- Proof: per adapter, a replay case edits one non-identity value in a real recording and asserts the record kept, the field absent and the problem named, and it fails on the base build; the family's pending audit and cap entries are gone; the reply diff lists only the named cases.
- Changes: these adapter folders, their audit entries, `tests/cases/`, schemas and the baseline.
- Defers: helper copies (0424).
