# Typed MCP tools

2026-11-05. Design for offering one MCP tool per BioMCP command beside today's single CLI-argument tool. Ticket 0623 built a first version on the branch `mcp-typed-tools`; this document sets the design the follow-up tickets build to.

## Decision and goal

Ian decided that BioMCP's MCP server should again offer function-specific tools such as `search_trial`, `get_trial`, `get_variant` and `search_article` (relayed by evals and Jarvis, m25063 and m25064). The reasons: Claude, Codex and pi now search for tools as they need them instead of preloading every description, and the Amazon Quick team said Quick does not preload MCP tools either and recommended function-specific tools with relevant summaries. Ian: "if the multi-tool is worse than the single tool, we might not want to keep it."

The goal is a typed mode that a tool-searching agent uses at lower cost and with fewer lost answers than the single tool, measured by the Biomed Agent Bench in pi on a strong model, pi on a weaker model, and Claude Code. Until that result and Ian's decision, typed mode lives on its branch and the single tool stays the default everywhere.

Experiment 513's 14-tool Python prototype set the bar: the single tool cost 0.83× and 0.92× of the typed tools in two pairs, within noise, and the 14 tools' descriptions cost about 546k tokens over one round. Its lessons: descriptions are the product, because tool search returns one tool alone; keep argument text short; use enums for short choice lists; accept names; reply with structured content and a one-line summary; put the most useful line first in the server instructions.

## What the branch does today (d5b0e4ba1)

- `mcp_tool_mode` config key and `serve` flag: `single` (default) or `typed`.
- In typed mode `tools/list` returns one tool per entry of `grammar::command_paths()`: 58 tools, named `{verb}_{entity}` with hyphens as underscores. The single `biomcp` tool and the `search` and `fetch` research tools are not offered.
- Description (`typed/description.rs`): `"{verb} {entity} source records. "`, the first sentence of the positional argument's schema description with some words shortened, then `Sections=source: ...` grouped from the same section-source declarations as the front map; capped near 300 characters by trimming the input sentence.
- Input schema (`typed/arguments.rs`): the command's query schema, `$ref`s inlined, keeping type, enum, const, bounds, required and `additionalProperties`; each description cut to its first line and 80 characters; `x-cli-flags` booleans exposed as separate boolean properties.
- Output schema (`output_schema::for_command`): the reply envelope with the command's reply schema IDs as an enum, or the error shape.
- A call is validated against the compact input schema, spelled back into CLI words (flags first, `--`, then positionals) and run through the same path as the CLI, so the structured reply is the CLI's JSON.
- An invalid call gets JSON-RPC `-32602 Invalid params` with no further detail.
- Text content is replaced by one line: the status and up to three `next` follow-ups translated into typed-tool calls, validated through the CLI parser; a usage error with no follow-up suggests a replacement call.
- Server instructions prepend one line, "Call verb_entity tools with named arguments; inspect problems and next commands.", to the front map.
- Size: the tool list is 130,784 characters (the whole `tools/list` response 130,935).
- Proof: installed typed cases, parity with CLI replies, single-mode bytes unchanged against main, scanners; routine gates green.

## Design

### Surface

- One build, one setting. `single` stays the default until the bench result and Ian's decision. Hosted modes and `hosted_max_use` apply identically in both.
- Typed mode exposes query commands only. Each tool runs exactly one parsed CLI command, so CLI, single tool and typed tool give the same reply for the same request.
- Help and schema stay reachable in typed mode through one small `biomcp_help` tool taking a command path, so an agent that lands on the wrong tool can find the right one without a second server.
- The `search` and `fetch` research tools stay in typed mode, because the 2026-10-25 amendment to `decisions/2026-10-23-mcp-is-one-tool-over-the-cli.md` requires them for ChatGPT.
- Merging typed mode to main needs that decision amended.

### Naming

- `{verb}_{entity}` from the command registry, never hand-named, so a new command gets a tool with no extra code. Names must stay unique and stable; a rename is a breaking change recorded in the release notes.

### Descriptions

- Generated, never hand-written, at most 300 characters, standing alone. Order: what the tool returns in plain words; the ID forms and names it accepts; each section's source. The leading words carry the search terms an agent would type (the entity and its common synonyms, such as "clinical trial" for trial), because tool search ranks on them.
- The description starts from the command's own one-line summary in the registry (the `///` doc line in `commands.rs`, exposed as `CommandSpec.summary` and rewritten as a plain sentence), never "source records", and the input sentence comes from the declared primary positional, not the alphabetically first one.

### Inputs

- Compact schemas, with each property keeping its own description (taken before a `$ref` is resolved) cut to 80 characters, no descriptions on `items` or `anyOf` branches, no nullable branches on optional fields, and enums only up to a configurable size (default 30), above which the description names the accepted form and the help call.
- Names accepted wherever the command accepts them (0599 drug names, gene symbols, trial and variant text).

### Outputs and errors

- `structuredContent` is the CLI JSON reply unchanged. Typed mode follows `mcp_reply_form` with default `both`, as single mode does: the first text block starts with the one-line summary (status and up to three executable typed calls) followed by the serialized JSON, because the MCP spec says structured results SHOULD also carry the JSON as text and some clients pass only `content[0].text` to the model, especially on errors. The size cap counts the JSON value alone in both modes, so the two modes refuse the same replies.
- The output schema is left out of typed `tools/list` by default (a setting keeps it), since it is about 35% of the list, carries only the envelope types, and strict clients throw on any mismatch.
- The output schema must not dominate the tool's size. Tool-search harnesses load a matched tool whole, so per-tool size is what an agent pays. The design target is under 2,000 characters per tool for description plus input schema plus output schema, and the total list reported on every build.
- Only unknown keys and non-object arguments are refused before parsing. Every other value is spelled into CLI words and parsed by `grammar::parse`, whose usage envelope (an `isError` tool result naming the argument, valid values and the exact next call) is the reply, so a string `"10"` for a number or a wrong enum value is recoverable in one step and the rules match the CLI exactly.
- "Search for this name" guidance comes from the CLI's own usage error for a get command, never from word-shape guesses in the typed layer.

### Instructions

- First line: the single most useful sentence for an agent that sees only one line (pi). Then a typed-mode front map that names tools rather than CLI syntax, within Claude Code's 4,096 characters.

### Hosted

- Hosted typed `tools/list` omits tools every hosted call would refuse (such as `article_pdf` and `trial_document`, which need `save`), and source enums and descriptions list only sources the serving mode allows.
- `readOnlyHint` is set per tool, true for every tool that cannot write a file.
- The hosting wrapper's tool catalog is refreshed from the typed `tools/list` before typed mode is offered there.

## Bench and merge

1. After 0632, build a Linux x86_64 binary from the branch tip by `pm job`; send path, SHA-256 and the tool-list size to Jarvis for the Biomed Agent Bench.
2. The bench compares typed tools, the single tool and the CLI in pi (strong model), pi (weaker model), then Claude Code.
3. Merge to main only on a bench result and Ian's decision. If typed wins, decide then whether it becomes the default for hosted.

## Review outcome

A fresh adversarial review on 2026-11-05 (`records/2026-11-05-typed-mcp-tools-review.md`) found the branch not ready for the bench, for reasons unrelated to the typed idea: replies carried one text line, 43% of argument descriptions were replaced by a generic type description, bad arguments got unrecoverable errors, and descriptions did not say what the tool does. The design above includes its corrections. Open questions it settled: keep all 58 commands as tools for the bench and decide from its tool-search misses; drop the output schema from typed `tools/list` by default; hosting needs the reply, research-tool and hosted-listing work, not only the setting.

## Work

Tickets 0624–0632 build the corrections on the branch; 0632 runs a harness check of what reaches the model before the bench build. None lands on main.
