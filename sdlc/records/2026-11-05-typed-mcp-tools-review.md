# Typed MCP tools review

2026-11-05. Fresh adversarial review, read-only, of `planning/typed-mcp-tools.md` and the branch `mcp-typed-tools` at d5b0e4ba1 against main. No build or run; sizes were estimated by mirroring the branch generator over its schemas (upper bounds, within about 10%).

Verdict: not ready for the bench. Tickets 0624–0632 carry the fixes.

1. High, confirmed: typed mode forces structured-only replies (`config.rs` `tool_reply_form`) and replaces text with one line (`typed.rs:35-48`), against the MCP 2025-11-25 tools guidance that structured results SHOULD also carry serialized JSON as text and against `docs/configuration.md:70`. Clients that pass only `content[0].text` to the model, notably on `isError`, see one line. Ticket 0624.
2. High, confirmed: `typed/arguments.rs` `resolve` replaces a `$ref` property with its target and drops the property's own description; 176 of 408 query properties have that shape, so `search_trial`'s `query`, `lat`, `lon`, `distance`, `sponsor`, `facility` and `next_page` all read as generic bounded text. Ticket 0625.
3. High, confirmed: invalid arguments get bare JSON-RPC `-32602` (`typed.rs:93-95`, `mcp.rs` admit, `http.rs`); the spec classes input validation as a tool execution error with `isError`. A string `"10"` for `limit` is refused, which the single tool never does. Ticket 0626.
4. Medium-high, confirmed: descriptions start "{verb} {entity} source records." and take the alphabetically first positional's sentence, so `search_adverse_event` describes its VAERS section and `get_drug` its sections. Ticket 0627.
5. Medium, estimated: input schemas are about 50% of the 130,784-character list and output schemas about 35% (about 800 characters per tool, identical but for the reply-ID enum); 31 of 58 tools exceed 2,000 characters; `search_trial` is about 8.7k, 3.0k of it a 226-value country enum; 73 optional properties carry nullable branches. Tickets 0625.
6. Medium, likely: `summary.rs` turns any get usage error without `next` into a search suggestion by word-shape guesses, including the too-broad error and hosted refusals, against `decisions/2026-10-24-no-text-heuristics`. Ticket 0628.
7. Medium, confirmed: the result size cap counts the whole result, so single mode's doubled JSON refuses near 70k characters while typed allows about 150k, skewing the bench. Ticket 0624.
8. Medium, design gap: typed mode drops `search` and `fetch`, which the 2026-10-25 amendment requires for ChatGPT. Ticket 0629.
9. Low-medium, design gap: hosted typed mode lists `article_pdf` and `trial_document`, which always refuse there; restricted mode still advertises refused sources; stdio sets `readOnlyHint:false` on all 58 tools though 4 can write; the hosting wrapper's static tool catalog logs typed calls as unknown. Ticket 0630.
10. Low, gate trust: the single-mode byte check passes when its baseline variable is unset; replay covers 35 tools, stops at the first passing case and skips cases the typed schema rejects; typed instructions use CLI syntax and there is no help tool. Tickets 0629 and 0631.

Sound: names come from the registry, are unique and fit client limits; flags-then-`--`-then-positionals spelling is safe; clap argument IDs match schema properties by construction; hosted policy is enforced at execution in both modes; cancellation and concurrency are unchanged; single-mode paths pass through; replay validates output with a real JSON Schema validator.

Skipped: build, scanners, advisory scan, gate history. Uncertain: exact sizes, and how pi and Quick handle structured content.
