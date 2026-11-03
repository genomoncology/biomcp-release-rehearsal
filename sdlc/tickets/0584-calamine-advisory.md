# Spreadsheet reader takes the calamine security fix

Status: COMPLETE.
Landed: 04ec714e94fbdd8b646da66031958cdb511d3269
Milestone: 5

## Outcome

BioMCP builds on calamine 0.38.0 or later, so a crafted spreadsheet supplement can no longer panic or exhaust memory while parsing, and the dependency advisory check passes again.

- Why: landing j27757 for 0583 at c099a3814 failed because the newly published cargo-deny advisory rejects calamine 0.37.0 (crafted OLE2 parsing panic or out-of-memory, fixed in 0.38.0, tafia/calamine PR 762), m22544. Every landing fails until this is fixed, and 1.0 must not ship a known advisory.
- Starts from: the workspace pin at `Cargo.toml` line 58 and its use in `crates/biomcp-adapters`.
- Rules: release-only work under the command freeze. Replies, help, skill and commands stay byte-identical; the reply diff for the workbook cases shows no change. No guard waiver. If 0.38 changes the API, adapt the adapter code without changing its output.
- Proof: the dependency advisory check passes; the existing supplement workbook cases pass unchanged on the installed binary.
- Changes: `Cargo.toml`, `Cargo.lock`, and adapter code only if the API moved.
- Defers: nothing.
