# 0532 — Trim dependency features and fail the gate on unused dependencies

Status: COMPLETE.
Landed: 17b049bcb44d491421cada7d8cf5131edcadb9ec

Milestone: 5

## Outcome

The release graph drops the features and crates BioMCP does not use, and `make lint` fails on an unused dependency.

- Why: Ian's shared ruling `instructions/decisions/2026-10-31-dependencies-are-few-chosen-and-used.md` in the agents repo, and release review B7 and the dependency table, weighed 2026-11-01. tonic's `channel` feature is unused, because the transport opens its own h2 connection and TLS, and `tls-ring` only enables a crate that is already direct. url pulls about 20 ICU crates through `idna`, though publisher hosts are ASCII. ureq's `rustls` feature pulls `webpki-roots`, which BioMCP's own verifier never uses and which needs a CDLA exception. `deny.toml` carries never-used exceptions for `borrow-or-share 0.2.4`, `foldhash 0.1.5` and `foldhash 0.2.0`.
- Starts from: `crates/biomcp-fetch/Cargo.toml`, `crates/biomcp-fetch/src/tls.rs`, `deny.toml`, `release/toolchain.json` and the `lint` target.
- Rules: turn off tonic's `channel` and `tls-ring`; keep url and pin `idna_adapter = "=1.0.0"` so hosts stay ASCII-only; switch ureq to `rustls-no-provider` with the platform verifier; delete exceptions that no longer match. Install `cargo-machete` at a pinned version through `release/toolchain.json` and the existing owned installer, and run it in its default mode, never `--with-metadata`. A confirmed false positive goes in its ignore list with a reason. ctrlc, base64 0.23, rpassword, serde_ignored and httpdate stay.
- Proof: `cargo tree` shows the ICU crates, `webpki-roots` and the tonic channel stack gone; `cargo deny` passes with no unused-exception warning; `make lint` fails when an unused dependency is added on purpose; the landing gate passes with an unchanged case count.
- Changes: manifests, `Cargo.lock`, `deny.toml`, `release/toolchain.json` and the Makefile.
- Defers: nothing.
