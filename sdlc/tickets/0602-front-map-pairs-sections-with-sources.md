# The front-door map pairs each section with its source

Status: COMPLETE.
Landed: d73a0226692647bb5b8a60410adc8c0e91ce0b64
Milestone: 5

## Outcome

The generated map at the top of help, the skill and the MCP instructions names each section's source beside it, so an agent can pick a section and source without a help call.

- Why: `planning/eval-recommendations-2026-11-04.md` item 3: the same main binary used 0.89× the cold agent with its own help and 0.70× with a one-page map that pairs each section with its source; agents made about as many help calls as data calls without it. Main's map (0586) lists an entity's sections and sources as two separate sets, so the reader cannot tell which source serves which section.
- Starts from: `crates/biomcp/build/front_map.rs` (line format and `DEFAULT_CHARACTER_LIMIT`), and the appendix map in the recommendations file as a model of the shape, not text to paste.
- Rules: the map stays generated from the command registry, never hand-written. Each section shows its source in parentheses, and a source shared by every section of an entity is named once. The character limit rises to 4,000. The trap lines stay. Help, the skill and the MCP instructions show the same map.
- Proof: a test proves every section in the registry appears with its source and the map fits the limit; an installed-binary check shows the same map in all three places.
- Changes: `crates/biomcp/build/front_map.rs`, `crates/biomcp/build.rs`, `skills/**`; `front_map.rs`, its tests, and the generated help, skill and instructions.
- Defers: nothing.
