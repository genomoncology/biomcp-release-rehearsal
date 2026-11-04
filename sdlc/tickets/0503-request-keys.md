# 0503 — Take each caller's source keys from request headers in hosted mode

Status: COMPLETE.
Landed: 6020a636e9a5aa9eb372ad4df3cd8c4cf21282b4

Milestone: 5

Depends on: 0502

## Outcome

In hosted mode, an HTTP MCP request may carry source keys as headers named `x-biomcp-key-` plus the config key name with underscores written as hyphens (`keys.ncbi_api_key` becomes `x-biomcp-key-ncbi-api-key`), matched without regard to case, and BioMCP uses them for that request only. `biomcp key list` adds each key's header name and tier (no-fallback or fallback) so a front end reads the list instead of copying it (m16520). Sources whose terms restrict a key to its holder (OncoKB and AlphaGenome, plus any other source whose recorded terms say so) never fall back to the server's own key; public sources such as NCBI, openFDA, NCI and Semantic Scholar may. A missing required key returns an error that tells the user to add that key in their account settings. Keys never come from tool arguments, and outside hosted mode these headers are ignored.

- Why: Ian approved a shared hosted remote MCP server for 1.0, live before 2026-11-30 (m16469); a code audit at 79883e3ba checked each outcome against this repo. Keys load only from the config file and environment into one process-wide `Secrets` (`config.rs` 257-300, `biomcp-fetch/src/secret.rs`), and `KeyUse` has no tier that forbids server-key fallback.
- Starts from: `crates/biomcp/src/mcp/http/wire.rs`, `mcp/http.rs`, `config.rs` secrets, `biomcp-fetch/src/lib.rs` and `containment.rs`, and the source declarations' `KeyUse`.
- Rules: a per-request overlay over the server keys, carried through the call context, never stored, logged, cached or echoed; the no-fallback tier is a declared source property, checked once per source; the hosting front end strips client-sent key headers, and this repo documents that requirement; secret values never appear in errors, replies or tests.
- Proof: cases over recorded replies show a header key used for one request and not the next; a no-fallback source with no header refuses even when the server has its key; a public source falls back; a key in tool arguments is refused; each fails on the base build.
- Changes: HTTP header parsing, secrets, the fetch layer, source declarations, cases and docs.
- Defers: cross-caller cache isolation (0504) and per-key pacing (0505).
