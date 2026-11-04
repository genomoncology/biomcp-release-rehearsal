# Hosted modes follow Ian's 2026-11-04 ruling exactly

Status: OPEN.
Milestone: 5

## Outcome

In `commercial`, a `noncommercial` or `licensed` source is enabled only when the caller sends the key that source requires, never by a key that only raises rate limits; and a hosted deployment sets the highest mode it serves, refusing a higher declared mode with an error that names the modes it allows.

- Why: `decisions/2026-11-04-hosted-source-modes.md`. `restrict` in `crates/biomcp/src/command/hosted_use.rs` enables a disabled source in `commercial` when the caller sends a key for it whether the key is `KeyUse::Required` or `KeyUse::Optional`, so a Semantic Scholar key (`crates/biomcp-adapters/src/semantic_scholar/mod.rs`, optional, `noncommercial`) turns on the Semantic Scholar API for a commercial caller. And any hosted caller can send `x-biomcp-use: academic` and reach every configured source, while Ian rules that the hosted solution never uses a source that is not safe for commercial use.
- Starts from: `DeclaredUse` and `restrict` in `hosted_use.rs`, the header parse in `crates/biomcp/src/mcp/http.rs` and `http/wire.rs`, and the hosted configuration in `crates/biomcp/src/config.rs` and `config/file.rs`.
- Rules:
  - In `commercial`, only a source whose key is `Required` and supplied by the caller is enabled; an `Optional` key still raises limits on sources the mode allows, and never enables one it does not.
  - A setting for the highest hosted mode, `restricted`, `commercial` or `academic`, defaulting to `academic` so self-hosted deployments keep today's behaviour; the hosting team sets `commercial`. A header above the ceiling is refused, never silently lowered, with the error naming the allowed modes. A request with no header still gets `restricted`.
  - Local CLI and stdio are unchanged. Keys never appear in replies or logs.
  - Error text and the new setting are the allowed exception to the command freeze; the release notes name both.
- Proof: installed-binary hosted cases: `commercial` with a Semantic Scholar key leaves the Semantic Scholar API disabled while openFDA with a key still answers; `commercial` with an OncoKB key still attempts OncoKB; with the ceiling at `commercial`, `academic` is refused naming `restricted` and `commercial`; with the default ceiling, `academic` behaves as today.
- Changes: `crates/biomcp/src/command/hosted_use.rs`, `crates/biomcp/src/mcp/http.rs`, `crates/biomcp/src/mcp/http/**`, `crates/biomcp/src/config.rs`, `crates/biomcp/src/config/file.rs`, `tests/cases/mcp/**`; the commercial key rule, the ceiling setting and its error, and cases.
- Defers: nothing.
