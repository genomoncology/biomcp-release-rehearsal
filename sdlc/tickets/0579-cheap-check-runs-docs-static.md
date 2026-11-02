# Cheap check runs the static docs check

Status: COMPLETE.
Landed: 6c774a8d22250b0ed8a9dbc6750217f62ce44fc5
Milestone: 5

## Outcome

`make check-cheap` runs `python3 scripts/check_docs.py` first, so a static docs or size-limit failure stops a focused check before the landing gate's later stages.

- Why: 0577 passed its focused cheap check, then landing job j26126 failed only in docs-check because `scripts/test_nightly_live_check.py` reached 640 lines against the existing 600-line limit. `scripts/check_docs.py` is local, read-only static validation with no build, subprocess or network.
- Starts from: `Makefile` (`check-cheap`, `docs-check`), `scripts/check_docs.py`.
- Rules: add one recipe line to `check-cheap`; keep the existing `docs-check` call; add no new checker, helper or limit.
- Proof: a file over the limit fails `make check-cheap`; the cheap gate passes on main.
- Changes: `Makefile`, plus the existing `scripts/check_shell.py` (which reads check-cheap's goals from its first dry-run line, lines 11-14) so it still finds the recursive Make goals, and the `scripts/test_gate_stages.py` fixture so it carries the newly invoked static script. The focused-check regression from 0572 keeps failing a partial selection.
- Defers: nothing.
