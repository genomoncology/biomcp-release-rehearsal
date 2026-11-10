# 0811 — Explain protein span coordinates in text output

Status: COMPLETE.
Landed: 630a52fc9366446ba03d71eeb363fccc85bc7d16

Milestone: 5

## Outcome

Protein feature/domain text output makes the zero-based interbase (half-open) span convention clear and distinguishes original one-based inclusive source coordinates and the one-based --position selector. CLI Markdown and MCP text expose the explanation without changing numeric JSON spans.

## Why

The reported off-by-one examples are correct conversions, not numeric corruption: native321–554 becomes320–554,2098–2584 becomes2097–2584,102–286 becomes101–286. Read-only inspection confirms schema descriptions explain this, but structural Markdown prints bare start/end fields and source_coordinates can be mistaken for the output convention. Ian asked to address these reports during qualification.

## Proof

- Extend existing synthetic protein entry/domain fixtures through CLI Markdown and MCP text. Require an explicit coordinate-convention explanation and unchanged JSON values (native10..20 becomes9..20; native20..30 becomes19..30).
- Retain the one-based --position inclusion controls. Existing native conversion assertions remain unchanged. Exercise the actual shared renderer, not a test-only formatter or string helper alone.
- Run focused owning cases and the routine landing gate. No live source call or entire release rebuild is needed for development proof.

## Starts from

- interpro/map.rs and uniprot/entry/sections_map.rs already use UncertainSpan0::from_inclusive correctly; pinned biodata subtracts one from start only. Generated protein-domains/features schemas specify zero-based interbase spans and describe source_coordinates as the convention before conversion.
- crates/biomcp/src/view/mod.rs structural rendering and view/hints.rs, existing protein_domains.rs and protein_sections/boundaries.rs cases. Do not change adapter arithmetic to match raw source starts.

## Keeps

- Existing typed numeric schemas, positions, uncertainty bounds, lengths, domain/feature identities and caller/source controls. No coordinate migration, per-source arithmetic, or contradictory duplicate convention constants. Derive or centrally own the note in the existing rendering/type path; do not alter unrelated entity output.

## Defers

- Any actual coordinate-calculation defect would need its own raw-response witness; none was found in the reported examples. Preserve the running rehearsal under its source identity and assess only affected presentation checks before selecting a new ending candidate.

## Changes

- Shared protein span presentation guidance and owning outside-in rendering cases. No provider adapter or stored coordinate changes.
