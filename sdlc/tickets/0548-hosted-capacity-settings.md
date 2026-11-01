# 0548 — Make hosted call and connection caps two settings

Status: OPEN.

Milestone: 5

## Outcome

The hosted HTTP server takes its tool-call cap and its connection cap from two settings, each a `config.json` key with a flag of the same name. Unknown keys stay refused. The default stays 8 for both, and an operator can raise connections above calls so list calls still get a connection while tool calls are held.

- Why: the hosted team's capacity review of main 692bded (m18618, Ian approved the request) found that `MAX_CONNECTIONS = 8` in `crates/biomcp/src/mcp/http.rs:18`, shared by `http/call.rs:58`, sets the hosted ceiling for both connections and tool calls. They start at 32 calls and 48 connections per process. Ian ruled that every limit is configurable in the config file, following the 2026-10-23 settings ruling. The hosted service goes live by 2026-11-30.
- Starts from: `mcp/http.rs`, `mcp/http/call.rs`, `config/file.rs` and the `/health` and 503 behavior from 0530.
- Rules: two settings, one for tool calls in flight and one for open connections, named the same in `config.json` and on the command line. A connection cap below the call cap is refused at startup with a named error. `MAX_READERS` follows the connection setting. `/health` stays outside both caps. No reply changes.
- Proof: tests with a call cap of 2 and a connection cap of 4 show that a third call waits or is refused as today while `tools/list` on a fourth connection answers; a fifth connection gets the fixed 503; the 0530 tests pass at the default; a config file and a flag each set the value, and an unknown key is refused.
- Changes: the hosted HTTP server, the settings file and flag parsing, their tests and the configuration reference docs.
- Defers: per-user caps, which the hosted service enforces in front of BioMCP.
