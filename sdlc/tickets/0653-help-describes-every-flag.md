# 0653 — Help describes every flag and resolves entity names

Status: COMPLETE.
Landed: b55a552a89128d887c57541dbb61d97f072901e5

Milestone: 5

## Outcome

`help search trial` and every other command's help give each flag a description with its value form, one example and the sources it applies to; `help drug` lists the drug commands instead of failing; `pgx` help marks `--source` required when it is; and `health` help holds no runtime messages.

- Why: the hosted evaluation of 2026-11-05 (hosting m26488; evidence under `data/biomcp-hosting-evidence/2026-11-05-claude-eval/`), items 2, 3, 8 and 16: nearly every flag description was empty, `help drug` returned unknown_command, and `health` help carried pasted timeout messages. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).
- Starts from: the help renderer and command registry in `crates/biomcp/src/grammar*`, the query schemas whose descriptions 0625 shortened for typed tools.
- Rules: Investigate first: reproduce on current main in single and typed mode with the evaluation's command; if main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. Descriptions come from the schemas and registry, one source of truth for CLI help, the single tool and typed tools; none hand-written per flag in the renderer.
- Proof: a structural test fails when any command flag renders an empty description; installed cases for `help drug` and `help health`; routine gate.
- Changes: `crates/biomcp/src/grammar*`, `crates/biomcp-types/src/`, `tests/`; the fix and its cases.
- Defers: nothing.
