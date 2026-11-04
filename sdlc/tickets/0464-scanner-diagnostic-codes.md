# 0464 — Give secret-scan failures safe diagnostic codes

Status: COMPLETE.
Landed: 95c9739e03ae324dded81cf6648bd97b15376a20

Milestone: 5

## Outcome

When the history scan fails on a known invalid binding, such as an anchor commit that is not reachable, it prints a fixed diagnostic code naming the cause and the ledger row number. Informational lines, such as the six opaque archive disclosures, carry an explicit informational label, so they are never read as the failure.

- Why: three jobs (j20224, j20281, j20331) were misdiagnosed because passing and failing scans print the same informational lines while the real binding error is suppressed.
- Starts from: `release/secrets.py` and the ledger checks.
- Rules: keep exit code 2 and the suppression of unexpected details. Never print exception strings, matched values, cursor values, scanner stderr or source contents. No change to scanner rules or the ledger policy.
- Proof: planted ledger rows with an unreachable commit, a wrong blob and a wrong line each fail with their own code and row number; a passing scan prints its informational lines labeled as informational; a planted secret's value never appears in output.
- Changes: the scan script and its tests.
- Defers: nothing.
