# 0506 — Fit the MCP tools to remote clients

Status: COMPLETE.
Landed: f542feb02acae73b5f13233c15924bdcceda773d

Milestone: 5


## Outcome

Every MCP tool declares its JSON Schema dialect and correct annotations, and every tool result stays under 150,000 characters. The `biomcp` tool carries `destructiveHint: false`, and `readOnlyHint: true` wherever saving is off (HTTP today, and hosted mode once 0502 lands; whichever of the two lands second wires the hosted case). A reply that would pass the cap returns a typed error that names the cap and the narrowing options, never silently cut text.

- Why: Ian approved a shared hosted remote MCP server for 1.0, live before 2026-11-30 (m16469); a code audit at 79883e3ba checked each outcome against this repo. BioMCP lists three tools with root `required` arrays and a 20 second budget, which already fit, but no tool declares `$schema`, `biomcp` has no annotations, and `protocol.rs` `outcome` serializes each value twice with no size cap.
- Starts from: `crates/biomcp/src/mcp/protocol.rs`, `rpc.rs`, `research.rs` and the output budgets.
- Rules: annotations depend on transport and hosted mode; the cap counts the whole serialized result; the output schemas stay valid; no new tools.
- Proof: a case lists tools over HTTP and checks the dialect and annotations; a case whose reply passes the cap returns the typed error; both fail on the base build.
- Changes: the MCP protocol layer, research tools, cases and the schema baseline.
- Defers: nothing.
