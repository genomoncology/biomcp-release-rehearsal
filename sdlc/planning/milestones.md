# Milestones

Each milestone says what it delivers, what it serves and how we know it is done. `sdlc/plan.md` orders the current milestone's stages. Every entity milestone runs its slices as `planning/slices.md` says, and each slice files its own tickets.

The 1.0 bar is parity with BioMCP 0.9's search and get: every useful behavior a clinician or agent relies on in 0.9 search and get works in 1.0, proved by the same harvested cases run against both binaries and compared by the facts they assert.

## 1. Foundations, genes and articles

Delivers: the workspace and crate layout, the lint ratchets, the shared transport with record and replay, the biodata conventions and schema generation, the outside-in runner, the JSON envelope and schema contract, the one MCP tool, then the gene slice and the article slice.

Serves: one typed, layered base that every later entity reuses, proved end to end by the simplest entity and by the entity where 0.9 fails a clinician most visibly.

Done when: `make landing-gate` passes with every ratchet at its 1.0 target. Every gene and article command and option has a harvested case that passes through the CLI and the MCP tool. Article search keeps every plain PubMed relevance hit or reports a partial source, AKT1 E17K's two key papers rank on page one, two full walks agree, and a warm page makes no request. The article adapters' sizes are measured against the estimate in `architecture.md`.

## 2. Variants and trials

Delivers: the variant slice and the trial slice. The variant slice settles where variant parsing lives. The trial slice builds the drill-downs that set the progressive disclosure pattern: a search returns trial headers, and subcommands drill into locations, outcomes, eligibility and other sections.

Serves: the questions that tie a patient's variant to evidence and to open trials.

Done when: every variant and trial command and option has a harvested case, trial counts match ClinicalTrials.gov's count for the same filter, and every 0.9 variant and trial command and option is ported, renamed or dropped by a named ruling.

## 3. Drugs, diseases and their bundles

Delivers: the drug, disease and adverse event slices, with the local data bundles each design keeps, such as EMA and WHO data, as local sources.

Serves: approval, label, safety and disease identity questions.

Done when: every command and option in these entities has a harvested case, each bundle the designs keep syncs and answers through the adapter shape, and a clinical reviewer has confirmed every changed drug identity answer.

## 4. The remaining entities

Delivers: the protein, pathway, phenotype, pharmacogenomics, GWAS, diagnostic, cell line, author and study slices. Study get returns the study and its files. Analysis and charts stay out.

Serves: the long tail of search and get that agents use around the main entities.

Done when: every command and option in these entities has a harvested case, and every 0.9 search and get command and option is ported, renamed or dropped by a named ruling.

## 5. Parity and history

Delivers: the parity run over both binaries, generated reference docs, a short hand-written guide, the recipes drawn from harvested cases, the release pipeline reused from 0.9, a release gate that compares schemas with the last released copy, and the history replacement.

Serves: users move from 0.9 to 1.0 without losing an answer they rely on.

Done when: the parity run shows every asserted fact that 0.9 gets right is also right in 1.0, every documented command runs against recordings with a zero exit, and Ian approves the release. The last step replaces the BioMCP repository's history with this repository's history, keeping its stars and release history.

## Later, unordered

Each waits for a consumer or for Ian. None has tickets.

- The browsable wiki over the store, served at `/` by `biomcp serve --http`.
- A Postgres store, once a shared deployment needs concurrent writers.
- Study analysis and charts.
- Python and TypeScript bindings for `biodata`.
- A merge of `biodata` with another biodata crate into one open-source crate.
- Bounded reading of PDF supplements and appendices, which experiment 497 showed hold variant evidence that BioMCP can only link today. Its 2026-10-27 rerun confirmed the gap remains.
- A sourced lookup from a gene and protein change to its rsID, so an empty LitVar protein lookup such as CDH1 D882N can reach the rsID's papers. Experiment 495's 2026-10-27 recheck raised it; 0335's recovery steps cover 1.0.
- A `schema validate` command that checks a JSON document against a published schema, and a field-path listing per schema for `jq` users. Experiment 501 raised both on 2026-10-28, following Ian's direction that the CLI tells an agent what JSON it will emit, shows any schema and validates a document against one.
- A standing agent evaluation run on each release: fuzzy multi-entity tasks modeled on MCP-Bench and InfoMosaic-Bench, scored on the same judged dimensions with paid model calls after Ian approves their cost, so regressions in agent results show before publication.
- FHIR patient reading and OMOP adapters, on a 1.1 branch (Ian, 2026-10-28). No released 0.9 version reads FHIR: v0.9.3 has no patient entity, and the FHIR work exists only on the earlier unreleased 1.0 line's branch. It is not a 1.0 regression; the 1.0 release notes say FHIR patient reading is planned for 1.1.
