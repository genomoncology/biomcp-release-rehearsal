# 0510 — Keep all-refused cBioPortal mutation pages out of the cache

Status: COMPLETE.
Landed: 090a9b8bc5ce8487202b411ada9850a69786f758

Milestone: 5

Depends on: 0496

## Outcome

The cBioPortal mutations read in `adapters/cbioportal/mod.rs` follows 0496's admission rule: a nonempty native mutation page whose records were all refused returns its problems and native counts and never enters the cache, and a valid native empty page still caches.

- Why: 0496's admission inventory (m16827). `mutations` caches a nonempty `RawMutations` page whose mapped records are all refused, and it sits outside 0496's named owners and 0508's bundle scope.
- Starts from: 0496's admission check, `RawMutations` and the variant recurrence route that reaches it.
- Rules: 0496's rules; no new paging or cache layer.
- Proof: one case over recorded replies through the recurrence route shows a second call making a fresh request after an all-refused mutation page; it fails on the base build. A valid empty page still caches.
- Changes: the cBioPortal mutations adapter, one case and baseline.
- Defers: nothing.
