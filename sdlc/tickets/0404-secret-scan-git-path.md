# 0404 — Find git in the release image during the secret scan

Status: COMPLETE.
Landed: dd125a40d4bf4fa485924b84ecf5a316993b47c4

Milestone: 5

## Outcome

`release/secrets.py` finds git where the pinned release image installs it, `/usr/local/bin/git`, and scans the full history in the cold rehearsal and the hosted workflow. It keeps its isolated environment: it builds a fixed search path that includes the image's git location instead of inheriting the caller's PATH.

- Why: the cold rehearsal of final candidate 179f3acb (j19965, coordinator m12743, 2026-10-28) showed `scan()` resetting PATH to `os.defpath`, which is `/bin:/usr/bin`, so git was not found in the arm64 manylinux image and the history scan never ran. This is not a secret finding.
- Starts from: `release/secrets.py`, its tests, the pinned image from 0394 and the retained diagnosis JSON.
- Rules: no suppression, skipped stage or new framework; the scan still fails closed when git is truly missing.
- Proof: an outside-in case runs the scan with git only in `/usr/local/bin` and passes; a case with no git still fails; the next cold rehearsal reaches the history scan.
- Changes: `release/secrets.py` and its cases.
- Defers: nothing.
