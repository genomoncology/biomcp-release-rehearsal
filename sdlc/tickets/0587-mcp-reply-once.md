# MCP replies carry the payload once, and the tool states its call shape

Status: COMPLETE.
Landed: beee5a1d787850e2717918709811574a6df5dbf4
Milestone: 5
Depends on: 0586

## Outcome

Each MCP tool reply carries its JSON once wherever the client allows it, and the `biomcp` tool's description and input schema state the call shape plainly in far fewer characters.

- Why: experiment 513 (2026-11-03) found every MCP reply carries the same JSON twice, as text and as `structuredContent` (`crates/biomcp/src/mcp/protocol.rs` line 100). In Code Mode that doubled or tripled the cost of each reply. The tool description is about 4,400 characters plus a 4,700-character output schema, and agents guessed its argument shape wrong. With the call shape stated and replies read once, the MCP route came level with the shell. Ian approved fixing the general causes before 1.0 (2026-11-03).
- Starts from: `protocol.rs`, `output_schema.rs`, the tool declaration, and `crates/biomcp/src/mcp/http/call.rs`.
- Rules:
  - The MCP spec says a tool that returns structured content SHOULD also return the serialized JSON as text, and a text-only client would see an empty reply. So reply form is a setting with three values: both copies, structured only with a one-line text note, or text only.
  - The hosted default stays both copies until the hosting team's client check confirms every hosted client reads `structuredContent` (m22768). The local default is chosen by a recorded check of the clients we can run locally; if any of them reads text only, the local default also stays both copies.
  - The input schema and description name the exact argument fields with one short example. The output schema is kept but trimmed to what clients use.
- Proof: installed-binary MCP cases for each reply-form setting show the expected content blocks; the description and schema sizes are recorded before and after; one case shows a call written from the description alone succeeds.
- Changes: `crates/biomcp/src/mcp/`, configuration, and cases.
- Defers: nothing.
