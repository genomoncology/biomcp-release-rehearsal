# 0467 — Extend the adapter value audit and file rules to every adapter

Status: COMPLETE.
Landed: da5f584d63e0eeeff985951525e1647f96ebb23b

Milestone: 5

Depends on: 0417, 0454

## Outcome

The adapter value audit sees every silent-drop form in every adapter file, and the source ratchet applies its file rules to every adapter file. Each current violation is listed as pending under its adapter, so the family tickets can land one at a time and the list can only shrink.

- Why: 0418 covers about 32 adapters and must land in whole family changes (m13852). The shared audit and cap changes need one owner before any family starts.
- Starts from: `crates/biomcp/tests/ratchets/adapter_values.rs`, `adapter_value_audit.json` and `ratchets/source.rs`.
- Rules: the audit fingerprints `.ok()`, `if let Ok`, `let Ok .. else` and `filter_map(..ok())` in adapter files, and `unwrap_or_default` in `raw.rs`, in `map.rs` and on parse results. `ratchets/source.rs` applies the 300-line cap and the `serde(default)` and `deny_unknown_fields` refusals to every adapter file. Every current hit is recorded as pending under its adapter directory; a new hit outside the pending list fails, and adding a pending entry fails. The query layer joins the audit in 0420. No adapter code changes here.
- Proof: a planted `.ok()` drop in an adapter fails the audit; a planted new pending entry fails; a planted 301-line adapter file fails; main passes with the recorded pending list.
- Changes: the ratchets and the audit file.
- Defers: the per-family fixes (0468 to 0472).
