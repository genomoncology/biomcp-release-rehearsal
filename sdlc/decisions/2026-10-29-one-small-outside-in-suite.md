# One small outside-in suite

Decided 2026-10-29 by Ian.

- The routine suite is about 400 command-line calls against recorded replies, plus about 150 Rust trials for what a command line cannot test, such as MCP framing, HTTP, streaming, TLS and cache contention. It replaces about 2,000 tests and runs in about a minute without coverage instrumentation.
- A list derived from the grammar names every command, option, section, global option and MCP tool, one argument refusal per command, and the shared failure kinds: not found, source unavailable, bad source data, offline miss and expired pool. Every item has at least one command-line call that checks answer values where the command returns values.
- Failure handling is tested once per kind overall, and once per source for the sources clinicians rely on most: trials, variants, drug labels and articles.
- Each case uses real recorded replies when the behavior can be shown with real data. Synthetic replies are only for failures that cannot be triggered on purpose. Every case with real recorded replies joins the nightly live check.
- Duplicate cases and runner trials that only restate a command-line call go. A retired command keeps one refusal case.
- The coverage floor resets to what the new suite reaches and does not drive test growth. The PM sets it.

This narrows `2026-10-23-outside-in-tests-and-one-small-runner.md`, which still holds: one small runner, YAML cases, recorded replies, no network in the routine suite. A 2026-10-29 read-only inventory found 608 required items, 383 covered by a command-line call, 225 not, and heavy duplication, such as 224 calls to `search article`.
