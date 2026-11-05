# Release qualification finds problems one at a time

Tickets: `0640-release-sections-pass-on-main-before-the-cut.md`, `0641-concurrency-tests-under-repeated-load.md`. Ruling: `decisions/2026-11-05-release-finds-no-bugs.md`.

## Cases

Nine problems surfaced between the cut of candidate ed9c36a96 (2026-11-04 16:26) and the cut of 1bb97e64f (2026-11-05 07:46). Over the same period, typed MCP mode joined 1.0 by Ian's decision and forced one new cut (015aea0a7). That was scope, not a defect.

| Group | Count | Cases | Found by |
|---|---|---|---|
| Release-only tooling | 5 | 0621 (wrappers checked old tickets; plugin never installed; installer fetched today's public files), 0634 (Windows wrapper PowerShell quoting, j29782 and j29789), 0635 (Windows wrapper had no build revision without `.git`, j29825), j30298 (mutation shard selection differed from the plan), j30306 (cold history clone assertion) | the first real run of each section on a candidate |
| History-scan allowlist | 2 | 0633 (50 findings in the appended history, j29778), 0638 (3 findings on public sibling branches, j30304) | the public-append scan, run only at release |
| Test timing on slow hosts | 1 | 0622 (bare `read` got `Interrupted` under M5 x86 emulation, j29497, after 99 minutes) | the cold matrix |
| Product bug | 1 | 0639 (concurrent first opens of a fresh cache returned Busy at the WAL switch; 1 of 24 starts in installed witness j30327) | native merge gate j30303 |

The Windows VM wrapper has never passed end to end. Each of its three runs stopped on a new defect before any installed case.

## Why

1. The release has taken a day instead of about two hours because qualification restarted repeatedly. Each find needed a ticket, a review, a landing and a rerun.
2. Routine checks pass, yet qualification found nine problems, because eight of the nine were in parts that routine checks never run. Only one was in the product.
3. Routine checks never run those parts because they need special hosts or long runs: the shared Windows VM, cold builds under emulation, a full public-history clone, hosted runners. So they live only inside the release tickets and only ever run against a cut candidate. 0556 moved the release workflow's code paths into the routine gate after Jarvis's warning (m19189), but not the sections that need special hosts.
4. They surface one at a time because each section stops at its first failure, and it cannot rerun until its fix lands. A second defect stays hidden behind the first. The Windows wrapper took three runs and found three defects, and never reached its tests.
5. Release tooling counted as done before it ran on its real host. 0635's proof called for a real VM run, but the ticket landed on the local walkthrough alone, so the next Windows defect is still waiting at release.
6. The product bug passed routine checks because the test that catches it fails about one run in twenty-four, and routine checks run it once. The same test had failed under load before (0569, with startup contention left unproven in 0287). Each failure was read as a timeout because the logs kept only the outer error, not the SQLite step that failed. Idea 0538 to harden startup was declined on 2026-11-01 until the failure was seen.
7. Each find costs hours because the long sections take one and a half to two hours: the hosted rehearsal runs about two hours, with the Intel Mac job alone at 100 to 118 minutes, and the cold matrix about 95 minutes. A product fix correctly means a new cut and a rerun of everything.
8. Nobody stopped the pattern because each find was fixed as its own ticket, correctly, without asking what they shared. 0635 recorded that it was the third Windows wrapper defect found one run at a time.

## Cause

Release qualification is the first real run of its own machinery. Sections that need a special host run only against a cut candidate, so their defects arrive one per run, on the critical path. The one product bug got through because a concurrency test that fails rarely ran once, on a quiet machine.

## Fixes

1. Now, no code: start each section of 1bb97e64f as its own job in parallel, and rerun only the section that fails. Run the Windows VM wrapper alone at once against the retained ed9c36a96 Windows artifacts until it passes (m26087). This keeps each rerun to the section that failed.
2. 0640: each release section passes alone against main on its real host before the next cut, and a ticket that changes release tooling is complete only after its section has run there. This prevents the tooling, allowlist and timing groups.
3. 0641: the multi-process cache checks run many times under load on a schedule, and one failure opens a ticket. This would have caught 0639 when 0569 first saw it.

The hosted rehearsal skips the build cache on tag dispatches, so the real release builds clean. That keeps it at about two hours, run in parallel with the local sections. It stays as it is.

## Reach

Jarvis saw the same cause in ThinkThen's hosted failures (m19189). The part that is new here, sections on special hosts that run only at release, goes to Jarvis.
