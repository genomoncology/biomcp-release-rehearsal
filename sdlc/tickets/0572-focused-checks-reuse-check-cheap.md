# Focused check scripts reuse the cheap check

Status: COMPLETE.
Landed: 08ec38e98040144d755bb43ea237baaa55dad445
Milestone: 5

## Outcome

Tracked focused check scripts run `make check-cheap` for their ordinary initial checks instead of a hand-picked subset, so a focused routine cannot pass while the landing gate's cheap stage fails.

- Why: 0567's j25680 and 0566's j25756 both passed their focused routines, then stopped in the landing gate's cheap stage, because each script selected a different part of `make check-cheap` (m19781). No full proof stage was wasted, but each cost a landing round trip.
- Starts from: `scripts/check_*.sh` and `Makefile` target `check-cheap`.
- Rules: replace partial cheap-check selections with `make check-cheap`; keep each script's focused tests and deliberate proofs-only reruns. No new framework or shared rule.
- Proof: a test or lint in the cheap gate fails when a tracked focused script calls a cheap-check sub-target without `check-cheap`; the cheap gate passes.
- Changes: tracked focused scripts and that one check.
- Defers: nothing.
