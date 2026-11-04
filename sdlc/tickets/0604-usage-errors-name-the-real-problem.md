# Usage errors name the filter, not the source

Status: COMPLETE.
Landed: 6783aeeff4ebd7ad029166744f9db16c4ed57074
Milestone: 5

## Outcome

When a filter does not apply to the chosen source, the usage error names that filter, the sources that accept it, and the nearest filter the chosen source does accept; and a boolean flag followed by `true` or `false` is read as the flag's value, never as query text.

- Why: `planning/eval-gap-recheck-2026-11-04.md` cross-cutting finding, from items 4, 9 and 10. `search adverse-event ... --source maude --received-from ...`, `search drug artemether --source who-pq --initiated-from ...` and `search diagnostic --source fda-cdx --gene BRAF` each exit 2 blaming `--source`, which sends agents to change the one argument that was right. `--has-results true` turns `true` into the trial query.
- Starts from: `parser_usage` and `checked_invalid` in `crates/biomcp/src/grammar/errors.rs`, `QueryInvalid` field lists in `crates/biomcp-types/src/query/validation.rs`, and boolean flag parsing in `crates/biomcp/src/grammar/arguments.rs`.
- Rules: one general rule in the grammar, never per command. The error's option is the filter; its valid values are the sources that accept the filter; its next step is the same command with the filter swapped for the chosen source's equivalent when the registry declares one (for fda-cdx, `--gene` to `--biomarker`), otherwise `help` for that command. A boolean flag takes an optional literal `true` or `false`; any other following word stays positional. Changes to error text are the allowed exception to the command freeze; replies that succeed stay byte-identical.
- Proof: installed-binary cases for the three witness commands each name the filter, and the CDx case suggests `--biomarker`; `--has-results true`, `--has-results false` and bare `--has-results` give the expected requests; a wrong source for a source-only command still blames `--source`.
- Changes: `crates/biomcp/src/grammar/**`, `crates/biomcp-types/src/query/validation.rs`, `crates/biomcp-types/src/response/error.rs`, `crates/biomcp-types/schemas/error.json`, `tests/cases/grammar/**`; the grammar errors, the boolean parse, the error schema text if needed, and cases.
- Defers: nothing.
