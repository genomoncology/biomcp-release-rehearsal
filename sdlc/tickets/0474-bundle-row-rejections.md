# 0474 — Disclose rejected bundle rows instead of failing the whole sync

Status: COMPLETE.
Landed: a34d0d9fb1d98fbda417a7d4e62b4e895487f6b0

Milestone: 5

Depends on: 0467

## Outcome

When a stored source bundle such as HGNC or ClinGen has a row that cannot be decoded or lacks every usable query key, sync keeps every valid row, and the reply's existing `problems` names each rejected row. A later sync that reuses the bundle (including a 304 reply) reports the same rejections.

- Why: strict CSV decoding aborts the whole bundle on one bad row, and row problems have no private path from parsing to the sync reply, so 0468 and 0469 cannot disclose rows with no usable key (m14304 ask 3).
- Starts from: the row parse step, `BundleInfo` and `Fetch::sync` in `crates/biomcp-fetch`, adapter `SyncResult`, and `command/sync::finish`.
- Rules: one owner for the channel: row parse returns rejected-row problems, `BundleInfo` stores them with the bundle metadata, adapters return `Mapped<SyncResult>`, and sync adds them to the existing envelope `problems`. No new public reply field and no new reporting framework. A rejection names the row number and the reason, never the row's content beyond its key.
- Proof: a bundle with one row lacking every key and one undecodable row syncs its valid siblings and names both rows; a reused bundle sync reports the same two problems; a fully valid bundle reports none.
- Changes: the fetch layer, the HGNC and ClinGen adapters, sync, cases and the baseline.
- Defers: nothing.
