# Entity tools study

Question (Ian, via Jarvis, 2026-11-07): would one MCP tool per entity, with actions and facets, beat today's 19 typed tools? Today 14 commands are direct tools and about 44 sit behind `biomcp_reference`, whose `arguments` is a free object with no schema.

Build: 160521f5d, the build that scored 14 of 17 in the morning evaluation. Run records are held privately with the team's data, not in this repo.

## Facet map

The registry's 61 rows give 58 distinct typed commands. An entity design places them in 14 entity tools (gene, variant, disease, drug, trial, article, adverse event, diagnostic, protein, pathway, phenotype, gwas, study, cell line) plus `biomcp`, `biomcp_help`, `search` and `fetch`: 18 tools.

- Facets of an entity: article text, pdf, supplement, citations, references, citation evidence, recommendations; variant assertions, population, recurrence; protein features, isoforms, domains, complexes, members, structures, sequence; pathway genes and events; cell line assay counts; gene pgx.
- Filters: author on article; organization and biomarker on trial.
- Own tool, by their arguments: protein (needs a UniProt accession), pathway (needs a pathway id), phenotype (free text or an HP id), gwas (spans gene, variant, trait and study), study (needs a study id).

## Size

Today's list is 9,575 tokens by the o200k tokenizer (11,100 by Claude's counter). The entity list with full schemas for all 58 commands is 18,020 (26,449). The growth comes from typing the hidden commands, not from nesting; nesting is about 20% smaller than 58 flat tools. Both providers accepted the list; no top-level oneOf, anyOf or allOf.

## Run

Hypothesis, written first: the entity design answers more hidden-entity questions fully right with fewer failed calls; refuted if it gets fewer right or has as many failed calls.

Sonnet, one run per question and design, five questions (pgx, phenotype, population, study, pathway and gwas). Today's tools: 5 of 5 right. Entity: 4 of 5, the miss on the GWAS question while the GWAS Catalog was down. The DeepSeek arm did not run, because its provider key on the build machine was rejected as expired.

The sample is too small to decide. The logs support two narrower findings: typed schemas stop argument guessing for hidden commands (pgx was found from its schema instead of guessed three times), and a feature filed under another entity's name (pgx under gene) is harder to find.

## Outcome

Recommendation to Ian: ship 1.0 with typed mode as it is, and add the entity design in 1.1 as a new tool mode beside `typed`, so the 19 hosted names stay valid. Before 1.1, name facets where agents look for them, trim toward today's size, and rerun on more questions with both models. For 1.0, fix the wrong `biomcp_reference` example.
