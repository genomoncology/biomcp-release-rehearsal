# 0560 — Make the installed smoke test ask for a current schema

Status: OPEN.

Milestone: 5

## Outcome

The installed-package smoke test asks the installed CLI for a schema that 1.0 ships, and a routine test fails when the smoke test names a schema that is not in the checked-in schema set.

- Why: the private no-op release rehearsal (run 41353995773, m19475). Apple ARM and Linux ARM builds installed and passed version, verbose and offline health, then `release/smoke.py:45` asked for the removed schema `biomcp.problem/1`, and the installed CLI answered with `usage.schema`. The smoke test runs only from the release matrix, so the routine gate never saw it.
- Starts from: `release/smoke.py:38-50`, `crates/biomcp-types/schemas` and the release suite.
- Rules: the smoke test and the container check (`release/container_check.py`, with its `release/test_linux.py` fixtures) share one schema ID that the checked-in schema set contains, read from that set or named once as a constant that a routine test checks. Keep the identity check. Fix every release script that names a removed schema ID. Change nothing else in either check.
- Proof: a routine release test fails when the smoke test's schema ID is missing from the checked-in set and passes on main; the next private rehearsal's installed smoke passes on every platform.
- Changes: the smoke test, the container check, their fixtures and the release suite.
- Defers: none.
