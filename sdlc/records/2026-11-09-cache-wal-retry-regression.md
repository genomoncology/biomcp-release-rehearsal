# WAL retry removed during first-open serialization

Ticket: `0777-retry-owned-cache-wal-transition-within-budget.md`.

## Failed belief and evidence

The failed belief was that the sibling setup lock and SQLite busy timeout make an explicit WAL retry unnecessary.

0639 established the contrary: a journal-mode read-to-exclusive upgrade can return SQLite Busy without invoking the busy handler. Its retained installed proof observed five such errors recover after retry. The 0748 change (`cb7065ffb`) introduced sidecar setup serialization and a shared deadline, but removed the WAL retry. The sidecar is advisory and explicitly permits an unlocked path when the filesystem cannot provide locks; it also cannot serialize nonparticipants. The exact reason that a particular installed run did not serialize is not established by its retained syscall-free witness.

Cold job j36136 at product `2d2dbda17` returned Busy from the WAL update with primary/extended SQLite codes 5/5, seven milliseconds in the WAL step, zero retries and 141 milliseconds for the command. Seven sibling commands succeeded. This is the same user-visible failure mechanism as 0639, not evidence that its five-second budget was exhausted or that the rusqlite 0.41.1 dependency update (libsqlite3-sys 0.39.1) itself introduced the regression. The 37-file witness is retained on Yellow under `data/biomcp-redux-evidence/0640/2d2dbda17ca2/installed-cache-first-open-f8h0d42e`.

## Smallest correction

Restore Busy retry only around the complete idempotent journal-mode query/update. Finalize each failed attempt before waiting and reread the mode on retry. Keep the already-created deadline shared with sidecar acquisition, ownership checking and initialization. Other failures return immediately; ownership and initialization remain single-attempt steps. Final Busy diagnostics count only operations that actually started after the first attempt.

The deterministic concurrency regression holds a real SQLite reserved writer lock while leaving the production SQLite busy timeout enabled. The fixture releases its writer only after observing the first actual WAL Busy, before returning that failed attempt. A single-attempt implementation remains red even though the lock is then released; a retry succeeds. The held-lock counterpart keeps that state until the supplied deadline expires and asserts bounded refusal, without a timer that releases the lock. These inner tests own SQLite locking behavior; the installed eight-process witness and lasting first-open helper exercise its caller surface.

No schema, namespace, BioData pin, user deadline or foreign-file behavior changes. The correction retains sidecar serialization rather than replacing it with a new lock mechanism.

Repeated per-attempt numeric SQLite notes leave a small reserved part of their existing 4096-byte bound for the final open-failure summary, so accurate retries are not dropped after the coordinate sink fills. This changes no diagnostic fields, credential handling or cache behavior.

## Validation

Job j36247 at `739aba0a3bf24a1c7611a81181bd2362a5b7d3fd` ran the two real-lock regressions against the single-attempt implementation: both failed, with the released-writer case returning Busy and the held-writer case returning before its deadline. Job j36251 passed all 23 fetch library tests, 29 cache integration tests and 47 ratchets, plus formatting and fetch Clippy. Job j36255 passed the strengthened regressions with the configured SQLite busy timeout asserted positive.

Ending job j36266 at product and campaign-runner revision `fed364235eecf00c8b314928a1ab8e7c234f0fbb` passed formatting, fetch Clippy, both WAL regressions and 47 ratchets. This revision refreshes the same deadline between the journal-mode query and update, and tests refusal after that deadline expires. The existing artifact producer built a native macOS ARM64 development-profile wheel, installed it outside the checkout, and the unchanged installed first-open helper ran 50 rounds of eight processes: 400 successful opens, every round starting with an empty cache, zero SQLite errors and every post-exit database in WAL mode. This is targeted development-artifact evidence, not final release or Linux artifact qualification.

The installed binary SHA-256 is `d528c334a4104b4f252607dd840f9859143df907f29e19b37be81c10560b9e81`. Artifact and producer record remain on M5 under `/Users/ian/workspace/data/biomcp-redux/0777/fed364235/`. The 2001-file campaign and producer record are also retained on Yellow under `/home/ian/workspace/data/biomcp-redux-evidence/0777/fed364235/`. Source and receiver campaign manifests match SHA-256 `c67b8727008ef583beb7721e8b0d4d2684cb34b9804d48054ce268dab268c3b9` (sorted relative paths and per-file SHA-256).
