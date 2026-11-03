# Fix the bugs and misleading errors the agent benchmark found

Status: COMPLETE.
Landed: f837afab4c65628a0c940987df443d956e99a621
Milestone: 5

## Outcome

Each bug and misleading error from the agent benchmark of 2026-11-02 is fixed, and each error names the right next step.

- Why: `planning/eval-baseline-2026-11-02.md` item 4. BioMCP made 25 error calls against 13 for scripts. Ian approved fixing the general causes before 1.0 (2026-11-03).
- Starts from: the list below, each reproduced on the installed binary first.
- Rules: fix each by its general cause, never by the benchmark question. The cases:
  - free-text drug label search reports a match count but returns no rows;
  - an unsupported label section does not list the valid ones;
  - `label` without a section names the wrong option;
  - `help get clingen` misleads, `--coding` dumps a regex, and `regulatory` is listed but unsupported;
  - retired `search article -q` and `--query` get an alias with a one-line note;
  - a bundle that is syncing on first read waits for the sync, within a deadline setting, instead of answering "syncing";
  - drug approvals default to more than one submission, as a setting.
- Proof: one installed-binary case per item, each failing at 27eadce and passing after.
- Changes: the commands named above and their cases.
- Defers: nothing.
