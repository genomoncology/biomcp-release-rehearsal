# 0511 — Sort the code the cut suite no longer reaches

Status: COMPLETE.
Landed: 25e49bc07229449a28f2c47f50ebedf046514ea4

Milestone: 5

Depends on: 0484, 0485

## Outcome

Every uncovered line region in the cut suite's coverage report is sorted into one of three groups, and each group is handled: unreachable code is deleted; a distinct user-visible behavior with no case gets one minimal command-line case when it belongs to a required grammar item, a shared failure kind, or trials, variants, drug labels or articles; anything else stays uncovered and is listed by family with a reason. The coverage floor then resets to what the suite reaches.

- Why: the 0484 and 0485 cut suite passes all 479 cases and 0483's check, but line coverage fell from 90.00 to 78.63 percent (m16827 ask 4). The suite decision says coverage does not drive test growth, so the drop is a list to sort, not a target to restore.
- Starts from: the retained j23454 profiles and report and the lander's read-only uncovered-owner inventory.
- Rules: no retired trial returns; no case exists only to raise coverage; a new case uses real recorded replies when it can, as the suite decision says. The PM sets the new floor.
- Proof: the sorted list with counts per group; each new case fails on a build that breaks its behavior; each deletion leaves the reply diff empty.
- Changes: product code for deletions, `tests/cases/`, `coverage.floor`.
- Defers: nothing.
