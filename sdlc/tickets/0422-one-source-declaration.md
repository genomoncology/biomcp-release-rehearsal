# 0422 — Declare each source once

Status: COMPLETE.
Landed: 981f3dc95ec8d2f21d2cae926318ef1bf1ff0874

Milestone: 5

Depends on: 0421

## Outcome

Each adapter has one `SourceDecl` holding its spec, display name, probe, bundles, components, auxiliary specs and guidance text. An exhaustive `decl(source)` derives the registry, health probes, bundle state, the key list, sync names and help. `Action::KeyHelp` carries a `SecretName`. HGNC re-keying leaves the transport through a `rekey` function each bundle may declare, and the store owners pass it to both the import path and the open-time migration. The source-name ratchet's allow list is empty.

- Why: source facts live in a dozen tables with `_` fallbacks. A source with no listed probe reports as rejected. The transport holds HGNC logic at two call sites.
- Starts from: `crates/biomcp-adapters/src/registry.rs`, `registry/names.rs`, `guidance.rs`, `crates/biomcp-fetch/src/secret.rs`, `cache_hgnc.rs`, `bundle/transfer.rs`, `cache/schema.rs`, `crates/biomcp/src/command.rs` `key_help` and `bundle_source`, `grammar.rs` `BUNDLE_HELP` and the sync match, `spell.rs` bundle names, `crates/biomcp-query/src/run.rs` `Bundle`, `store/` owners.
- Rules: restructure (c). Quotas stay as they are. Bare `sync` keeps HGNC as an explicit default. Ruling 7 accepts one decode pass for re-keying. If store generations 1 to 3 never shipped in a release, delete the migration instead and record the release check. `SecretName::source()` leaves the transport. `wildcard_enum_match_arm` is denied in `decl.rs`, the health module and `biomcp_query::sources`. A benchmark ticket that adds a source after this lands adds a declaration.
- Proof: a planted `Source` variant with no declaration fails to compile; a case imports a generation-3 HGNC cache and resolves a gene by UniProt key, and it fails on a build without re-keying; the source-name ratchet passes with an empty allow list and fails on a planted `hgnc` in a transport file; the reply diff is empty.
- Changes: the files above and one declaration per adapter folder.
- Defers: nothing.
