# 0492 — Make help short enough to read in one call

Status: COMPLETE.
Landed: 72542b1770428a45b3066f3af7f94b96c917e9a5

Milestone: 5

## Outcome

`biomcp help` and `biomcp --help` list the commands with one line each and fit in 4 KB. `help <command>` lists that command's entities and shared options in 8 KB, and `help <command> <entity>` gives the full arguments for one entity. Today `help` prints 120,937 characters and `help get` 51,312 on one JSON line, and the Markdown view escapes ordinary punctuation such as `:` and `.` as HTML entities.

- Why: BioData's experiment 502 rerun on internal build e7913048b (m15537), finding B. Help reads added 43,000 to 51,000 tokens per BioMCP-only cell, half or more of its tokens, and 27 of 83 round 1 help reads hit the agent's output cap. 0.9.3's `--help` was 2,640 characters.
- Starts from: the help command and its `biomcp.help` schema, the Markdown view, the help caps and `skills/SKILL.md`.
- Rules: JSON stays the default output and the Markdown view holds nothing the JSON lacks; every argument stays reachable from some help call; the Markdown view escapes only what Markdown needs.
- Proof: size checks fail when top-level help exceeds 4 KB or any `help <command>` exceeds 8 KB, in JSON and Markdown; a case shows `help get drug` names `--label-section`; a Markdown case fails on `&#58;`.
- Changes: help, its schema version and baseline, the Markdown view, size checks, cases and the skill's pointer to help.
- Defers: nothing.
