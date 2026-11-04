# A stale offline copy answers at once while it refreshes

Status: COMPLETE.
Landed: c131b96f8a75ce1325dc3cbc6c5f1db7083766ce
Milestone: 5

## Outcome

When a live read finds a stale offline bundle, it answers from the stale copy at once, marked stale as today, and the refresh runs in the background. Only a missing bundle makes the caller wait.

- Why: hosting bakes eight offline bundles (824.8 MiB, j27986) into its image, and ClinGen goes stale after a day and HGNC after seven (m22906). Today `Leg::rows` (`crates/biomcp-fetch/src/bundle/read.rs`) runs the refresh inside the first read that finds the bundle stale, so that caller waits for the whole download within its deadline. Concurrent readers already get the stale copy, and a failed refresh already falls back to it. Not a launch blocker.
- Starts from: `Leg::rows` and `sync_for_read` in `crates/biomcp-fetch/src/bundle/read.rs`, the store's single-publisher sync lock, and the server's lifetime for background work.
- Rules: one refresh per bundle at a time, under the existing sync lock. `--fresh` still waits for a fresh copy. Offline, replay and record modes are unchanged. A one-shot CLI run that exits keeps today's behaviour, waiting for the refresh, unless the process can finish it safely after replying; `serve` refreshes in the background. A failed background refresh is logged and retried on a later read, never more than the source's pacing allows.
- Proof: an installed-binary `serve` case with a stale local fixture bundle shows the first read returns the stale rows, marked stale, before the fixture download completes, and a later read returns the refreshed rows; a missing bundle still waits; `--fresh` still waits.
- Changes: `crates/biomcp-fetch/src/bundle/read.rs`, the serve background worker, and cases.
- Defers: nothing.
