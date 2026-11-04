# 0406 — Accept the legal files in the desktop bundle member check

Status: COMPLETE.
Landed: 653907b8576d2b28e5a1ed80c946c3ec500e75c9

Milestone: 5

## Outcome

The install gate's desktop bundle check accepts exactly the members a bundle must contain: `manifest.json`, the server entry, `LICENSE` and `THIRD-PARTY-NOTICES.txt`. It still rejects any other member.

- Why: the cold rehearsal of final candidate 179f3acb (j19965, coordinator m12761, 2026-10-28) failed because `release/gate.py` still expects only the manifest and entry, although 0395 correctly added the legal files to every artifact.
- Starts from: `release/gate.py`, its tests and the retained bundle and traceback.
- Rules: keep 0395's requirement that every artifact carries LICENSE and notices; derive the expected member list from one place shared with packaging if one exists; no new framework.
- Proof: outside-in cases accept a bundle with the four members, reject one missing LICENSE or notices, and reject one with an unexpected member; the next cold rehearsal passes the bundle check.
- Changes: `release/gate.py` and its cases.
- Defers: nothing.
