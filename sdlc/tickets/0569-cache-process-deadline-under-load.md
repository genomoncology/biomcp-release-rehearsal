# Cache process test deadline under load

Status: COMPLETE.
Landed: 06a0e68bfdb5a9a5721ee9c7b71d06e2c5729882
Milestone: 5

## Outcome

`cache_process::concurrent_first_opens_and_distinct_writes_survive_into_a_new_process` passes reliably on a loaded machine, or fails only for a real cache defect, with its cause named.

- Why: job j25631 on Linux ARM64 (the Linux source inventory run) failed this test in `make test`: a child at `crates/biomcp-fetch/tests/support/cache_process.rs:248` got a PubMed DeadlineExceeded inside its 10-second leg deadline, and the parent panicked at line 134. Linux x64 passed it in the rehearsal. A failure under load is evidence, not noise: either the stub server or a cache lock stalls a child, or the deadline is too short for a busy machine.
- Starts from: `crates/biomcp-fetch/tests/support/cache_process.rs`, the test that calls it, the cache open and write path it exercises, and the retained j25631 log.
- Rules: bounded diagnosis first: reproduce under load (for example several copies in parallel with other jobs) and record where the child spends its time. If a lock, stub or cache wait stalls the child, fix that. Raise the deadline only if the diagnosis shows the work is honest and merely slow, and then say why in a comment. Do not retry the test or mark it flaky.
- Proof: a loaded reproduction that failed before and passes after a fixed number of repeated runs, with the count recorded; the routine gate passes.
- Changes: the test support, or the cache path if it is the cause.
- Defers: the Linux release clean source ticket, which this does not touch.
