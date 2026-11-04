# `get drug` accepts a brand or generic name

Status: OPEN.
Milestone: 5

## Outcome

`get drug` accepts a brand or generic name as well as a source-native ID, resolves it through the chosen source's own name search, and answers the requested section in one call. With no `--source`, the requested section picks its source. When a name cannot be resolved, the usage error gives the exact next command for that input.

- Why: `planning/eval-recommendations-2026-11-04.md` item 1, the one finding shown above the noise level on its own. On main 2ffbeab2, `biomcp get drug imatinib approvals` exits 2 with `usage.arguments`, lists ten sources and offers only `help get drug`; 58 of 69 `get drug` calls in one evals arm hit this error. The evals patch with name matching cut a drug-regulatory question from 161k tokens (main with a map) to 82k.
- Starts from: `GetFields` and `DrugSource::accepts` in `crates/biomcp-types/src/drug/query.rs` (the input is "never a selected name", and `--source` is required), the per-source search in `crates/biomcp-query/src/drug.rs` and its sources (`drugsfda.rs`, `ema.rs`, `openfda_label.rs`, `native.rs`, `civic.rs`), and `parser_usage` and `checked_invalid` in `crates/biomcp/src/grammar/errors.rs`.
- Rules:
  - An input that matches the source's ID shape is used exactly as today, so every current ID call keeps its reply.
  - Any other input is a name. It is matched case-insensitively against the source's brand and generic names, using that source's existing search; no new source is called.
  - With no `--source`, a section served by one source uses that source. An identity-only call, or sections served by different sources, uses MyChem, the `search drug` default.
  - Ambiguity rule, stated in help and in the reply: an exact brand-name match ranks before an exact generic-name match; among equals, the original application (NDA or BLA before ANDA, then earliest approval) ranks first in Drugs@FDA, and the source's own search order decides elsewhere. The top record answers; the reply says it was chosen by name, and lists up to five other matches, with a total, as ready `get drug <id> --source <source>` steps.
  - No exact match: exit 2 with the usage error, whose `next` is the exact `search drug <input> --source <source>` command, never only `help`.
  - Salt forms: a generic name also matches an ingredient that is the input plus one trailing salt or ester word from a fixed USAN/USP list (mesylate, hydrochloride, sodium and the like), kept in one place and cited in help, so `imatinib` matches IMATINIB MESYLATE. Candidates come from the source's existing word search (m24458).
  - No silent fuzzy match: a partial name never answers a `get`.
- Proof: installed-binary cases. `get drug imatinib approvals` answers from Drugs@FDA with the brand NDA and lists other matches. `get drug Gleevec approvals --source drugsfda` and `get drug "imatinib mesylate" approvals` give the same record. One EMA and one openFDA label name case. One unresolvable name gives the exact search step. Every existing ID case is byte-identical.
- Changes: `crates/biomcp-types/src/drug/**`, `crates/biomcp-query/src/drug.rs`, `crates/biomcp-query/src/drug/**`, `crates/biomcp/src/grammar/errors.rs`, `crates/biomcp-types/schemas/drug-*.json`, `tests/cases/drug/**`, `tests/replies/drug/**`; the drug query type and schema, drug name resolution, the usage error's next step, help text and cases.
- Defers: name matching across several sources in one call.
