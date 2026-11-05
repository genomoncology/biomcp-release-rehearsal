# MCP server can offer one typed tool per command, on a branch for the bench

Status: OPEN.
Milestone: 5

## Outcome

On the branch `mcp-typed-tools`, `biomcp serve` takes a tool-mode setting: `single`, today's one CLI-argument tool and the default, or `typed`, one tool per command such as `search_trial`, `get_trial`, `get_variant` and `search_article`. Each typed tool is generated from the command registry and is usable on its own. The branch is built for the Biomed Agent Bench and never merges to main until the bench results and Ian's decision.

- Why: Ian's decision relayed by evals and Jarvis (m25063, m25064): today's harnesses (Claude, Codex, pi) and Amazon Quick search for tools as they need them instead of preloading them, and Quick recommends function-specific tools with relevant summaries. Ian: "if the multi-tool is worse than the single tool, we might not want to keep it." Experiment 513's 14-tool Python prototype cost 1.09–1.2× the single tool, within noise, so the typed tools must do better than the prototype.
- Starts from: `crates/biomcp/src/mcp/protocol.rs` (`tool`, `tools`, `DESCRIPTION`), `crates/biomcp/src/mcp/output_schema.rs`, `crates/biomcp/src/mcp/research.rs`, the command tree in `crates/biomcp/src/grammar/`, the front map generator `crates/biomcp/build/front_map.rs` (section and source pairs from 0602), and the brief's prototype lessons.
- Rules:
  - Branch only, from current main; the frozen 1.0 candidate and main are untouched. Commit and push work in progress to the branch, never land to main.
  - Tool mode is one setting of `biomcp serve` (config key and flag); `single` stays the default. Both modes run from one build.
  - Every typed tool is generated from the command registry, never hand-written, and runs the same parsed command as the CLI, so replies match the CLI byte for byte.
  - Each description, at most about 300 characters, stands alone: what the tool returns, the ID forms and names it accepts, and the source of each section.
  - Arguments: enums for sources, sections and other short choice lists; each argument description at most 80 characters.
  - Tools that take an ID accept a plain name where the command does (0599 on main), and otherwise their error gives the exact next call as a typed-tool call.
  - Replies carry structured content plus a one-line text summary (superseded by 0624: the summary leads the JSON text).
  - Server instructions: the short command map with the most useful line first, within 4,096 characters.
  - Hosted modes and `hosted_max_use` apply unchanged in both modes.
- Proof: installed-binary MCP cases in `typed` mode list every generated tool within the size budgets, call one search and one get per entity, and match the CLI's replies; a wrong enum value is refused (superseded by 0626: as an `isError` usage result); `single` mode is byte-identical to main. Report the total tool-list characters. The bench build moves to 0632, after the review fixes 0624–0631 (`records/2026-11-05-typed-mcp-tools-review.md`).
- Changes: `crates/biomcp/src/mcp/**`, `crates/biomcp/src/grammar/**`, `crates/biomcp/src/help.rs`, `docs/**`, `crates/biomcp/src/config.rs`, `crates/biomcp/src/config/file.rs`, `crates/biomcp/build/**`, `tests/tool-cases/**`, `tests/cases/mcp/**`; the tool mode setting, the typed tool generator, descriptions, cases and the branch build.
- Defers: merging to main, until the bench results and Ian's decision.
