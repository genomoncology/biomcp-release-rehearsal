# 0418 — Apply the bad-value rule in every adapter outside the benchmark families

Status: COMPLETE.
Landed: 81bfa5d301df914ee6c20813c1f646ad68195bd8

Milestone: 5

Depends on: 0467, 0468, 0469, 0470, 0471, 0472

## Outcome

Every adapter outside the openFDA, NCI and ctgov families keeps a reply when one non-identity value is bad: it omits that value with one problem naming the field, refuses only a record with a bad identity, and fails only on a bad protocol value. The audit sees every silent-drop form in adapter files, and no adapter site stays unclassified.

- Why: the pre-1.0 code quality review. About 32 adapters fail whole replies on one bad date or number, such as semantic_scholar, erepo, uniprot, litsense, mutalyzer and variantvalidator, and 120 audit sites are pending.
- Starts from: the helpers 0417 adds to `crates/biomcp-adapters/src/mapped.rs` and `raw.rs`, `crates/biomcp/tests/ratchets/adapter_values.rs`, `adapter_value_audit.json`, `ratchets/source.rs`, each adapter's `raw.rs` and `map.rs`.
- Rules: the field table in restructure (d). Use 0417's helpers and add none unless two adapters need the same new one. The audit fingerprints `.ok()`, `if let Ok`, `let Ok .. else` and `filter_map(..ok())` in adapter files, and `unwrap_or_default` in `raw.rs`, in `map.rs` and on parse results. The query layer joins the audit in 0420. `ratchets/source.rs` applies the 300-line cap and the `serde(default)` and `deny_unknown_fields` refusals to every adapter file. Leave `nci`, `fda_*`, `maude`, `drugsfda`, `openfda_label`, `faers`, the recall adapters and `ctgov` alone. The work lands in 0467 (shared audit and file rules) and the family tickets 0468 to 0472 (m13852). This ticket closes when they have landed: it checks that the pending list is empty, that every in-scope adapter directory belongs to one family, and runs the reply diff across all families. It needs no code unless that check finds a gap. A reply property that leaves a required set takes a major under 0437's gate.
- Proof: per adapter, a replay case edits one non-identity value in a real recording and asserts the record kept, the field absent and the problem named, and it fails on the base build; a planted `.ok()` drop in an adapter fails the audit; the reply diff lists only the named cases.
- Changes: adapter folders in scope, the ratchets, `tests/cases/`, schemas and the baseline.
- Defers: openFDA and NCI (0455), helper copies (0424), the query-layer audit (0420).
