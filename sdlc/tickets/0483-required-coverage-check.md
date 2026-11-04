# 0483 — Check that every command, option and section has a case

Status: COMPLETE.
Landed: 0081bf71afc41f4fa067899e0f242ee0bb99ec95

Milestone: 5

Depends on: 0431

## Outcome

A check derives the required list from the grammar, maps every case step onto it, and fails when an item has no command-line call. The list covers every command, option, section, global option and MCP tool, one argument refusal per command, and the shared failure kinds. It also prints how many calls cover each item, so duplicates are visible.

- Why: Ian's ruling in `decisions/2026-10-29-one-small-outside-in-suite.md`. The 2026-10-29 inventory found 608 required items. 383 had a call and 225 did not, and some items had over a hundred duplicate calls.
- Starts from: the grammar in `crates/biomcp/src/grammar*`, the generated `docs/reference/commands.md`, the MCP tool list in `mcp/protocol.rs` and `research.rs`, and the case files under `tests/cases/`.
- Rules: the list comes from the grammar, never from a hand-kept file. The failure kinds are named in the decision. Per-source failure items exist for trials, variants, drug labels and articles. A case step counts toward an item only by its argv. Items that 0456 has not yet covered are recorded as pending, and the pending list can only shrink. The check runs with the cheap structure checks before coverage. MCP tools (m15478): the existing step gains a named MCP tool call with its actual arguments, run through the existing protocol client and response validation, so `search` and `fetch` can be covered by YAML; no new harness or CLI alias, and a CLI call never counts as MCP coverage. 0456 writes the cases that use it.
- Proof: a planted new option with no case fails the check; removing the only case for a section fails it; the printed per-item counts match a hand count for three items.
- Changes: one check in the existing ratchets, its pending list and its cases.
- Defers: the case cut (0456) and the runner trial cut (0484).
