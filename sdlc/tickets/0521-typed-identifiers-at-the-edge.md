# 0521 — Keep parsed identifiers typed after the edge

Status: OPEN.

Milestone: 6

Depends on: 0294

## Outcome

Queries that parse an identifier at the edge keep it as a typed value, so adapters never parse it again or map an impossible failure to an input error. The JSON and schemas stay byte-identical.

- Why: Jarvis's 1.0 code review (m17823, finding 1). `biomcp-types/src/query/disease.rs:98` keeps `id: Text` after line 162 parses it as a Curie, and `biomcp-adapters/src/monarch/mod.rs:55` parses it again and maps the failure to `InputError::Path`. About 14 sites repeat this, including diagnostic get (`query/diagnostic/get.rs:21` and `:150`), drug recall and variant assertions.
- Starts from: those sites and the parse-at-the-edge rule in the architecture document.
- Rules: each site stores a typed value, such as `DiseaseId { Concept(Curie), SeerSite(..) }`, that serializes to the same string. No wrapper exists only to forward. Remove the adapters' second parse and its error mapping.
- Proof: an empty reply diff and an unchanged schema comparison; `rg` finds no adapter that parses a query identifier the query already parsed.
- Changes: the query types, the adapters that read them, and their tests.
- Defers: nothing.
