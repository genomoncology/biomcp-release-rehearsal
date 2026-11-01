# 0559 — Install cargo-deny in the release docs check

Status: COMPLETE.
Landed: c6d2835aa707277f69884409d26334d0b1173634

Milestone: 5

## Outcome

The release workflow's docs-check job installs the pinned cargo-deny before it runs the security tests, so `make audit` runs in qualification as it does in the routine gate.

- Why: the private no-op release rehearsal (run 41353995773, m19475). The Linux x64 docs-check ran `release/test_security.py:345`, which expects `make audit` to deny an unknown Git source, and cargo reported that it has no `deny` command.
- Starts from: the docs-check job in `.github/workflows/release.yml`, `release/toolchain.json` and `release/test_security.py:335-347`.
- Rules: install cargo-deny at the version pinned in `release/toolchain.json`, the same way the workflow installs its other pinned cargo tools, before any step that runs `make audit` or the security tests. Pin a version there if none is pinned. Skip no test.
- Proof: a workflow test fails if a job runs the security tests without first installing cargo-deny; the next private rehearsal's docs-check passes `test_security.py`.
- Changes: the release workflow, the toolchain pin if missing, and the workflow tests.
- Defers: none.
