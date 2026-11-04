# 0561 — Hand generated license notices to the Windows artifact build

Status: COMPLETE.
Landed: 7608fa5c74d70614a78d8b9c87d23872b5cc5783

Milestone: 5

## Outcome

The Windows release job generates the third-party license notices and its artifact build finds them, as the other platforms' builds do.

- Why: the private no-op release rehearsal (run 41353995773, m19475). The Windows artifact build failed with "artifact requires generated third-party notices" after cargo-about installed and ran, with no license rejection, so the notices are generated somewhere the artifact step does not look, or not written at all.
- Starts from: the Windows job and the notice step in `.github/workflows/release.yml`, `release/notices.py`, `release/artifacts.py` and the raw Windows log from the rehearsal.
- Rules: run a bounded diagnosis from the raw log first and record the cause on this ticket: path separators, the output location, the shell or an exit status ignored. Fix that cause only. All platforms keep one notice path.
- Proof: a release test reproduces the cause on Linux where it can, such as a Windows-style path, and passes after the fix; the next private rehearsal's Windows artifact build passes.
- Changes: the notice or artifact step and its tests.
- Defers: none.
