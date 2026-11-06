# Hosting validates the fixes before 1.0, and an evaluation decides new sources

Ian, 2026-11-05: "they're definitely after getting this stuff live on hosting, tested, and fully validated. Then what I want to do is do an evaluation with you based on what the BioMCP agent testing comes up with and see if it requires new sources. Then we'll decide if 1.0 waits for it or not. I'm not in any big rush to push 1.0 out the door. I'm more in a rush to get AWS solid and to get the agents framework working than I am for stamping out 1.0."

## What it means

- The fixes now landing go live on the hosted server from main, and are tested and validated there, before the 1.0 cut.
- Then Ian and the PM run an evaluation on what the BioMCP agent testing finds, and decide whether it needs new sources (regulatory event feed, RxNorm, Orange Book, DepMap and others).
- Ian then decides whether 1.0 waits for those sources. Until then, no new source work starts.
- 1.0 has no deadline. Work that makes the hosted service (AWS) solid or helps the agents framework comes before release ceremony. Machine time and lanes go to them first when they compete.
- The main preflight (0640) and the cut (0293) wait for that decision. Release tooling work already in flight finishes; no new candidate is cut before it.

## Replaces

The "wait for Ian's word" line on new sources in `decisions/2026-11-05-everything-in-1-0.md`, and the plan to cut as soon as the fixes and preflight finish.

## Amendment, 2026-11-06

Ian, 2026-11-05 (relayed by hosting m27741 via Jarvis m27744): no deploys until he approves. The hosted server stays on revision 4 while Amazon tests, and that covers any BioMCP release meant for hosting. So the hosting validation above waits for Ian's approval to deploy; BioMCP fixes keep landing on main meanwhile. Claude's retest of revision 4 added 0679 to 0684 to 1.0.
