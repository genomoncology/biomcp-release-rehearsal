# BioMCP 1.0

BioMCP 1.0 retrieves source records for cancer care and human health research through the CLI and MCP. Upgrade existing `biomcp-cli` installations with their package manager. FHIR patient reading is planned for 1.1 and is unavailable in 1.0.

## Breaking changes for 0.9 users

JSON replaces Markdown as the default CLI output. Select `--format md` or set `"format": "md"` in configuration for Markdown. MCP always returns JSON. Scripts must read the new response envelopes and entity schemas instead of parsing 0.9 output.

MCP uses `single` mode by default: the `biomcp` tool accepts a CLI argument array, alongside the research `search` and `fetch` tools. Opt into typed tools with `"mcp_tool_mode": "typed"` in configuration or `biomcp serve --tool-mode typed`. The command-line setting overrides configuration. Typed mode exposes tools generated from the command registry, such as `search_gene` and `get_variant`, plus `biomcp_help`, `search` and `fetch`. Available tools and source choices follow the server's policy; use its `tools/list` response for the current catalog.

Command help now includes summaries from shared command metadata, and typed tool descriptions use those summaries. `biomcp_help` accepts a command path and returns the existing command help. For `get` errors, follow the returned `next` action: an identifier with the accepted shape but no source record keeps its existing help advice.

Run `biomcp mcp-config` to generate stdio wiring for the installed executable. The client starts `biomcp serve` with the selected tool mode. HTTP uses `biomcp serve --http [ADDR]` with `/mcp`; it refuses browser origins and requires allowed hosts for non-loopback listeners. Hosted deployments need caller credential isolation and source pacing configuration. See the [MCP setup](https://github.com/genomoncology/biomcp/blob/main/README.md#connect-an-mcp-client).

The [upgrade table](https://github.com/genomoncology/biomcp/blob/main/sdlc/planning/upgrade.md#commands) lists every 0.9 leaf command and each removed or deferred option, with its replacement or retirement ruling. Read the linked entity sections for positional sections and aliases. The [1.0 command reference](https://github.com/genomoncology/biomcp/blob/main/docs/reference/commands.md) gives accepted arguments. Major changes include:

- Replace global `-j` and `--json` with `--format json`. Replace `--no-cache` with `--fresh`; this requests fresh source replies under the 1.0 store rules.
- Run `mcp-config` without `--absolute-path`; it always prints the executable's absolute path. Keep `--client CLIENT` for client-specific configuration.
- Use positional text for `search author` and `search cell-line`; their `-q` and `--query` options retire.
- Use `get ENTITY` with repeated IDs instead of batch wrappers. Use trailing section words instead of batch `--sections`.
- Use `sync SOURCE` for stored bundles and `store clean` for expired replies and collections. `cache clean --dry-run`, `--max-age` and `--max-size` have no maintenance-policy replacement.
- Use returned `--next-page` and `--pool` commands for continuation. Keep the returned argument arrays and policy settings together. Trial `--offset` is continuation bookkeeping and requires `--next-page` when nonzero. Hidden sessions do not identify the new collections.
- Use explicit entity search and get commands. Discover, chart generation, local study analysis, self-update and automatic text-based classification or identity merging retire. Patient reading moves to 1.1.
- Trial discovery uses literal `--variant` and `--criteria`; mutation and biomarker aliases and semantic therapy-line filters retire. Discovery never determines eligibility.
- Article downloads use exact file selections and `--save PATH`. Text readers, PDFs and supplements have separate commands. HTTP MCP refuses file saves.
- Variant normalization uses `--source`, `--hgvs` and explicit builds. Variant source assertions, population and recurrence use their named commands. No source score becomes a clinical classification.

## Added population fields

Variant population replies add source-published `homozygote_count` for exome and genome totals and `joint` AC, AN and homozygote totals. A joint section with `state: empty` means gnomAD supplied no joint data; BioMCP never adds exome and genome counts. `dataset_version` names a release only when matching same-response exome and genome totals independently report the same version. Joint counts have no invented independent release or frequency. Existing counts, filters and ancestry pagination retain their meanings.

## Hosted use and source terms

Hosted HTTP requests accept `x-biomcp-use: restricted`, `commercial` or `academic`. Missing headers select restricted use. Restricted use admits open and attribution sources and commercially permitted full-text records; caller keys do not enable noncommercial or separately licensed sources. Commercial use also admits a noncommercial or licensed source only with its required caller key; optional rate-limit keys never enable blocked sources. Academic use retains every configured source. The JSON setting `hosted_max_use` sets the highest hosted mode (`restricted`, `commercial` or `academic`, default `academic`). Deployments serving only commercially safe sources set `"hosted_max_use": "restricted"`; caller declarations and keys cannot widen that ceiling. Higher declarations are refused with an error naming the allowed modes. Server exclusions always apply. Ordinary CLI and stdio defaults keep every configured source available.

The published `restricted` source label is replaced by the actual `open`, `attribution`, `noncommercial`, `licensed` or `per-record` category. Existing `disable_sources: ["restricted"]` configurations retain exactly their former source and component exclusions as a legacy alias. Current named groups follow corrected labels:

- `open` gains `car`, `clinvar`, `erepo`, `gwas`, `hotspots`, `litsense`, `mutalyzer`, `mygene`, `nci`, `pubmed`, `pubtator`, `variantvalidator` and `cpic`, plus the MyVariant `clinvar` and `snpeff` components. The MyVariant service is open while each carried component retains its own terms.
- `noncommercial` loses `cpic` and gains `who-pq` and the MyVariant `snpedia` component. Its existing DDInter, ClinPGx, Semantic Scholar API, ORCID, WHO IVD, CADD and academic dbNSFP exclusions remain.
- The `licensed` label applies to `alphagenome`, `oncokb`, the MyVariant `cosmic` component and the MyDisease `omim` component.
- The `per-record` label applies to PMC OA and Europe PMC full text. Publisher PDFs obtained through Semantic Scholar also retain their own record licenses, separately from the API's noncommercial terms.

ClinicalTrials.gov, cBioPortal, VAERS, FAERS MedDRA terms and MAUDE GMDN terms carry attribution labels and remain available in restricted use. CPIC has its own terms independently of ClinPGx. CADD and COSMIC terms links are corrected, and SnpEff is identified by its MIT license. NCBI and openFDA keys continue to raise the limits of free sources. Previously stored continuations resolve old labels through the current source registry when read.

## Configuration migration

Create a JSON configuration file at the platform path described in the [upgrade guide](https://github.com/genomoncology/biomcp/blob/main/sdlc/planning/upgrade.md#configuration-and-keys). Unknown fields fail. Replace the old `cache.toml` fields `[cache].dir`, `[cache].max_size`, `[cache].min_disk_free` and `[cache].max_age_secs`. The old cache mode, size and disk policies no longer apply. The following 0.9 operator variables no longer configure 1.0:

- `BIOMCP_CACHE_DIR`, `BIOMCP_CACHE_MODE`, `BIOMCP_CACHE_MAX_AGE`, `BIOMCP_CACHE_MAX_SIZE`, `BIOMCP_CACHE_MIN_DISK_FREE`: use the platform store and explicit `store clean`; there is no user cache eviction policy.
- `BIOMCP_DISABLE_SOURCES` and `BIOMCP_DISABLE_KEGG`: move exclusions to JSON `disable_sources` or repeat `--disable-sources SOURCE`. The old environment variables no longer exclude sources. Replace the old `non-commercial`, `license-required` and `commercial-restricted` group names with the current `noncommercial` and `restricted` groups as appropriate. To exclude both groups, set `{"disable_sources": ["noncommercial", "restricted"]}`. Review source terms again before upgrading because group membership follows the current source registry.
- `BIOMCP_DEFAULT_ASSEMBLY`: pass the command's explicit `--build` when required.
- `BIOMCP_STUDY_DIR`, `BIOMCP_DDINTER_DIR`, `BIOMCP_EMA_DIR`, `BIOMCP_WHO_DIR`, `BIOMCP_CVX_DIR`, `BIOMCP_GTR_DIR`, `BIOMCP_GENCC_DIR`, `BIOMCP_WHO_IVD_DIR`: use supported `sync SOURCE` bundles. Unsupported bundles and local study analysis have no 1.0 replacement.
- `BIOMCP_FHIR_BASE`: patient reading is planned for 1.1.
- `BIOMCP_CA_BUNDLE` and its `SSL_CERT_FILE` fallback: set `ca_bundle` to add PEM roots to platform trust.
- `BIOMCP_VARIANT_QUERY_GENE_ROUTING`: use explicit variant selectors.
- `DISGENET_API_KEY`: the DisGeNET gene section retires.
- `RUST_LOG`: the 0.9 tracing filter has no 1.0 logging configuration equivalent.

The source credential names `ONCOKB_TOKEN`, `NCBI_API_KEY`, `S2_API_KEY`, `NCI_API_KEY`, `OPENFDA_API_KEY`, `ORCID_ACCESS_TOKEN`, `UMLS_API_KEY` and `ALPHAGENOME_API_KEY` remain recognized. Use `biomcp key set SOURCE` to keep values out of shell history. Stored keys override environment values. Use JSON `format`, `disable_sources`, `keys`, `ncbi_email`, `ca_bundle`, `proxy` and `http.allowed_hosts` for the corresponding settings. Proxy environment variables remain supported. Every source starts enabled; commercial users must disable the `noncommercial` and `restricted` groups and review publisher terms.

## Output migration

Every JSON reply advertises its schema identity. Read `result` for records, `sources` for source outcomes and terms, `problems` for typed failures, `status` for completeness, `page` for paging and `next` for argument arrays. Commands have their own typed result shapes; 0.9 fields are not a compatibility interface. Missing optional values stay absent. Source-native identifiers and assertions remain separate when sources disagree. Counts preserve their source scope and never establish clinical eligibility or an assayed denominator.

The 0.9 metadata and error fields change as follows:

| 0.9 field | 1.0 migration |
| --- | --- |
| Flattened entity fields at the reply root | Read the command's `result` and its typed item or section shape. Gene cards, for example, use `result.items[].answer.gene`; article and trial cards have different schemas. |
| `_meta.evidence_urls[].label`, `_meta.evidence_urls[].url` | Read record URLs and source provenance from the result's schema. There is no universal evidence URL list. |
| `_meta.next_commands[]` | Read `next[].args` arrays with typed actions; do not split shell command strings. |
| `_meta.suggestions[]` | Read explicit next actions and source problems. No generated suggestion text is a compatibility field. |
| `_meta.section_sources[]` | Read `sources[]` and the source-native identities in each result. |
| `_meta.source_status[]`, `_meta.warnings[]`, `_meta.notes[]` | Read typed `sources[]`, `problems[]` and `status` for source failures, warnings and stale data. The old message strings are not compatibility fields. |
| `_meta.workflow`, `_meta.workflow_rationale`, `_meta.workflow_playbook` | Removed with automatic workflow selection. |
| `_meta.discovery_sources[]` | Removed with discover. Use explicit entity and source queries. |
| `_meta.not_found` | Read the typed problem and status for the requested command. |
| `_meta.alias_resolution.kind`, `requested_entity`, `query`, `canonical`, `canonical_id`, `confidence`, `match_tier`, `sources`, `candidates` | Read each command's explicit resolution and candidates. The old confidence tiers and cross-entity alias inference retire. |
| `error.code`, `error.message`, `error.source`, `error.recovery`, `error.limit_bytes`, `error.limit`, `error.unit` | Read `problems[]` and returned next actions. Error codes and limit fields use the advertised command schema. |
| Variant `filter_evaluation`, `requested_variant`, `resolution` at the reply root | Read the variant result's checked request, source outcomes and native candidates. Do not reuse old top-level projections. |
| Preview `values`, `returned`, `total`, `has_more`, `omitted_oversized` | Read the section or page schema and follow its continuation. A preview count never supplies a population denominator. |

The seven 0.9 public entity schemas also change their entire root contract: gene, variant, article, trial, drug, disease and pathway records now sit inside command-specific envelopes. Their [1.0 schemas](https://github.com/genomoncology/biomcp/tree/main/crates/biomcp-types/schemas) define every field. Source-specific sections no longer flatten into those seven shared records. Rebuild integrations against their command schema rather than treating a field of the same name as the same contract.

The following 0.9 entity fields leave the reply root. Every nested path beneath these roots also changes. Read each command's `result` schema for the new nesting and source-specific values. Derived clinical summaries and local analysis fields have no replacement unless their source record appears in that schema.

| Entity | 0.9 root fields replaced |
| --- | --- |
| gene | `symbol`, `name`, `entrez_id`, `ensembl_id`, `location`, `genomic_coordinates`, `omim_id`, `uniprot_id`, `summary`, `gene_type`, `aliases`, `clinical_diseases`, `clinical_drugs`, `clingen`, `gencc` |
| variant | `id`, `genome_build`, `genome_build_provenance`, `gene`, `hgvs_p`, `legacy_name`, `hgvs_c`, `rsid`, `cosmic_id`, `significance`, `significance_source`, `significance_evaluated`, `significance_note`, `clinvar_id`, `clinvar_review_status`, `clinvar_review_stars`, `conditions`, `consequence`, `cadd_score`, `polyphen_pred`, `sift_pred`, `conservation`, `expanded_predictions`, `population`, `cosmic_context`, `cgi_associations`, `civic`, `clinvar_conditions`, `clinvar_condition_reports`, `top_disease`, `cancer_frequencies`, `cancer_frequency_source`, `supporting_pmids`, `prediction` |
| article | `pmid`, `pmcid`, `doi`, `title`, `authors`, `journal`, `date`, `citation_count`, `publication_type`, `open_access`, `abstract_text`, `annotations`, `pubtator_fallback`, `source`, `score`, `is_retracted` |
| trial | `nct_id`, `source`, `title`, `status`, `phase`, `study_type`, `conditions`, `interventions`, `arms`, `arm_intervention_assignments`, `sponsor`, `enrollment`, `summary`, `start_date`, `completion_date`, `eligibility` |
| drug | `name`, `drugbank_id`, `chembl_id`, `unii`, `drug_type`, `mechanism`, `mechanisms`, `approval_date`, `approval_date_raw`, `approval_date_display`, `approval_summary`, `brand_names`, `route`, `targets`, `target`, `indications`, `pharm_classes`, `top_adverse_events` |
| disease | `id`, `name`, `definition`, `synonyms`, `parents`, `top_genes`, `associated_genes`, `treatment_landscape`, `recruiting_trial_count`, `top_variant`, `xrefs` |
| pathway | `id`, `name`, `species`, `summary`, `genes`, `events`, `enrichment` |

Cache failures now distinguish `fetch.cache_encoding`, `fetch.cache_filesystem`, `fetch.cache_internal` and `fetch.cache_limit`. Their `CacheProblem` wire values are `encoding`, `filesystem`, `internal` and `limit`. Genuine SQLite failures keep `fetch.cache_database`; unreadable stored snapshots keep `fetch.cache_snapshot`. The former use of the database code for these other failures has no compatibility alias. Handle each typed code instead of treating every local failure as a database failure.

Identifier schemas now enforce the same bounds as their checked values. These exact restrictions also apply wherever a command embeds the identifier, including `VariantInput`.

| Identifier | Accepted numeric range |
| --- | --- |
| VCV version | 1 through 4,294,967,295 |
| RCV version | 1 through 4,294,967,295 |
| SCV version | 1 through 4,294,967,295 |
| COSMIC number | 1 through 18,446,744,073,709,551,615 |
| ClinVar variation number | 1 through 4,294,967,295 |
| ClinVar allele number | 1 through 4,294,967,295 |
| ClinVar branch of `VariantId` | 1 through 4,294,967,295 |
| OMIM number | Six digits with a first digit from 1 through 6 |

`VariantInput` applies the ClinVar variation bound to both bare numbers and `clinvar:` IDs. Other identifier namespaces keep their own bounds. Author IDs require `semantic-scholar:ID` or a checked `orcid:ID`; `semanticscholar:ID` is removed from `AuthorId`, embedded author records and selector schemas. Author errors now describe only the canonical ID spellings and pattern. Their error code stays the same. The `--source semanticscholar` source selector remains accepted.

ClinVar VCV, RCV and SCV equality and hashing include the version. Rust callers can use `same_accession` to compare accessions across versions. Variable-width accession output uses its canonical spelling, so `LRG_0001t01` becomes `LRG_1t1`. Fixed-width accessions such as `NM_000546` retain their zero padding.

The candidate's schema identity changes appear below when these notes are staged. They compare current schemas with the preserved `release/migration-schema-baseline.tar.gz` history. Refreshing the compatibility baseline for a release does not replace that history. The [schema directory](https://github.com/genomoncology/biomcp/tree/main/crates/biomcp-types/schemas) defines output fields and required values. In particular, adverse-event snapshot fields become optional when a cache write fails; callers must handle their absence. A major bump permits an intentional breaking schema change. The schema check does not prohibit all breaking changes.

Posted trial results are an explicit exception to the 1.0 command freeze. `get trial ID results` adds the ClinicalTrials.gov results section, including native group values, units, parameter and dispersion labels, analyses and adverse-event tables. The trial-cards reply schema adds `card.results_first_posted` and `sections.results`; the trial-get schema adds the `results` section. The card date is the first public posting date, not submission or last update. Values are reported source values, with no clinical calculations.
