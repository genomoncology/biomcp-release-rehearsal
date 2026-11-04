# 0473 — Carry CIViC molecular profiles with source-native members and expressions

Status: COMPLETE.
Landed: f8699f19a556fa39b724f013ad3cfbe4bb1ccba0

Milestone: 5

Depends on: 0478

## Outcome

A CIViC molecular profile becomes a 1.0 value that keeps its profile ID, its member references as CIViC gives them, and its Boolean expression, so every profile in the monthly release parses without inventing an allele identity. The BCR::ABL1 fusion profile (molecular profile 1, variant 1) and a real AND, OR or NOT profile are the witnesses.

- Why: the CIViC sweep (0460) found that `VariantRef::Profile` accepts only allele or protein members with no Boolean grouping, so profile 1 cannot map (m14304 ask 1). The 1.0 data model freezes at the tag.
- Starts from: `VariantRef::Profile` and the CIViC adapter in `crates/biodata` and `crates/biomcp-adapters/src/civic`, and CIViC's molecular profile model: `molecular_profile_id` names the profile, `variant_ids` and the variant URLs name its members, and `name` is the generated Feature-Variant expression with AND, OR, NOT and nested parentheses ([CIViC docs](https://docs.civicdb.org/en/latest/model/molecular_profiles/name.html)).
- Rules: a member is a CIViC variant reference (`civic:` variant ID with its feature), which may also carry an allele or protein form when CIViC states one; nothing derives an allele from a label. The expression is parsed into a tree over those members and also kept as source text. A single-variant profile keeps its current JSON form. New cases are listed additions for the BioData switch (0459). A profile alone says nothing about same-allele phase. Witness (m14719): the native API profile 6153, a real AND tree with unary NOT children and all member and feature IDs, is the code proof for Boolean expressions; the exact source `rawName` is required to check an expression against its tree. A parenthesized mixed AND/OR witness is not required here. Schema leaves (m15043): the schema for profile members and source forms states only the forms admission accepts, so members are not declared as nested profiles or compound forms; only the expression tree is recursive.
- Proof: profile 1 parses with one fusion member; a nested Boolean profile from the monthly release parses into the tree and writes back its source text; an expression naming a variant missing from `variant_ids` is refused with a named reason; existing single-variant replies are unchanged in the reply diff.
- Changes: the BioData profile type, the CIViC mapping, schemas, cases and the baseline.
- Defers: same-allele phase from other sources.
