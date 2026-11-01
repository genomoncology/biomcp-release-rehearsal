# 0547 — Give cache failures their own kinds

Status: OPEN.

Milestone: 5

Depends on: 0519

## Outcome

No cache failure that is not a SQLite failure reports `fetch.cache_database`. Each of the 131 non-SQL `CacheError::Database` sites that 0519 measured maps to a kind that names what failed.

- Why: code review finding 4 and release review D1. 0519 measured the remainder at d7aeabe7a and recorded the per-file list in its progress: request-key JSON 14, article-pool JSON 13, bundle-row JSON 2, metadata JSON 7, output size 2, filesystem 41, logic or missing state 34, poisoned sync 4, validation and bounds 14. Users and agents read `fetch.cache_database` as a database fault and act on it wrongly.
- Starts from: 0519's list, `CacheError` and `CacheProblem` in the fetch layer and 0519's broken-snapshot kind.
- Rules: add the fewest kinds that keep the categories apart, for example encoding (all JSON), filesystem, internal (logic, missing state, poisoned sync) and limit (output size, validation and bounds), each with a dotted code in the existing pattern. Real SQLite failures stay `fetch.cache_database`. Ian's no-compatibility stance applies, so no alias keeps the old code for the moved sites. Regenerate schemas with `make schema-generate`, and list the changed codes for the release notes in 0534. Do not touch `crates/biodata`.
- Proof: `rg` over `crates/` finds `CacheError::Database` only where a rusqlite error is mapped. One case per new kind shows its code. The case suite and the schema check pass.
- Changes: the fetch layer's cache errors, the mapped sites, generated schemas and the cases.
- Defers: nothing.
