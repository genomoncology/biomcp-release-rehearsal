# Help, skill and MCP instructions open with a short generated map

Status: COMPLETE.
Landed: 4545ef31274b2c873268d207f1c0e7265009443a
Milestone: 5

## Outcome

The front page of `biomcp help`, the skill and the MCP server instructions all open with the same short map: one line for each entity naming its commands, sections and sources, followed by one line for each known trap. It is generated from the command and source declarations, so it cannot drift. Per-command help is shorter, and every command description names its sources.

- Why: the agent benchmark of 2026-11-02 (`planning/eval-baseline-2026-11-02.md`) found that agents don't know which sources BioMCP covers and that the skill and help are a large fixed cost. Experiment 513 (relayed by Ian, 2026-11-03) found that telling the agent what BioMCP covers and how to call it roughly halved the tokens it read. A generated map of about 1,800 characters, with one line per trap, matched or beat a 4,000-character hand-written map and gave the steadiest runs. Agents rarely read help and never read schemas, so the map must sit where they look first. Ian approved fixing the general causes before 1.0 (2026-11-03).
- Starts from: `skills/SKILL.md`, `protocol::INSTRUCTIONS` in `crates/biomcp/src/mcp/protocol.rs`, the help front page, and the command and source declarations that 0421 and 0422 made the single source of truth.
- Rules:
  - The map is generated, never hand-copied, and has no flags and no prose. A test fails when an entity, command, section or source is missing from it.
  - Trap lines are general rules about BioMCP itself (for example, which ID forms a command takes), never hints about particular questions. Never tune to the 20 benchmark questions.
  - The map stays under 2,500 characters including trap lines; a test enforces the limit as a setting with that default.
  - The map names service sources; carried components stay in source help, and one map line names the command that lists them all (m22817).
  - Per-command help keeps syntax, sections and source names, and drops repeated prose.
  - This ticket lifts the command freeze for help, skill and instruction text only.
- Proof: installed-binary cases show the map in `biomcp help`, in the MCP `initialize` instructions and in the shipped skill, byte-identical across the three. A synthetic new entity in a test declaration appears in the map without other edits. The longest per-command help is at least 40% shorter than at 27eadce.
- Changes: help rendering, `skills/SKILL.md`, the MCP instructions, command descriptions, and their cases.
- Defers: measured trimming beyond this, which waits for experiment 513's ranked list.
