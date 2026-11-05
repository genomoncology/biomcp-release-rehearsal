# Typed tool schemas keep their own descriptions and fit a size budget

Status: OPEN.
Milestone: 5
Depends on: 0623

## Outcome

Each typed input property keeps its own description, cut to 80 characters; nested `items` and `anyOf` branches carry none; optional fields drop nullable branches; enums above a configurable size (default 30) become a described string; and the output schema is left out of typed `tools/list` by default, behind a setting. Every build reports each tool's size and fails over budget.

- Why: `records/2026-11-05-typed-mcp-tools-review.md` items 2 and 5. `resolve` drops a `$ref` property's own description, so 176 of 408 properties read as generic text; inputs are about 50% and output schemas about 35% of the 130,784-character list; `search_trial` is about 8.7k, 3.0k of it the country enum.
- Starts from: `crates/biomcp/src/mcp/typed/arguments.rs`, `for_command` in `crates/biomcp/src/mcp/output_schema.rs`, and `crates/biomcp/src/mcp/typed.rs` `tools`.
- Rules: Branch only: build and land on `mcp-typed-tools`, never main; the frozen 1.0 candidate is untouched. Validation of values moves to the CLI parser in 0626, so dropping an enum never loosens what is accepted. Budget: 2,000 characters per tool for name, description and input schema, with a named exception list only where a command's required inputs cannot fit, each with its reason.
- Proof: a test proves every compact property's description equals the first line of its own schema description; a size report lists each tool and the total; the budget test fails on an oversized synthetic tool; the output-schema setting restores today's schemas.
- Changes: `crates/biomcp/src/mcp/typed/arguments.rs`, `crates/biomcp/src/mcp/output_schema.rs`, `crates/biomcp/src/config/file.rs`, `crates/biomcp/tests/runner/typed_policy.rs`; compact schemas, the setting, the size report and tests.
- Defers: nothing.
