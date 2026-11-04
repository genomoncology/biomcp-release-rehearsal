# 0513 — Bind fetched clocks to each cache acquisition in the reply diff

Status: COMPLETE.
Landed: f0d87d8c8fca634eb5e464a5f6843072f073989e

Milestone: 5

## Outcome

The reply diff binds each schema-owned fetched timestamp to its own isolated cache acquisition, read from the recorded `execution.root` and the existing owner, step and occurrence provenance, instead of requiring one timestamp bijection across every root. Native request metadata stays a separate, schema-known field and is compared exactly. Any other difference still fails.

- Why: the 0420 comparison j23854 (m17345). In disease-monarch/monarch-disease-cache-admission, three roots each run two steps; the baseline fetched seconds differ across roots while the candidate's match, so the current owner scope collapses the roots and fails, and native request metadata makes the raw comparison exit 2. 0509 fixed the same class for acquired clocks per catalog root.
- Starts from: the 0501 and 0509 normalizer and the retained j23854 captures.
- Rules: bind inside the recorded root, never by a mask; every other timestamp the normalizer binds is checked for the same cross-root assumption and fixed in this change, including repeated sync events inside one root, bound by the recorded sync sequence so later reads pair with the latest active acquisition (m17345 ask 2). The capture helper records a cross-command sequence, old sequence-less collisions stay explicit failures, and only the study-catalog owner reruns offline on its existing input with both retained executables (m17345 ask 3); per-read scoping and grouping by seconds stay forbidden; no recapture.
- Proof: the retained j23854 case pairs after the change and fails before it; a changed public field, a fetched clock changed inside one root, and a missing or unpaired execution each still fail.
- Changes: the reply diff normalizer and its tests.
- Defers: nothing.
