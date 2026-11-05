# Typed tool descriptions say what the command does

Status: OPEN.
Milestone: 5
Depends on: 0623

## Outcome

Each typed description starts with the command's own one-line summary from the registry, written as a plain sentence, then the input it takes (from the declared primary positional), then each section's source, within 300 characters, with the entity's common search words in the leading words.

- Why: `records/2026-11-05-typed-mcp-tools-review.md` item 4. Descriptions start "{verb} {entity} source records." and take the alphabetically first positional, so `search_adverse_event` describes its VAERS section and `get_drug` its sections, not the drug it takes.
- Starts from: the `///` doc lines in the command registry (`crates/biomcp-types/src/query/commands.rs`), `crates/biomcp/src/mcp/typed/description.rs`, and `input_description`.
- Rules: Branch only: build and land on `mcp-typed-tools`, never main; the frozen 1.0 candidate is untouched. Summaries are declared once per command and generated into descriptions, never hand-written per tool. Internal wording such as "literal native candidates" is rewritten in the summary itself so help benefits too; help text changes stay on the branch.
- Proof: a test proves each description starts with its command summary, names the primary input, stays within 300 characters and names a source for every section; spot cases for `get_drug`, `search_adverse_event` and `search_variant`.
- Changes: `crates/biomcp-types/src/query/commands.rs`, `crates/biomcp/src/mcp/typed/description.rs`, `crates/biomcp/src/help.rs`, `crates/biomcp/tests/runner/typed_policy.rs`; summaries, descriptions and tests.
- Defers: nothing.
