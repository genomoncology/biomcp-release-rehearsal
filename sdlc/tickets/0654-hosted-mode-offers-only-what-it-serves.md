# 0654 — Hosted mode offers only what it serves and says why a source is off

Status: OPEN.

Milestone: 5

## Outcome

In hosted mode, help, `next` and the tool list never offer a command, flag or section the server refuses (`health`, `sync`, `store clean`, `--fresh`); no `next` command carries `--config` or any server path; `version` works or is not offered; `help sources` shows which sources this server serves; and a source turned off says whether the hosted use ceiling or the operator turned it off, and that a self-hosted install can enable it.

- Why: the hosted evaluation of 2026-11-05 (hosting m26488; evidence under `data/biomcp-hosting-evidence/2026-11-05-claude-eval/`), items 5, 6, 7, 15 and issue 5: `version` failed, `health` was listed then refused, `next` suggested `--fresh` and `sync clingen`, every `next` carried `--config /etc/biomcp/config.json`, and OncoKB showed only `skipped: disabled`. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).
- Starts from: hosted mode in `crates/biomcp/src/mcp/`, the `next` builder, `decisions/2026-11-04-hosted-source-modes.md` and 0619's `hosted_max_use`.
- Rules: Investigate first: reproduce on current main in single and typed mode with the evaluation's command; if main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. Hosted refusals stay as they are; only what is offered changes. The wording names the ceiling, never a commercial term BioMCP cannot know.
- Proof: hosted installed cases at `restricted` assert each refused item is absent from help, `next` and tools/list, no `--config` in any `next`, `version` answers, and a disabled source names its reason; local mode unchanged; routine gate.
- Changes: `crates/biomcp/src/mcp/`, `crates/biomcp/src/grammar*`, `tests/`; the fix and its cases.
- Defers: nothing.

## Amendment, 2026-11-05

PM ruling (m27096): 0654 covers the operational surface in its Outcome: refused commands and flags, `--config` and server paths in `next`, `version`, `help sources`, source-off reasons, and the typed continuations, error and relation guidance already in progress. Field-level metadata for fields only a disabled source answers moves to 0676, after the cut.
