# 0499 — Bind coverage reuse to the embedded skill files

Status: COMPLETE.
Landed: ccf093cd6d0300b347d08053d1a4c33c850d51ac

Milestone: 5

## Outcome

`scripts/coverage.sh` includes the root `skills/` files in the input hash that binds a coverage collection to its sources, because `crates/biomcp/build.rs` embeds them in the binary. A change to a skill file then invalidates a stored collection.

- Why: the 0456 developer's audit (m16217 ask 5) found the hash lists `crates`, `tests`, the Cargo files, `scripts/coverage.sh` and `sdlc/features.md`, but not `skills/`, which `embed_skills` compiles in.
- Starts from: `inputs()` in `scripts/coverage.sh`, `embed_skills` in `crates/biomcp/build.rs` and `scripts/test_coverage.py`.
- Rules: add the path to the existing list; check whether `build.rs` embeds any other file outside the listed paths and add it too; no other change.
- Proof: a `scripts/test_coverage.py` case changes a skill file after collection and shows `report` refusing; it fails on the current script.
- Changes: `scripts/coverage.sh` and its test.
- Defers: nothing.
