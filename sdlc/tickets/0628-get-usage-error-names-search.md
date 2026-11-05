# A get command's usage error gives the search call, and typed guesses go

Status: OPEN.
Milestone: 5
Depends on: 0626

## Outcome

The typed layer's word-shape replacement guesses are deleted, and a `get` usage error carries a `search` next step only where existing ID parsing proves the input is lexically not an ID and a recorded case shows that call executes meaningfully; everywhere else the frozen `help` advice stays.

- Why: `records/2026-11-05-typed-mcp-tools-review.md` item 6. `summary.rs` turns any get usage error without `next`, including the too-broad error and hosted refusals, into a search suggestion by word-shape guesses, against `decisions/2026-10-24-no-text-heuristics`; the evals report item 1 asks for the next step in the CLI error itself.
- Starts from: `replacement` and its callers in `crates/biomcp/src/mcp/typed/summary.rs`, and the get usage errors in `crates/biomcp/src/grammar/errors.rs`.
- Scope for 1.0 (m25561, option B): the search step is given only where existing ID parsing proves the input is lexically not an ID and a recorded case shows the suggested call executes meaningfully; everywhere else, including valid-shape absent IDs such as `NCT00000000`, numeric author IDs and a literal `--help`, the frozen `help` advice stays. A lexical-shape API from BioData waits until after 1.0.
- Rules: Branch only: build and land on `mcp-typed-tools`, never main; the frozen 1.0 candidate is untouched. Errors for other causes keep their own next steps, and 0599's drug rule is unchanged.
- Proof: each case that gains a search step is a named witness whose call executes, and cases without proven lexical rejection (such as `NCT00000000`, `get author 123`, a literal `--help`) show the frozen help advice; the too-broad error and a hosted refusal carry no search suggestion; the typed summary translates the CLI step into a typed call.
- Changes: `crates/biomcp/src/grammar/errors.rs`, `crates/biomcp/src/mcp/typed/summary.rs`, `tests/cases/grammar/**`; the usage error, the deletion and cases.
- Defers: nothing.
