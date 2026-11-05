# 0652 — Usage errors name the argument that is wrong

Status: COMPLETE.
Landed: 266387d2cc90684c6372b6962a976be2fe64fbd6

Milestone: 5

## Outcome

Every usage error names the argument that actually broke the request, lists its valid values as a list, and gives the corrected command: a second section where one is allowed names the section, a filter the source does not support names the filter and the sources that support it, and a required flag missing names that flag.

- Why: the hosted evaluation of 2026-11-05 (hosting m26488; evidence under `data/biomcp-hosting-evidence/2026-11-05-claude-eval/`), items 1, 4, 8 and 9: `get trial NCT07491445 summary eligibility --source ctgov` and `search diagnostic --source fda-cdx --gene KRAS` blamed `--source`; `search pgx --gene CYP2D6` answered `usage.retired`; `article text 39214094` gave a sentence as `valid_values`. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).
- Starts from: the grammar's usage envelope builder and parser errors in `crates/biomcp/src/grammar*`, `crates/biomcp-types/src/response/error.rs`, and 0604's filter errors.
- Rules: Investigate first: reproduce on current main in single and typed mode with the evaluation's command; if main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. One rule for every command, from the command registry; no per-command message tables. The typed tools reuse the same envelope (0628).
- Proof: replayed usage cases for each of the four examples assert the named argument, a list of valid values and a corrected command that parses; routine gate.
- Changes: `crates/biomcp/src/grammar*`, `crates/biomcp-types/src/response/error.rs`, `tests/`; the fix and its cases.
- Defers: nothing.
