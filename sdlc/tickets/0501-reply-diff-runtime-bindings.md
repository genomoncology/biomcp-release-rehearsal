# 0501 — Let the reply diff pair runtime values by binding

Status: COMPLETE.
Landed: f8f8b44e7911f735a5ea57971d47443e9920c026

Milestone: 5

## Outcome

The reply diff from 0454 treats values a run makes up for itself as equal when they pair one to one across the two builds: query pool IDs and every reference to them in next steps, executed argv and query pools; pool created and expiry clocks with the same lifetime; fetched and snapshot acquired clocks bound to their owner; and saved file paths and record path keys that match the declared names. Any other difference still fails.

- Why: 0456 (m16217 ask 4) and 0495 (m16384) each passed every public execution but stayed red on these runtime values alone, and each needed a hand classification of hundreds of leaves. Every later structure ticket would repeat that.
- Starts from: the 0454 reply diff, its normalizer and tests, and the raw classifications archived for 0456 and 0495 on Yellow.
- Rules: pair values by a one-to-one binding checked across the whole reply set, never by masking a string pattern; a pool lifetime must match exactly; raw replies stay retained; no new framework.
- Proof: the 0456 and 0495 raw comparisons come out empty; a planted change to a pool ID that breaks the one-to-one pairing, a changed lifetime, and a changed non-runtime value each fail; the tests fail on the current normalizer.
- Changes: the reply diff normalizer and its tests.
- Defers: nothing.
