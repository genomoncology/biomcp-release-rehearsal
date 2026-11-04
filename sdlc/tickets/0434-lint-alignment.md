# 0434 — Align lint settings and guard against dead code

Status: COMPLETE.
Landed: 092b993f93213fc61dcb1b31c3baad26536604e1

Milestone: 5

Depends on: 0425, 0407, 0408, 0413

## Outcome

The internal crates deny `unreachable_pub`. The eight never-built error messages leave the schemas. Lint settings match the written Rust standard: one shared clippy base checked by the ratchet, interior-mutability types denied with item-level reasons, `allow_attributes` denied, and the bool-parameter and cast rules either enforced or removed from the standard. A gate builds biodata and biomcp-types without the `schema` feature, or the feature is removed. Strings split mid-word are rejoined. Dependency duplicates move to workspace dependencies.

- Why: the pre-1.0 code quality review `sdlc/records/2026-10-28-code-quality-review.md`. Split from 0425 so pure removals can land while this waits for the restructuring design review.
- Starts from: `crates/*/clippy.toml`, `crates/biomcp/tests/ratchets`, `crates/biomcp-types/src/response/error.rs`, the Cargo manifests and the Rust standard.
- Rules: error message removal takes a schema major bump if the compatibility check calls it breaking (ruling 6). It waits for 0407, 0408 and 0413, which edit `error.rs` and `error.json`. `ErrorMessage` stays public. `map_err_ignore` stays allowed, because many sites drop inner errors on purpose under architecture 8.5. Existing `#![allow]` attributes, the generated AlphaGenome code included, become `#[expect]` with reasons.
- Guard ruling (2026-10-28): the token-based public-use ratchet is dropped, because a name match cannot prove an item is used. Instead deny rustc's `unreachable_pub` lint in biomcp-fetch, biomcp-query, biomcp-adapters and the binary where the change is mechanical. biodata and biomcp-types are public libraries, so items unused by BioMCP are not dead there; no separate guard ticket.
- Proof: a planted `pub` item unreachable from an internal crate root fails the build; planted lint drift fails the shared-base check; the gate builds without the feature.
- Changes: the files above.
- Defers: nothing.
