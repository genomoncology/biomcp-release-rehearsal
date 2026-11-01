# 0549 — Scale only keyed buckets by the rate share

Status: OPEN.

Milestone: 5

## Outcome

`--rate-share` scales only buckets that belong to a caller's own key. Keyless hosted buckets keep the source's full rate, because keyless limits apply per address and each hosted task has its own public address.

- Why: the hosted team's capacity review (m18618, Ian approved the request). `crates/biomcp-fetch/src/quota/limiter.rs:47` applies the share to every caller except `Caller::Process`, including `Caller::Hosted(None)`, which halves keyless capacity for nothing. The hosted team's pacing work waits on this.
- Starts from: `quota/limiter.rs`, `quota/rate_share.rs`, `quota/admit.rs` and `crates/biomcp-fetch/tests/quotas.rs:432`.
- Rules: `Caller::Hosted(Some(_))` buckets use the share; `Caller::Hosted(None)` and `Caller::Process` buckets do not. The flag's help and the configuration docs say the share applies to keyed buckets only. No reply changes.
- Proof: the existing test that scales both keyed and keyless rates becomes a test that a share of 0.5 halves a keyed bucket's rate and leaves the keyless hosted bucket at the full source rate; the rejection test for bad shares still passes.
- Changes: the limiter, its tests, the flag help and the docs.
- Defers: none.
