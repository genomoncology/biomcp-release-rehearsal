# 0553 — Take each hosted request's declared use from a header

Status: COMPLETE.
Landed: aca2c6e83b2cde4dacbfe7c0f611be58e2cde725

Milestone: 5

## Outcome

In hosted mode each request declares its use in an `x-biomcp-use` header with the value `commercial` or `academic`. A commercial request runs with the noncommercial and restricted sources and components disabled. An academic request runs with the server's configured sources. A missing header means commercial. A source key the caller sends counts as their license and enables that source in either use. One server serves both kinds of user.

- Why: the hosted team's request of 2026-11-01 (m19057). Ian ruled that hosted BioMCP serves commercial users by default and academic users who attest on the hosted account page. Today `disable_sources` is process-wide, so one server cannot serve both. The hosted service goes live by 2026-11-30; the hosted edge strips any client-sent value and sets its own.
- Starts from: the per-request keys from 0503 (`biomcp-fetch/src/secret.rs:171`), the per-caller licensed replies from 0504, `registry::disabled_uses` and `registry::disabled` (`biomcp-adapters/src/registry.rs:48-52`), the component checks in `registry/component.rs`, and `Fetch::disabled` (`biomcp-fetch/src/lib.rs:258`).
- Rules: the header is read only in hosted mode, like the key headers. Any value other than `commercial` or `academic` gets a refusal naming the header and the allowed values, with no source called. The per-request disabled set is the server's configured set plus, for commercial, every source and component whose use is noncommercial or restricted, minus any source whose key the request carries. A key never enables a source the server's own configuration disables. A disabled source answers with the existing disabled-source refusal. Local CLI and stdio MCP are unchanged. Stored replies of a noncommercial or restricted source are never served to a commercial request.
- Proof: hosted tests show a request with no header and one with `commercial` refused by a noncommercial source and a restricted source, an `academic` request served by both, a commercial request with that source's key served, a server-disabled source refused in every case, a bad value refused, and a reply stored for an academic request not served to a commercial one.
- Changes: hosted request parsing, the per-request fetch settings, their tests and the hosted mode docs.
- Defers: none.
