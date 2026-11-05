# Typed calls are validated by the CLI parser and fail as recoverable tool errors

Status: OPEN.
Milestone: 5
Depends on: 0624

## Outcome

A typed call refuses only unknown keys and non-object arguments up front; every other value is spelled into CLI words and parsed by `grammar::parse`, and a parse failure is returned as an `isError` tool result holding BioMCP's usage envelope (argument, valid values, exact next call as a typed call).

- Why: `records/2026-11-05-typed-mcp-tools-review.md` item 3. Invalid arguments get bare JSON-RPC `-32602`, which models cannot recover from and the spec classes as a tool execution error; a string `"10"` for `limit` is refused though the CLI accepts it.
- Starts from: `parse` in `crates/biomcp/src/mcp/typed.rs`, `crates/biomcp/src/mcp/typed/validation.rs`, `admit` in `crates/biomcp/src/mcp.rs`, and the HTTP call path in `crates/biomcp/src/mcp/http/call.rs`.
- Rules: Branch only: build and land on `mcp-typed-tools`, never main; the frozen 1.0 candidate is untouched. Accepted input is exactly what the CLI accepts. Delete the typed validator's value rules rather than keeping two.
- Proof: installed typed cases: a wrong enum, a string integer and an unknown key each return an `isError` result with the usage envelope and a typed next call; `"10"` for `limit` succeeds; stdio and HTTP agree.
- Changes: `crates/biomcp/src/mcp/typed.rs`, `crates/biomcp/src/mcp/typed/validation.rs`, `crates/biomcp/src/mcp.rs`, `crates/biomcp/src/mcp/http/call.rs`, `crates/biomcp/tests/runner/typed_tools.rs`; the parse path and cases.
- Defers: nothing.
