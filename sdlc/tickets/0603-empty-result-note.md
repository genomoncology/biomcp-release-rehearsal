# Empty and failed sections read differently

Status: OPEN.
Milestone: 5

## Outcome

In the markdown reply, a section whose source answered with no data says so in one plain line naming the source, and a failed section says it failed, with its reason and next step, so an agent can tell "the source says absent" from "not found because the call failed".

- Why: `planning/eval-recommendations-2026-11-04.md` item 2 lists an empty-result note in the bundle that beat the cold agent. Main's `Section` already tells `Empty` apart from `Unavailable`, but the markdown view in `crates/biomcp/src/view/mod.rs` prints only `**state:** empty` or "None".
- Starts from: the generic walk in `view/mod.rs` and the `Section` and `Status::Absent` types in `crates/biomcp-types/src/response/`.
- Rules: markdown only; JSON replies stay byte-identical. One fixed sentence per state, generated from the state and source, never per command.
- Proof: installed-binary cases show an empty section, a failed section and an all-absent reply, each with its sentence, and the JSON form unchanged.
- Changes: `crates/biomcp/src/view/mod.rs`, `tests/cases/view-empty/**`; the markdown view and its cases.
- Defers: nothing.
