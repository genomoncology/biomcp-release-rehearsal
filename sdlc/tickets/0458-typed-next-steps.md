# 0458 — Build next steps from typed constructors in the query layer

Status: COMPLETE.
Landed: 967a9ff51c37e87af41d9633114f14a286a2d173

Milestone: 5

Depends on: 0419

## Outcome

Every next step is built from a checked query constructor, and a constructor that fails becomes a problem instead of a dropped step. `next::finish` in the query layer applies the order and caps that `command/steps.rs` applies today. Each step carries its target source from its entity's `next.rs`, and `finish` drops steps for disabled sources for every entity. `Action::NetworkHelp` holds the `help network` steps. The binary only spells steps and adds its global prefixes.

- Why: eleven query files build steps with `json!` and `from_value(..).ok()`, so a renamed field drops a step silently. The binary orders and caps steps and filters disabled sources only for article and author.
- Starts from: `pgx/next.rs`, `gwas/next.rs`, `gwas/get/next.rs`, `disease/seer.rs`, `disease/ols.rs`, `adverse_event.rs`, `diagnostic/next.rs`, `pma.rs`, `clearance.rs`, `companion.rs`, `study/page.rs`, the seven `run` copies, `crates/biomcp/src/command/steps.rs`, `command/step_sources.rs`.
- Rules: `crates/biomcp-query/clippy.toml` disallows `serde_json::json` and `serde_json::from_value`, with one reasoned `#[expect]` in `store/pool.rs`. No table maps requests to sources; each step names its own. The filter removes steps only and adds no entry to `sources`.
- Proof: a case with a disabled source on an entity other than article or author shows no step for that source, and it fails on the base build; a planted `json!` in an entity module fails clippy; the reply diff lists only the cases this ticket names. Reply diff scope also holds one added next Problem (value.unknown, field next, source myvariant) per row whose native coordinate lacks representations.build, in tests/cases/variant/filters/condition.yaml (1), filters/review-status.yaml (3), search-Arg117Gly.yaml (1), search-L858R.yaml (1) and search-region.yaml (2), with rows, counts, status, actions and existing problems unchanged; a byte cap rises only by the measured added Problem bytes (m17251).
- Changes: the files above, `crates/biomcp-query/src/next.rs`, `crates/biomcp-query/clippy.toml`, `crates/biomcp-types` query constructors.
- Defers: nothing.
