# 0789 — prove-refresh-worker-ownership-and-single-flight

Status: COMPLETE.
Landed: 657569166daf2bf9247d3483effce8e226919f0b

Milestone: 5

## Outcome

The existing public Fetch regressions reject removal of refresh-owner cleanup, removal of worker joining, and inversion of the running-worker single-flight guard. They prove actual worker/socket/resource completion and prompt stale replies while a refresh is held, rather than merely observing that Drop returned.

## Why

The completed mutation sample passed both shutdown tests after deleting the entire Drop method. Independent review found three consequential assertion gaps and no new runtime defect.

## Proof

- Retain j36572 as the original witness: 20 completed, 11 caught, 2 unviable, 7 survivors. Strengthen the existing held-response owner test with peer EOF and the transfer-file lock. Pre-open the locked part file and hold an external SQLite writer during owner destruction; require its file lock released before releasing SQL or HTTP. This observes worker completion as well as socket cancellation.
- Add a second ordinary stale read while the first response remains held. Require its stale reply before publication, with one download and the original snapshot retained. Use existing fixtures and event/watchdog bounds, not short timing-only assertions.
- Run the owning bundle tests and focused mechanical checks, then verify only the three exact mandatory mutations with native cargo-mutants filtering. Set its timeout above the fixture watchdogs so failed assertions are observed rather than misclassified as harness timeouts. Do not repeat the whole twenty or add a selection framework.
- Record the actual test-driver revision and prove runtime/Cargo/toolchain inputs match frozen product source 0adbae81319cbfd89066c880ced85391f7a4775a. No runtime edit or artifact relabelling is part of this ticket.

## Starts from

- `crates/biomcp-fetch/tests/bundle/shutdown.rs`, its existing server fixture, and the retained patches/logs under Yellow `biomcp-redux-evidence/0788/0adbae81319c/mutations`. Mandatory survivors are refresh.rs line 22 delete-not, line 86 no-op join and line 92 no-op Drop. The original guard/cancellation/join code is correct; the assertions are incomplete.

## Keeps

- Existing caller independence, snapshot/fencing, bounded cleanup and test failure cleanup. Prefer the existing public API/file-lock/SQLite/socket fixtures; no new clock, internal harness, production hooks, native target expansion or source behavior change. Preserve the complete original mutation report, including all four independently classified nonblocking survivors; do not call them equivalent or hide them with exclusions.

## Defers

- The two lower-consequence concurrent-publication/expiry-second coverage gaps are future work. Debug formatting and optional diagnostic deletion do not justify release-blocking string tests. All classifications remain in the qualification record.

## Changes

- Existing public Fetch bundle tests and their server fixture, plus qualification evidence. No product source or new checker.
