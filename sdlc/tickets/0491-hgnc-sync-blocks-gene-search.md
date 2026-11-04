# 0491 — Keep gene searches working while HGNC syncs

Status: COMPLETE.
Landed: 64ba85762f0977e2bd717184b25a9fb1e39f7fc9

Milestone: 5

## Outcome

A command that resolves a gene keeps working while another BioMCP process syncs HGNC. In experiment 502's fresh homes, `search trial --gene … --phase 2` succeeded and the next `--phase 3` call failed with `hgnc` in `state syncing` and exit 4.

- Why: BioData's experiment 502 rerun on internal build e7913048b (m15537), finding A, seen in two round 1 cells.
- Starts from: the HGNC bundle and store owners in `crates/biomcp-fetch` and `crates/biomcp-query`, the sync state record and the exit code mapping.
- Rules: diagnose first and record the cause in progress. A reader uses the last complete snapshot while a sync runs, or waits a bounded time for the first one when none exists; a sync left behind by a finished or killed process never blocks a later command. No new lock framework.
- Proof: a case in a fresh home runs two gene trial searches back to back, and another runs one while a sync is in progress; both fail on the base build and pass after the fix.
- Changes: the HGNC sync and store code, cases and baseline.
- Defers: nothing.
