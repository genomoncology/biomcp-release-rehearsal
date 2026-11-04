# 0465 — Create missing parents for source capture output

Status: COMPLETE.
Landed: 14da3e53de59f9e195b145eb426b010f1a394a5b

Milestone: 5

## Outcome

The source capture helper creates missing parent directories of its output path, and still refuses to overwrite an existing output directory.

- Why: jobs j20028 and j20342 failed because the helper calls `mkdir()` without parents.
- Starts from: the capture helper and its Make target.
- Rules: no change to what is captured.
- Proof: a helper test with a missing parent succeeds; a test with an existing output leaf is refused.
- Changes: the helper and its test.
- Defers: nothing.
