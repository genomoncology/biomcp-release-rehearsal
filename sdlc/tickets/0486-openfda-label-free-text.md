# 0486 — Find drug labels whose openFDA names are empty

Status: COMPLETE.
Landed: 5bfa61687f693467b38fafa898386631776ea1cb

Milestone: 5

## Outcome

`search drug --source openfda-label CAVHANZA` finds the CAVHANZA label (set ID 98b197ea-bca4-427e-8737-97f70a3262bc) even though its `openfda` block is empty, and a label can also be found by its application number.

- Why: BioData's root cause analysis of experiment 502 (m15356, m15395), where an agent writing its own scripts beat BioMCP 0.9.3 on dated cancer questions; the analyst checked this against 1.0 at 19a748042.
- Starts from: `crates/biomcp-adapters/src/openfda_label/` (`mod.rs` search order and `raw.rs`, which rejects records with empty openFDA names) and the label fallback case, which always writes names into the `openfda` block.
- Rules: after the exact brand or generic match and the word match find nothing, search openFDA's free text (`search=`) and say in the reply that the free-text step ran; accept application numbers such as NDA220089 and BLA761445 as a search term; a record with an empty `openfda` block is admitted, its missing names omitted with no problem, because openFDA leaves the block empty for some labels; never take a different product's label. Empty-name match (m15458): a free-text hit with an empty `openfda` block counts only when the complete requested name starts `spl_product_data_elements` and appears in the principal display panel followed by a trademark or registered-mark boundary; warnings and clinical text are never scanned, a shorter brand inside a longer marked name does not match, and a record that cannot be verified this way stays out. Application-number search matches the label's application number exactly. No extra source or lookup.
- Proof: a case over a recorded reply with an empty `openfda` block finds CAVHANZA by free text and fails on the base build; an application-number search finds its label; the existing exact and word cases are unchanged in the reply diff.
- Changes: the openFDA label adapter, query type, cases and baseline.
- Defers: nothing.
