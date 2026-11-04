# Targeted Make checks run through the canonical tool setup

Status: OPEN.
Milestone: 5

## Outcome

One small wrapper runs any single Make goal with the same pinned Python, Rust and inherited PATH as the ticket gate, by reusing the substitution `scripts/check_usage_tests.sh` already does, and the docs branch delegates to it.

- Why: the coordinator's RCA (m24903) found three setup failures across two tickets: 0599 j29267 and 0336 j29449 ran M5's Python 3.9, so `tomllib` failed to import; 0336 j29463 replaced PATH and lost sccache. `scripts/check_usage_tests.sh` already reads `ticketsGate` from `sdlc/pm.json` and swaps only the Make goal, but other targeted checks bypass it.
- Starts from: `scripts/check_usage_tests.sh` and `sdlc/pm.json`.
- Rules: test infrastructure only; no product source change, so it does not reopen the frozen 1.0 product source and does not block 0336. No second pin list, checker or framework. The goal name is the only substitution.
- Proof: a test runs a targeted goal from a shell whose PATH puts an old Python first and gets the pinned Python and sccache; the docs branch gives the same result through the wrapper; an unknown goal fails with Make's own error.
- Changes: `scripts/check_usage_tests.sh`, `scripts/make_goal.sh`, `scripts/test_*.py`; the wrapper, the docs branch and a test.
- Defers: nothing.
