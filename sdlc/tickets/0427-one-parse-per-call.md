# 0427 — Build config and grammar once per process

Status: COMPLETE.
Landed: 6afdca5ca7b6b39acee32d0b28231ea50a48237e

Milestone: 5

Depends on: 0421

## Outcome

The binary reads its config file once and builds its clap tree once per process, in `main` and in the MCP server's setup, and passes both by reference. Each reply is encoded once, with no serialize-then-reparse step.

- Why: `config.rs` rereads the config file on each call, at lines 13, 63, 67, 237 and 250. The grammar rebuilds its root on each parse. `encode` serializes JSON and parses it again.
- Starts from: `crates/biomcp/src/config.rs`, `grammar.rs` `parse`, `main.rs`, `crates/biomcp/src/mcp/`, the binary's `Reply` implementation from 0421.
- Rules: no `OnceLock` or `LazyLock`, which `crates/biomcp/clippy.toml` disallows. No test hook in product code. A ratchet allows `config::file::read_file` and the grammar tree builder to be called only from `main.rs` and the MCP server setup. Key writes may reread the config file under the existing key-write lock to keep concurrent writers and field order; every other read, key listing and MCP read uses the one process snapshot, and the ratchet names that persistence owner as the only exception (m17541). MCP keeps its per-call credential reload from 0199 and 0101, so a key set, replaced or removed from the CLI reaches a running server; static settings and the clap tree stay one process snapshot (m17603). A server whose static transport setup failed rereads settings on later calls until the first successful setup, then keeps that snapshot fixed (m17654).
- Proof: a planted call to `read_file` in a command module fails the ratchet, and the base build fails it at today's call sites; the reply diff is empty; one recorded timing of 200 calls in one MCP server process goes in the ticket record.
- Changes: the files above.
- Defers: nothing.
