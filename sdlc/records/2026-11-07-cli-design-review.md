# CLI design review

Question (Ian, via Jarvis, 2026-11-07): before 1.0, should the CLI grammar change, and should it match the MCP entity design from `records/2026-11-07-entity-tools-study.md`?

Build: 160521f5d. Run records are held privately with the team's data, not in this repo.

## Commands

The 61 registry rows are 58 distinct commands over 18 entities: 35 actions (search and get per entity, and variant normalize), 14 facets, 8 pivots to another kind of record (author papers, protein complexes and members, pathway genes, article citations, references, recommendations, citation evidence), and 4 filters that feed another entity's search (organization, biomarker search and get, author search).

## Inconsistencies

1. Word order: 38 rows are verb-first and 23 entity-first; protein uses both.
2. Nine section names share a word with a facet command. For protein, pathway and variant population the section is the first page of the same data. `get article X citations` (counts) and `article citations X` (citing papers) are different data under one word.
3. One spelling covers two kinds of request: `get gwas`, `get pgx`, and `get drug` with the WHO source.
4. A bare word means a gene after `search gwas`, free text after `search article`, and a query after `search author`.
5. Short flags are uneven across searches.

## Run

Hypothesis, written first: entity-first order (`biomcp gene get BRAF`) is no worse on rightness, turns within 20%, or failed calls. A thin shim mapped the new order onto the build and kept the old order as an alias.

Sonnet, two runs on five questions: today's order 111 turns, 7 failed calls, 9 of 10 fully right; entity-first 110 turns, 8 failed calls, 10 of 10. No failed call in either arm came from word order. Not refuted, no gain; the sample is small. The DeepSeek arm did not run because no usable login was available to the harness.

Two observations matter more than the totals. Entity-first typed today fails with an unknown-command error (one evaluation run gave up and used curl). And a model shown only entity-first examples still typed verb-first 20 times in one run, so removing the old order would cause failures.

## Options

- A. Entity-first before 1.0, old order kept as an alias: about 3 to 4 days, a new hosted pin, and the CLI disagrees with the MCP tool names until the MCP entity design lands. No measured gain.
- B. Keep verb-first and make entity-first run as an alias: about half a day, no change to hosting or docs.
- C. Leave as is.

Recommendation: B for 1.0, then make entity-first the documented form together with the MCP entity tools in 1.1, settling the section and facet names (starting with article citations) in that design.
