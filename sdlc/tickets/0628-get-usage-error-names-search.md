# A get command's usage error gives the search call, and typed guesses go

Status: OPEN.
Milestone: 5
Depends on: 0626

## Outcome

When a `get` command refuses an input that looks like a name rather than an ID, the CLI's own usage error carries the exact `search` next step for that input, and the typed layer's word-shape replacement guesses are deleted.

- Why: `records/2026-11-05-typed-mcp-tools-review.md` item 6. `summary.rs` turns any get usage error without `next`, including the too-broad error and hosted refusals, into a search suggestion by word-shape guesses, against `decisions/2026-10-24-no-text-heuristics`; the evals report item 1 asks for the next step in the CLI error itself.
- Starts from: `replacement` and its callers in `crates/biomcp/src/mcp/typed/summary.rs`, and the get usage errors in `crates/biomcp/src/grammar/errors.rs`.
- Rules: Branch only: build and land on `mcp-typed-tools`, never main; the frozen 1.0 candidate is untouched. The rule is general: a refused get ID whose input fails the ID shape gets a search step for the same entity and input; errors for other causes keep their own next steps. 0599's drug rule is the model.
- Proof: CLI cases for three entities show the search next step; the too-broad error and a hosted refusal carry no search suggestion; the typed summary translates the CLI step into a typed call.
- Changes: `crates/biomcp/src/grammar/errors.rs`, `crates/biomcp/src/mcp/typed/summary.rs`, `tests/cases/grammar/**`; the usage error, the deletion and cases.
- Defers: nothing.
