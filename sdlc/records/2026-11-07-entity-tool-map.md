# Tool map for 1.0

Question (Ian, via Jarvis, 2026-11-07): 59 tools felt like too many, and `biomcp_reference` felt like an attic. Which set of about 20 tools is the most obvious way for an agent to reach every entity and pick the right tool and options? Ian's model: entity, then function (search, get, a facet or a pivot), then options.

Decision (Ian, via Jarvis, 2026-11-07): the 17-tool entity design below goes into 1.0 and replaces today's 19 typed tools. No further study.

Build measured: origin/main 4668f2b5d. Run records are held privately with the team's data, not in this repo.

## The map

17 tools: 13 entity tools plus `biomcp`, `help`, `search` and `fetch`. An entity tool takes `action` and one typed object named after the action, for example `{"action":"pgx","pgx":{"gene":"CYP2C19","source":"cpic"}}`. Each action's object is that command's own schema, unchanged, so every filter, section (the rendering choice) and paging option stays typed. An action name means the same command on every tool that carries it. Each tool's description is one line naming its sources and listing every action.

| Tool | Actions (search and get first, then facets, pivots, link views and helpers) |
| --- | --- |
| gene | search, get; views pgx, pgx_get, gwas |
| variant | search, get, normalize; facets assertions, population, recurrence; views gwas, gwas_get |
| disease | search, get; view gwas |
| drug | search, get; view pgx_get |
| trial | search, get, document; helpers organization_search, biomarker_search, biomarker_get |
| article | search, get; facets text, pdf, supplement; pivots citations, references, citation_evidence, recommendations; helpers author_search, author_get, author_papers |
| protein | search, get; facets features, isoforms, domains, structures, sequence; pivots complexes, complex_members |
| pathway | search, get; pivots genes, events |
| phenotype | search, get |
| adverse_event | search, get |
| diagnostic | search, get |
| cell_line | search, get; facet assays |
| study | search, get (cBioPortal); views gwas, gwas_get (GWAS Catalog studies) |
| biomcp | the command line, as today |
| help | today's `biomcp_help`, renamed |
| search, fetch | as today |

Links are views on the things they link: GWAS on gene, variant, disease and study; PGx on gene and drug. Helpers sit on the tool whose filters use them: author on article, organization and biomarker on trial. `biomcp_reference` is gone.

Coverage check (no run): all 58 typed commands are reachable from a named tool with a closed, typed schema, and no tool takes a free-form object. Pass. Two commands appear in more than one place: `search_gwas` (gene, variant, disease, study) and `get_gwas` (variant, study); `get_pgx` (gene, drug). That makes 63 placements.

## Every command, with its options

The machine-readable map is `map.json` ({tool: {action: command}}) and the full tool list `map-tools-list.json`, both beside the run data. The table below lists each placement and the command's typed options as origin/main declares them.

| Tool | Action | Command | Kind | Options (typed arguments) |
| --- | --- | --- | --- | --- |
| gene | search | search_gene | action | band, limit, locus_group, offset, query |
| gene | get | get_gene | action | input, section |
| gene | pgx | search_pgx | view (link) | cpic_level, gene, limit, offset, source |
| gene | pgx_get | get_pgx | view (link) | annotation_id, diplotype, drug_id, gene, kind, section, section_limit, section_offset, snapshot, source |
| gene | gwas | search_gwas | view (link) | extended_geneset, gene, include_child_traits, limit, offset, p_value, pool, pubmed, query, studies, study, trait, trait_id, variant |
| variant | search | search_variant | action | build, coding, condition, gene, has, limit, offset, protein, query, region, review_status, source |
| variant | get | get_variant | action | build, civic_cursor, civic_kind, civic_profile, evidence_level, evidence_status, input, limit, offset, resolver, section, section_limit, section_offset, tumor_type |
| variant | normalize | variant_normalize | action | build, hgvs, limit, offset, source |
| variant | assertions | variant_assertions | facet | assertion, build, input, limit, offset, source, version |
| variant | population | variant_population | facet | build, detail, input, limit, offset, resolver |
| variant | recurrence | variant_recurrence | facet | build, gene, input, limit, page, profile, sample_set, source, study |
| variant | gwas | search_gwas | view (link) | extended_geneset, gene, include_child_traits, limit, offset, p_value, pool, pubmed, query, studies, study, trait, trait_id, variant |
| variant | gwas_get | get_gwas | view (link) | id, section, section_limit, section_offset, snapshot |
| disease | search | search_disease | action | limit, offset, ontology, source, text |
| disease | get | get_disease | action | age_range, dataset, id, interval, ontology, period, race, section, section_limit, section_offset, sex, source |
| disease | gwas | search_gwas | view (link) | extended_geneset, gene, include_child_traits, limit, offset, p_value, pool, pubmed, query, studies, study, trait, trait_id, variant |
| drug | search | search_drug | action | classification, firm, indication, initiated_from, initiated_to, limit, name_selector, offset, prequalified_from, prequalified_to, query, reason, source, status |
| drug | get | get_drug | action | civic_cursor, civic_kind, civic_profile, document_id, evidence_level, evidence_status, input, label_section, product_number, section, section_limit, section_offset, source |
| drug | pgx_get | get_pgx | view (link) | annotation_id, diplotype, drug_id, gene, kind, section, section_limit, section_offset, snapshot, source |
| trial | search | search_trial | action | age, biomarker_code, biomarker_use, condition, condition_id, count_only, country, criteria, distance, exact_condition, exact_term, facility, gene, has_results, id, intervention, keyword, lat, limit, lon, next_page, offset, phase, query, sex, source, sponsor, sponsor_type, status, study_type, updated_since, updated_until, variant |
| trial | get | get_trial | action | input, limit, offset, pool, section, source |
| trial | document | trial_document | facet | file, input, replace, save |
| trial | organization_search | search_organization | helper | city, country, distance, family, lat, limit, lon, name, offset, pool, postal_code, source, state |
| trial | biomarker_search | search_biomarker | helper | limit, name, offset, pool, source |
| trial | biomarker_get | get_biomarker | helper | id, source |
| article | search | search_article | action | author, disease, drug, exact_term, exclude-retracted, gene, include-retracted, journal, limit, no_preprints, offset, open_access, pool, since, sort, source, text, type, until, variant |
| article | get | get_article | action | input, section |
| article | text | article_text | facet | find, input, offset, part, replace, save, variant |
| article | pdf | article_pdf | facet | input, replace, save |
| article | supplement | article_supplement | facet | file, find, input, offset, replace, rows, save, sheet, variant |
| article | citations | article_citations | pivot | limit, offset, pmid, pool, source |
| article | references | article_references | pivot | limit, offset, pmid, pool, source |
| article | citation_evidence | article_citation_evidence | pivot | cited, citing, fulltext, offset, paragraph_offset, pool |
| article | recommendations | article_recommendations | pivot | limit, negative, offset, pool, seeds |
| article | author_search | search_author | helper | limit, offset, pool, query, source |
| article | author_get | get_author | helper | ids |
| article | author_papers | author_papers | pivot | full, id, limit, offset, pool, section |
| protein | search | search_protein | action | gene, limit, offset, pool, query, reviewed |
| protein | get | get_protein | action | input, section, section_limit |
| protein | features | protein_features | facet | accession, limit, offset, pool, position, type |
| protein | isoforms | protein_isoforms | facet | accession, limit, offset, pool |
| protein | domains | protein_domains | facet | accession, limit, offset, pool, type |
| protein | complexes | protein_complexes | pivot | accession, limit, offset, pool |
| protein | complex_members | protein_complex_members | pivot | accession, complex, limit, offset, pool |
| protein | structures | protein_structures | facet | accession, limit, offset, pool |
| protein | sequence | protein_sequence | facet | isoform |
| pathway | search | search_pathway | action | fresh, gene, limit, offset, pool, query, source, top_level |
| pathway | get | get_pathway | action | fresh, id, pool, section, section_limit, section_offset |
| pathway | genes | pathway_genes | pivot | details, id, limit, offset, pool |
| pathway | events | pathway_events | facet | id, limit, offset, pool |
| phenotype | search | search_phenotype | action | limit, offset, source, text |
| phenotype | get | get_phenotype | action | id, section, section_limit, section_offset, snapshot, source |
| adverse_event | search | search_adverse_event | action | aggregate, drug, limit, manufacturer, offset, problem, product_code, reaction, received_from, received_to, section, serious, snapshot, source, text |
| adverse_event | get | get_adverse_event | action | drug_index, id, section, section_limit, section_offset, snapshot, source |
| diagnostic | search | search_diagnostic | action | biomarker, bundle_revision, condition_id, decision_from, decision_to, device_id, drug_application, gene, gene_id, initiated_from, initiated_to, limit, manufacturer, marker, name, offset, product_code, source, type |
| diagnostic | get | get_diagnostic | action | bundle_revision, id, section, section_limit, section_offset, source, supplement |
| cell_line | search | search_cell_line | action | disease, exclude_derived, gene, limit, mutation, offset, protein, query, species |
| cell_line | get | get_cell_line | action | id, section, section_limit, section_offset |
| cell_line | assays | cell_line_assay_count | facet | id |
| study | search | search_study | action | cancer_type, limit, offset, pool, text |
| study | get | get_study | action | gene, id, molecular_profile, section, section_limit, section_offset |
| study | gwas | search_gwas | view (link) | extended_geneset, gene, include_child_traits, limit, offset, p_value, pool, pubmed, query, studies, study, trait, trait_id, variant |
| study | gwas_get | get_gwas | view (link) | id, section, section_limit, section_offset, snapshot |
| biomcp | (one call) | biomcp | shared | args |
| help | (one call) | biomcp_help | shared | command |
| search | (one call) | search | shared | query |
| fetch | (one call) | fetch | shared | id |

## Size

By the o200k tokenizer, today's 19-tool list is 9,578 tokens and the map's 17-tool list is 19,405. Every hidden command now carries its full schema; nesting itself is not the cost (the 63 commands listed flat are 18,807). The repeated GWAS and PGx views add about 1,300. The top-line descriptions are 742 tokens in all. Size did not drive the shape: a tool-searching client loads only the tools it finds.

## Pilot results (pilot only; the study was stopped)

These ran through a thin proxy that serves the map over today's build, with the bench's answer flow, before Ian decided. They are a pilot, not a measurement.

- Five bench questions, one run each per design. Sonnet 5.5: 5 of 5 fully right on both; failed calls 51 with today's tools and 30 with the map; $0.87 and $0.69. DeepSeek v4.1 flash ran two questions before the plan changed: both right on both designs, fewer failed calls with the map.
- Main round, stopped part way: 32 short questions, one per tool-and-action group and weighted to the commands that sat behind `biomcp_reference`, keys written before any run. Completed: 45 DeepSeek runs per design (two passes, partly) and 31 Sonnet runs per design.
  - DeepSeek: fully right 45 of 45 with the map and 40 of 45 with today's tools, but today's five misses are four runs that read outside their folder (graded as leaked, fields right) and one that hit its cap on the GWAS question, not wrong answers. Right first call 40 of 45 with the map and 7 of 45 today; today's first call was `biomcp_help` in 14 runs and failed on usage in 15. Failed calls 32 of 489 against 92 of 676. Turns 355 against 489. Cost per right answer $0.0044 against $0.0045.
  - Sonnet: 31 of 31 fully right on both. Right first call 16 of 31 with the map and 6 of 31 today (Sonnet made no BioMCP call at all in 12 and 13 runs, answering with curl). First-call usage errors 2 against 9. Failed calls 59 against 67. Turns 214 against 238. Cost per right answer $0.027 against $0.034.
  - Spend: DeepSeek $0.43 in all, Sonnet $3.44 in all.
- What the logs show: with today's tools, hidden commands are reached by guessing `biomcp_reference` arguments or by the command-line tool, and most first-call failures are those guesses (for example `get_pgx` without `gene` and `source`, `variant_population` with `id` for `input`). With the map, first calls land on the right tool and action; the remaining failures are wrong values inside a typed field (a drug name where CPIC wants its RxNorm key), not the wrong shape. No run used the flat form (arguments beside `action`). The GWAS Catalog was down throughout, so the GWAS questions measure only the first call.

## Fit and cost

Ian has put the map in 1.0 in place of today's 19 tools, so hosted's tool names change with 1.0. Building it in the server is about 3 to 4 days: grouping and per-action dispatch in the typed catalog (`crates/biomcp/src/mcp/typed/`), next-call hints written in the new form, hidden-source filtering applied inside each action's schema, `help` naming, and tests and docs. Hosting needs a new pin and its tool-name checks updated.

## Naming, settled by the PM (2026-11-07)

1. Arguments stay in a typed object named after the action, as measured. No run misplaced them, and each action keeps its own schema.
2. `_get` reads one record wherever a tool has both a list and a record view (`pgx_get`, `gwas_get`, `biomarker_get`). Drug's single view keeps `pgx_get`, so one action name means one command everywhere.
3. Author stays as three actions on `article`. Authors matter here only as a way to reach papers.
4. GWAS Catalog studies stay on `study` as `gwas` and `gwas_get`. An agent looking for a study looks there.
5. Facets keep their names. Each tool description says that `get` returns the first page and the facet returns all of it. Article's `citations` description says it lists citing papers, not counts.
6. `help` replaces `biomcp_help`. `biomcp`, `search` and `fetch` keep their names, because ChatGPT requires the last two.
7. Tool names use underscores (`adverse_event`, `cell_line`), the form MCP clients handle everywhere. The command line keeps hyphens.
