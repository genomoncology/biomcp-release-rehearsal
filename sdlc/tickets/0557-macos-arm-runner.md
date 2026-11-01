# 0557 — Move Apple ARM release jobs to macos-15

Status: OPEN.

Milestone: 5

## Outcome

The release workflow builds, qualifies and install-checks the Apple ARM release on the `macos-15` hosted image instead of the retiring `macos-14` image, with the same architecture, platform tag and toolchain.

- Why: the private no-op release rehearsal (run 41348267985, m19292) passed the candidate stage, but GitHub cancelled the `macos-14` ARM qualification at once during an image brownout; `macos-14` retires on 2026-11-02 (actions/runner-images issue 13518).
- Starts from: `.github/workflows/release.yml:205-207,451`, `release/workflow.py:31-32` and its tests.
- Rules: change only the runner label from `macos-14` to `macos-15`. The platform tag stays `macosx_11_0_arm64`, the deployment target and the pinned toolchain stay as they are, and the Intel job stays on `macos-15-intel`. A workflow test fails if any release job names a `macos-14` runner.
- Proof: the workflow tests pass, and the next private rehearsal qualifies the Apple ARM build on `macos-15`.
- Changes: the release workflow, the platform table and their tests.
- Defers: none.
