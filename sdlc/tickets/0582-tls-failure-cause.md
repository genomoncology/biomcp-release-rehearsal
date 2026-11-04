# TLS failures name a short, safe cause

Status: COMPLETE.
Landed: 27c4cd8ce0eba5b592e6bfdce62892579c8e9179
Milestone: 5

## Outcome

When a TLS connection fails certificate or handshake checks, BioMCP keeps the reason `tls` and adds one short, safe cause to the error detail and the log: unknown issuer, CA used as end entity, name mismatch, expired, not yet valid, revoked, or other.

- Why: the hosted team's local pacing proof failed every call with reason `tls` and recovery `ca_bundle` (job j26457, m20757) and could not tell which check rejected the certificate. `crates/biomcp-fetch/src/tls.rs` `tls_io` maps every `rustls::Error` to one fixed message. Hosted deployments with a CA bundle hit the same blind spot. Not a launch blocker.
- Starts from: `tls_io` and the `Error::Tls` mapping in `crates/biomcp-fetch/src/tls.rs`, and the `rustls::Error::InvalidCertificate(CertificateError)` variants it receives.
- Rules: the reason and recovery stay unchanged. The cause is a fixed word list, never the certificate, chain, host list or raw library text.
- Proof: an installed-binary case against a local TLS fixture whose certificate is the CA itself, or whose name does not match, shows reason `tls` with the matching cause; a valid fixture still succeeds.
- Changes: `crates/biomcp-fetch/src/tls.rs`, its error detail, and one case.
- Defers: nothing.
