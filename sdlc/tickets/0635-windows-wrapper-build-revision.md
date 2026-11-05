# Windows release wrapper passes the tooling build revision explicitly

Status: COMPLETE.
Landed: 612e37d75a472d8394ea539b77e3f288efb6139a
Milestone: 5

## Outcome

The Windows wrapper derives the verified full tooling commit before it archives the tooling and passes it as `BIOMCP_BUILD_REVISION` when `gate.py` compiles the consumer runner on the VM, so a checkout with no `.git` builds, while the expected product revision stays the candidate under test (ed9c36a96 now, the post-0636 head next).

- Why: m25392. Corrected wrapper job j29825 passed setup, transfer and candidate identity, then failed before installed cases because `build.rs` cannot resolve a build commit: the controller transfers `git archive HEAD` with no `.git`, and `check_windows_release.cmd` at d81b7fb does not supply `BIOMCP_BUILD_REVISION`. This is the third Windows wrapper defect found one run at a time (0621, 0634).
- Starts from: `scripts/check_windows_release.sh`, `scripts/check_windows_release.cmd`, `release/gate.py`, and the existing explicit build-revision path in `crates/biomcp/build.rs`.
- Rules: proof tooling only; no product source change. Reuse the existing build-revision path; no fake Git, guessed revision or validator bypass. The tooling revision and the product revision stay distinct in the evidence. Before the next real VM run, walk the whole wrapper end to end in a no-`.git` archive locally (every step the VM runs, up to the installed cases) so any further defect of this kind is found in one pass, not per VM run.
- Proof: a no-`.git` bootstrap regression compiles the consumer runner with the explicit revision; the local end-to-end walk is recorded; then the real VM wrapper compiles the runner and runs the installed gate.
- Changes: `scripts/check_windows_release.sh`, `scripts/check_windows_release.cmd`, `release/gate.py`, `release/test_gate.py`; revision propagation, the walk and the regression.
- Defers: nothing.
