# 0421 — Declare each command once and type every usage error

Status: COMPLETE.
Landed: 8495305fb85f6490f3070217e29c34aaff3a6d27

Milestone: 5

Depends on: 0420, 0426, 0444

## Outcome

One `commands!` table in `biomcp-types` generates `Request`, paths, schema IDs, the command list and a `Rewrite` enum for split and rewritten paths. The binary's `grammar/rewrite.rs` holds one exhaustive match per direction for those rewrites. Grammar, spelling, help, the MCP tree and the query rows of `contracts!` read the table. The query layer returns each reply through a generic `Reply` visitor that ties the body type to the request type. The binary implements it once for envelopes and once for saves, so `QueryBody`, the pairing match and its mismatch errors go. Usage errors are a typed `UsageError`, and checked constructors carry their refusal messages, so argv rescanning goes.

- Why: each command is described in eight hand-kept places. A body can pair with the wrong request at runtime. Usage errors travel as bodies, and messages come from rescanning argv text.
- Starts from: `crates/biomcp-types/src/query/request.rs`, `response/gene.rs` `QueryBody`, `response/error.rs`, `crates/biomcp/src/grammar.rs`, `grammar/queries.rs`, `grammar/entity.rs`, `grammar/gwas.rs`, `crates/biomcp/src/spell.rs`, `command.rs`, the per-entity encode modules, `crates/biomcp/src/save/`, `crates/biomcp-query/src/run.rs`, `validation.rs`, `crates/biomcp-types/tests/schemas.rs`.
- Rules: restructure (b). The splits and rewrites are `get pgx`, `get drug`, `get gwas`, the `search gwas` positional rename and the `query_option` injection for `search cell-line` and `search author`. Spelling's default-dropping for `GetPgx` and phenotype lives in `grammar/rewrite.rs`. `ErrorMessage` stays public. Schema IDs are whatever 0444 leaves. Reply rows of `contracts!` come from 0439's registry, and this ticket adds no second registry. Rebase on 0437's reply derives. `wildcard_enum_match_arm` is denied in `run.rs`, `spell.rs` and `grammar/rewrite.rs`. Land as three commits on one branch: typed errors, the table, the visitor.
- Proof: a table test fails on a planted second row for an existing path without a shared `via` name, and on a schema ID with no file; a planted body type that does not match its request fails to compile; a planted new `via` name without a rewrite arm fails to compile; the reply diff against the base is empty, which covers the `--facility` with `--lat` and gwas threshold messages.
- Changes: the files above and `crates/biomcp-types/src/query/commands.rs`, `crates/biomcp/src/grammar/rewrite.rs`.
- Defers: building the grammar once (0427).
