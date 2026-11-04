# 0502 — Add a hosted mode that refuses host-touching MCP calls

Status: COMPLETE.
Landed: 1de20120ba798aaca0bcd2b85f3ced7ee1b9121a

Milestone: 5

## Outcome

`biomcp serve --hosted` refuses, on every transport, each MCP call that writes files on the host (`--save` for article and trial documents), reveals key or store state (`health`), or changes the host's stores (`sync`, `store clean`, `--fresh`, `key`). Each refusal is a typed usage error that names hosted mode.

- Why: Ian approved a shared hosted remote MCP server for 1.0, live before 2026-11-30 (m16469); a code audit at 79883e3ba checked each outcome against this repo. Today HTTP already refuses saves, sync, store clean, `--fresh` and `key` (`command.rs` 363-393, `command/mcp_policy.rs`), but stdio allows them and `health` reports which keys are configured on both transports.
- Starts from: `crates/biomcp/src/grammar/local.rs`, the grammar globals, `command/mcp_policy.rs`, `mcp.rs` and `command.rs` `allow_save`.
- Rules: one policy owner in `mcp_policy.rs` reads the hosted flag; without `--hosted` behavior stays as it is; the shared request cache and query pools stay, because they are keyed by request, not by caller; no public text names any private project.
- Proof: a case per refused call over HTTP and stdio with `--hosted` fails on the base build; the same calls without the flag keep today's result.
- Changes: the serve grammar, the MCP policy, cases, help and docs.
- Defers: per-request keys (0503).
