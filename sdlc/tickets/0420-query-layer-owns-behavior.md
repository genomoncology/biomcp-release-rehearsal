# 0420 — Make the query layer own behavior and adapters own wire syntax

Status: COMPLETE.
Landed: 307b71667e309d60e9819f33780218f07c4eb411

Milestone: 5

Depends on: 0458

## Outcome

Adapters build every provider query string and keep raw types private. The query layer owns capability checks, disease section support, SEER query preparation and selector errors, cell line database checks and label bounding. The binary no longer depends on `biomcp-adapters`, and `biomcp-query` exports only what the binary uses, with source facts behind one `sources` module. The value audit covers the query layer.

- Why: PubMed, Europe PMC and ctgov syntax sit in the query layer. Seven adapters export raw modules. The binary imports adapter modules and holds source rules in `drug_label.rs`, `disease.rs` and `unsupported.rs`.
- Starts from: `crates/biomcp-query/src/article/term.rs`, `article/sources.rs`, `trial.rs`, `drug/native/identity.rs`, the raw re-exports in `chembl`, `opentargets`, `cbioportal`, `drugsfda`, `openfda_label`, `gwas`, `semantic_scholar/author`, `crates/biomcp/src/command/drug_label.rs`, `command/disease.rs`, `command/unsupported.rs`, `crates/biomcp/Cargo.toml`, `crates/biomcp-query/src/lib.rs`, and the binary's adapter imports listed in restructure (a).
- Rules: placement and export list in restructure (a). Ruling 2: the label bound measures the label section's own serialized bytes, while the old bound measured the whole envelope, spelled steps and `--config` prefix included. The constant keeps existing label fragment counts. Changed label pages go to the PM. `crates/biomcp-query/clippy.toml` disallows `biomcp_fetch::Arg`, `Endpoint` and `Input`. `deferred_keys_tests.rs` moves to `crates/biomcp-adapters/tests/`. The audit from 0418 extends to `crates/biomcp-query/src`. Refusals keep today's envelope codes until 0421 types them.
- Proof: a planted `biomcp_adapters` import in the binary fails to build; a planted `Arg` in the query layer fails clippy; a planted `.ok()` drop in a query module fails the audit; the reply diff lists only the label pages handed to the PM.
- Changes: the files above.
- Defers: the command table (0421), source declarations (0422).
