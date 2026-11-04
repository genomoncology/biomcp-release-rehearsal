# 0481 — Search variants by gene and coding change

Status: COMPLETE.
Landed: d49c28082766a78cba1d28887b32a4cd6b557063

Milestone: 5

## Outcome

`search variant --gene PTEN --coding c.333G>T` finds the ClinVar record for a gene plus coding change without a transcript, including intronic forms such as FLCN c.780-4C>G and MLH1 c.678-14_678-3del, and lists each match with its get step.

- Why: BioData's experiment 502 compared BioMCP 0.9.3 with an agent writing its own Python and curl on dated cancer questions, and the scripts won (m15051). A code triage against 1.0 at 8466b4700 found this defect still present with no ticket.
- Starts from: `search variant` options, the protein and coding change parsers in `crates/biomcp-types/src/query/`, the ClinVar adapter, and the refusal case `tests/cases/variant/identity.yaml:45`.
- Rules: send `G[GENE] AND "c.…"[VRNM]` to ClinVar E-utilities; the coding change goes through the existing HGVS coding grammar, so a malformed change is refused with a named error; never guess a transcript; more than one match lists every match, as the existing ambiguous protein refusal does; `get variant` keeps requiring a stable ID. `tests/cases/variant/identity.yaml`'s step that asserts the refusal of gene shorthand changes to point to this search. Coding grammar (m15417): no checked coding grammar exists, so this ticket adds one small checked coding-change value in `crates/biodata`, because BioData owns HGVS notation. It follows HGVS 21.1 for a `c.` change without a reference: substitution, deletion, duplication, insertion and delins, with single positions or ranges, intronic offsets (`+n`, `-n`) and UTR positions (`-n`, `*n`). Alleles (`[...;...]`), uncertain positions and `p.` or `g.` forms stay out. Parse and display round-trip, `c.333garbage` is refused, and the type is a listed addition for the BioData switch (0459).
- Proof: a case over a recorded ClinVar reply finds PTEN c.333G>T as VariationID 3669092 and an intronic case finds its record, both failing on the base build; a malformed coding change is refused.
- Changes: the variant query type, the ClinVar adapter, help, cases and baseline.
- Defers: nothing.
