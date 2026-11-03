# Command reference

Generated from the command grammar and schema help. Run `make docs-generate` to update.

Options shown on the root command apply to its subcommands. Query sections and source rules follow each command.

## biomcp

```text
Retrieve source records for biomedical evidence questions.

Usage: biomcp [OPTIONS] <COMMAND>

Commands:
  search      Search source records by entity and checked filters.
  get         Retrieve one entity and its source sections.
  author      Read papers attributed to a source author record.
  cell-line   Read source cell line assay counts.
  protein     Read protein sequences, features and source relationships.
  pathway     Read pathway genes and source events.
  article     Read article text, references, citations and files.
  trial       Read trial groups and public documents.
  variant     Read source variant assertions, populations and recurrence.
  sync        Synchronize a local source bundle.
  skill       Read embedded guidance and worked examples. No source requests. Overview and catalog are bounded by the embedded payload; install and status return one summary. Install writes only inside DIR and refuses symbolic links and parent traversal.
  version     Print version; --verbose adds build, schema and path provenance. No source requests or key reads.
  mcp-config  Print MCP configuration with this executable’s absolute path. JSON formats name their top-level object and retain released client aliases. The codex client prints its shell registration command. No source requests or key reads.
  health      Report source terms and bounded reachability.
  store       Manage local cached replies and snapshots.
  key         Register and manage source credentials without showing values.
  serve       Serve MCP over standard input or HTTP.
  schema      List every schema identity and description with biomcp schema, or print one schema artifact with biomcp schema <id>.
  help        Describe commands and their arguments.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

-h, --help: Print scoped command help without source requests.
--version: Print the installed version without source requests.

```

search: Search source records by entity and checked filters.
get: Retrieve one entity and its source sections.
author: Read papers attributed to a source author record.
cell-line: Read source cell line assay counts.
protein: Read protein sequences, features and source relationships.
pathway: Read pathway genes and source events.
article: Read article text, references, citations and files.
trial: Read trial groups and public documents.
variant: Read source variant assertions, populations and recurrence.
sync: Synchronize a local source bundle.
skill: Read embedded guidance and worked examples
version: Print version; --verbose adds build, schema and path provenance
mcp-config: Print MCP configuration with this executable’s absolute path
health: Report source terms and bounded reachability.
store: Manage local cached replies and snapshots.
key: Register and manage source credentials without showing values.
serve: Serve MCP over standard input or HTTP.
schema: List every schema identity and description with biomcp schema, or print one schema artifact with biomcp schema <id>.
help: Describe commands and their arguments.

- `--format`: Choose JSON or Markdown output; configuration can change the default.
- `--disable-sources`: Exclude source names or source terms groups.
- `--source_pacing`: Override source pacing with a JSON map of rate, keyed_rate and in_flight values.
- `--config`: Read configuration from this file.
- `--offline`: Read local data without source requests.
- `--fresh`: Request fresh source data instead of cached replies.
- `--help`: Print scoped command help without source requests.
- `--version`: Print the installed version without source requests.

## biomcp search

```text
Search source records by entity and checked filters.

Usage: biomcp search [OPTIONS] <COMMAND>

Commands:
  biomarker      Find NCI biomarker concepts by name or synonym. The public tier returns codes, name, type, trial count, synonyms, parent_ids, ancestor_ids and eligibility_criterion. These concepts differ from diagnostic --biomarker filters and literal trial --variant text. Use search trial --source nci --biomarker-code CODE for trials that list an exact concept. Native criteria do not establish patient eligibility.
  author         Literal author name query and native window; positional text maps to query.
  organization   Find NCI organizations. Names are not unique; trial counts cover every status, including closed trials. No organization ID or get is available.
  cell-line      One literal cell line search, with native order and no total claim.
  gwas           Source associations in default p-value-ascending order. Reports no personal risk, causal variant or gene, or clinical interpretation of a statistic.
  protein        Search five reviewed human UniProt headers by literal text or exact gene.
                 No variant effect, structure, interaction or clinical meaning is inferred. No bioinformatics tool runs.
  phenotype      Search literal human HP term candidates through Monarch with bounded native pages. A term lookup answers terminology and does not establish a diagnosis.
  pathway        Search Reactome (default) or WikiPathways human pathways. Membership names a source and release; it implies no therapy, response or actionability. No enrichment is calculated.
  study          Literal stored metadata search; text is never parsed as a query language. No frequency, denominator, survival or comparison is calculated. Source counts keep their native names.
  pgx            Checked gene-keyed CPIC pair search. No phenotype, phase or dose is calculated.
  diagnostic     Search five source records by default. FDA device recalls cover every medical device; totals count recall records, without harm inference.
  adverse-event  Search five report headers by default. MAUDE accepts device text or a manufacturer, product-code or problem filter. One report may list several devices; filters may match different devices. VAERS accepts MMR, COVID19, COVID-19 vaccine or one exact qualified native influenza code, matched without case; use influenza to list codes and labels. Tables remain separate and are never summed. One report may list several vaccines. VAERS returns five reported symptom rows by default; seriousness and age select independent tables. Counts are reports, not people. Reports do not show that a vaccine caused an event and cannot measure risk or compare vaccine safety.
  disease        Literal discovery text or an explicit SEER catalog page.
  drug           Search one drug source. FDA drug recalls accept a name or native firm, classification, status, reason and initiation dates; counts are recall records. Other sources require a name.
  gene           Search genes by text, cytoband, or their intersection.
  article        Checked search selectors and pool cursor.
  trial          Search ClinicalTrials.gov or keyed NCI with native counts. NCI --facility matches exact organization names; a listed site is not necessarily enrolling.
  variant        Discover source candidates using one declared selector, without interpreting free text.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Search source records by entity and checked filters.


## biomcp search biomarker

```text
Find NCI biomarker concepts by name or synonym. The public tier returns codes, name, type, trial count, synonyms, parent_ids, ancestor_ids and eligibility_criterion. These concepts differ from diagnostic --biomarker filters and literal trial --variant text. Use search trial --source nci --biomarker-code CODE for trials that list an exact concept. Native criteria do not establish patient eligibility.

Usage: biomcp search biomarker [OPTIONS] --source <source>

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --limit <limit>
          Maximum headers per local page.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --name <name>
          Native partial name or synonym search.

      --offset <offset>
          Offset within the complete retained observation.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --pool <pool>
          Snapshot identity retained by continuation commands.

      --source <source>
          Explicit keyed NCI source.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Find NCI biomarker concepts by name or synonym. Sources: nci.

- `--limit`: Maximum headers per local page.
- `--name`: Source partial name or synonym search.
- `--offset`: Offset within the complete retained observation.
- `--pool`: Snapshot identity retained by continuation commands.
- `--source`: Explicit keyed NCI source.

## biomcp search author

```text
Literal author name query and native window; positional text maps to query.

Usage: biomcp search author [OPTIONS] <query>

Arguments:
  <query>
          Literal query from the positional argument.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

  -l, --limit <limit>
          Bounded source page size.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Native offset.

      --pool <pool>
          Retained page identity.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --source <source>
          Native author source.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Exact author name query and source window; positional text maps to query. Sources: semantic-scholar.

- `--limit`: Bounded source page size.
- `--offset`: Source offset.
- `--pool`: Retained page identity.
- `query`: Exact query from the positional argument.
- `--source`: Source author source.

## biomcp search organization

```text
Find NCI organizations. Names are not unique; trial counts cover every status, including closed trials. No organization ID or get is available.

Usage: biomcp search organization [OPTIONS] --source <source>

Options:
      --city <city>
          Native organization city.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --country <country>
          Native organization country.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --distance <distance>
          Positive radius with explicit mi or km units.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --family <family>
          Native organization family.

      --lat <lat>
          Radius center latitude in degrees; requires lon and distance.

      --limit <limit>
          Five headers by default, at most fifty.

      --lon <lon>
          Radius center longitude in degrees; requires lat and distance.

      --name <name>
          NCI partial organization name; names are not unique.

      --offline
          Read local data without source requests.

      --offset <offset>
          Offset within the complete retained organization snapshot.

      --fresh
          Request fresh source data instead of cached replies.

      --pool <pool>
          Snapshot identity retained by continuation commands.

      --postal-code <postal_code>
          Native postal code.

      --source <source>
          Explicit keyed NCI source.

      --state <state>
          Native state or province.

```

Find NCI organizations. Sources: nci.

- `--city`: Source organization city.
- `--country`: Source organization country.
- `--distance`: Positive radius with explicit mi or km units.
- `--family`: Source organization family.
- `--lat`: Radius center latitude in degrees; requires lon and distance.
- `--limit`: Five headers by default, at most fifty.
- `--lon`: Radius center longitude in degrees; requires lat and distance.
- `--name`: NCI partial organization name; names are not unique.
- `--offset`: Offset within the complete retained organization snapshot.
- `--pool`: Snapshot identity retained by continuation commands.
- `--postal-code`: Source postal code.
- `--source`: Explicit keyed NCI source.
- `--state`: Source state or province.

## biomcp search cell-line

```text
One literal cell line search, with native order and no total claim.

Usage: biomcp search cell-line [OPTIONS] <query>

Arguments:
  <query>
          Literal source query from the positional argument.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

  -l, --limit <limit>
          Native page size.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Native start offset.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

One exact cell line search, with source order and no total claim. Sources: cellosaurus.

- `--limit`: Source page size.
- `--offset`: Source start offset.
- `query`: Exact source query from the positional argument.

## biomcp search gwas

```text
Source associations in default p-value-ascending order. Reports no personal risk, causal variant or gene, or clinical interpretation of a statistic.

Usage: biomcp search gwas [OPTIONS] [query]

Arguments:
  [query]
          Exact native mapped gene; legacy positional query aliases this field.

Options:
      --extended-geneset
          Source extended gene set; requires gene.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

  -g, --gene <gene>
          Exact native mapped gene; legacy positional query aliases this field.

      --include-child-traits
          Source child-trait expansion; requires trait_id.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

  -l, --limit <limit>
          Maximum association rows.

      --offset <offset>
          Native association offset.

      --p-value <p_value>
          Inclusive positive probability in scientific notation. Requires the default p-value-ascending order; no sort option is available. The qualifying total stays unknown until the ordered walk reaches its end.

      --pool <pool>
          Retained threshold walk identity; requires a threshold.

      --pubmed <pubmed>
          Exact source publication identity.

      --offline
          Read local data without source requests.

      --study <study>
          Exact source study accession.

      --fresh
          Request fresh source data instead of cached replies.

      --trait <trait>
          Literal native efo_trait label; never searches reported_trait prose.

      --trait-id <trait_id>
          Exact native ontology identity.

      --variant <variant>
          Exact source rsID.

```

Source associations in default p-value-ascending order. Sources: gwas.

- `--extended-geneset`: Source extended gene set; requires gene.
- `--gene`: Exact source mapped gene; legacy positional query aliases this field.
- `--include-child-traits`: Source child-trait expansion; requires trait_id.
- `--limit`: Maximum association rows.
- `--offset`: Source association offset.
- `--p-value`: Inclusive positive probability in scientific notation. Requires the default p-value-ascending order; no sort option is available. The qualifying total stays unknown until the ordered walk reaches its end.
- `--pool`: Retained threshold walk identity; requires a threshold.
- `--pubmed`: Exact source publication identity.
- `query`: Exact source mapped gene; legacy positional query aliases this field.
- `--study`: Exact source study accession.
- `--trait`: Exact source efo_trait label; never searches reported_trait prose.
- `--trait-id`: Exact source ontology identity.
- `--variant`: Exact source rsID.

## biomcp search protein

```text
Search five reviewed human UniProt headers by literal text or exact gene.
No variant effect, structure, interaction or clinical meaning is inferred. No bioinformatics tool runs.

Usage: biomcp search protein [OPTIONS] [query]

Arguments:
  [query]
          Literal text, exclusive with gene.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --gene <gene>
          Exact native gene field, without identifier inference.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --limit <limit>
          Five rows by default; no more than 25.

      --offset <offset>
          Source/retained result offset.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --pool <pool>
          Exact retained collection identity.

      --reviewed
          Reviewed human scope stays enabled; unreviewed search is deferred.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Search five reviewed human UniProt headers by exact text or exact gene.
No variant effect, structure, interaction or clinical meaning is inferred. Sources: uniprot.

- `--gene`: Exact source gene field, without identifier inference.
- `--limit`: Five rows by default; no more than 25.
- `--offset`: Source/retained result offset.
- `--pool`: Exact retained collection identity.
- `query`: Exact text, exclusive with gene.
- `--reviewed`: Reviewed human scope stays enabled; unreviewed search is deferred.

## biomcp search phenotype

```text
Search literal human HP term candidates through Monarch with bounded native pages. A term lookup answers terminology and does not establish a diagnosis.

Usage: biomcp search phenotype [OPTIONS] <text>

Arguments:
  <text>
          Unchanged literal term search text.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

  -l, --limit <limit>
          Maximum native headers, default five.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Native source row offset.

      --source <source>
          Monarch only; HPO search is unsupported.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Search exact human HP term candidates through Monarch with bounded source pages. Sources: hpo, monarch.

- `--limit`: Maximum source headers, default five.
- `--offset`: Source source row offset.
- `--source`: Monarch only; HPO search is unsupported.
- `text`: Unchanged exact term search text.

## biomcp search pathway

```text
Search Reactome (default) or WikiPathways human pathways. Membership names a source and release; it implies no therapy, response or actionability. No enrichment is calculated.

Usage: biomcp search pathway [OPTIONS] [query]

Arguments:
  [query]
          Literal source query; exclusive with gene and top-level discovery.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --fresh
          Request a new retained observation.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --gene <gene>
          Human gene input for exact resolution and UniProt mapping; Reactome only.

  -l, --limit <limit>
          Five rows by default, one to fifty. Reactome rows equals this limit without refill.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offset <offset>
          Native Reactome Start row or retained WikiPathways catalog offset.

      --pool <pool>
          Retained snapshot identity for deterministic continuation.

      --source <source>
          Sources in caller order; repeated sources take turns without title joins.

      --top-level
          Native human Reactome top-level discovery, exclusive with text and gene.

      --offline
          Read local data without source requests.

```

Search Reactome (default) or WikiPathways human pathways. Sources: reactome, wikipathways.

- `--fresh`: Request a new retained observation.
- `--gene`: Human gene input for exact resolution and UniProt mapping; Reactome only.
- `--limit`: Five rows by default, one to fifty. Reactome rows equals this limit without refill.
- `--offset`: Source Reactome Start row or retained WikiPathways catalog offset.
- `--pool`: Retained snapshot identity for deterministic continuation.
- `query`: Exact source query; exclusive with gene and top-level discovery.
- `--source`: Sources in caller order; repeated sources take turns without title joins.
- `--top-level`: Source human Reactome top-level discovery, exclusive with text and gene.

## biomcp search study

```text
Literal stored metadata search; text is never parsed as a query language. No frequency, denominator, survival or comparison is calculated. Source counts keep their native names.

Usage: biomcp search study [OPTIONS] [text]

Arguments:
  [text]
          Literal study ID, name or description text, without synonyms or typo repair.

Options:
      --cancer-type <cancer_type>
          Exact cBioPortal cancer-type code; no OncoTree conversion.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --limit <limit>
          Maximum headers per page.

      --offset <offset>
          Zero-based offset in the checked snapshot.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --pool <pool>
          Opaque continuation retaining the catalog revision and selectors.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Exact stored metadata search; text is never parsed as a query language. Sources: cbioportal.

- `--cancer-type`: Exact cBioPortal cancer-type code; no OncoTree conversion.
- `--limit`: Maximum headers per page.
- `--offset`: Zero-based offset in the checked snapshot.
- `--pool`: Opaque continuation retaining the catalog revision and selectors.
- `text`: Exact study ID, name or description text, without synonyms or typo repair.

## biomcp search pgx

```text
Checked gene-keyed CPIC pair search. No phenotype, phase or dose is calculated.

Usage: biomcp search pgx [OPTIONS] --gene <gene> --source <source>

Options:
      --cpic-level <cpic_level>
          Literal native level, including compound levels such as B/C.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --gene <gene>
          Explicit source gene symbol.

      --limit <limit>
          Rows per native page.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offset <offset>
          Native source offset.

      --source <source>
          Required CPIC source; ClinPGx is not supported for search.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Checked gene-keyed CPIC pair search. Sources: cpic.

- `--cpic-level`: Exact source level, including compound levels such as B/C.
- `--gene`: Explicit source gene symbol.
- `--limit`: Rows per source page.
- `--offset`: Source source offset.
- `--source`: Required CPIC source; ClinPGx is not supported for search.

## biomcp search diagnostic

```text
Search five source records by default. FDA device recalls cover every medical device; totals count recall records, without harm inference.

Usage: biomcp search diagnostic [OPTIONS] --source <source>

Options:
      --biomarker <biomarker>
          Exact table-0 biomarker cell; unsupported in the group table.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --bundle-revision <bundle_revision>
          Immutable stored bundle revision retained by a continuation.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --condition-id <condition_id>
          Exact MedGen:CUI identifier only.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --device-id <device_id>
          Exact PMA application, application/supplement or native K identity.

      --drug-application <drug_application>
          Cell-owned typed NDA/BLA application.

      --gene <gene>
          Exact source gene_symbol field, without alias expansion.

      --gene-id <gene_id>
          Exact source GeneID, exclusive with gene symbol.

      --initiated-from <initiated_from>
          Inclusive initiation date, YYYY-MM-DD; FDA device recalls only.

      --initiated-to <initiated_to>
          Inclusive initiation date, YYYY-MM-DD; FDA device recalls only.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

      --limit <limit>
          Five results by default; at most fifty source rows or product headers.

      --manufacturer <manufacturer>
          WHO manufacturer words or native FDA recalling_firm phrase.

      --marker <marker>
          Literal whole-word phrase in WHO's Pathogen/Disease/Marker cell.

      --offset <offset>
          Zero-based source result offset.

      --product-code <product_code>
          Exact native FDA device recall product code.

      --source <source>
          Explicit native GTR, WHO IVD, FDA companion or medical device recall discovery.

      --type <type>
          Exact native test_type value.

```

Search five source records by default. Sources: fda-cdx, fda-device-recall, gtr, who-ivd.

- `--biomarker`: Exact table-0 biomarker cell; unsupported in the group table.
- `--bundle-revision`: Immutable stored bundle revision retained by a continuation.
- `--condition-id`: Exact MedGen:CUI identifier only.
- `--device-id`: Exact PMA application, application/supplement or source K identity.
- `--drug-application`: Cell-owned typed NDA/BLA application.
- `--gene`: Exact source gene_symbol field, without alias expansion.
- `--gene-id`: Exact source GeneID, exclusive with gene symbol.
- `--initiated-from`: Inclusive initiation date, YYYY-MM-DD; FDA device recalls only.
- `--initiated-to`: Inclusive initiation date, YYYY-MM-DD; FDA device recalls only.
- `--limit`: Five results by default; at most fifty source rows or product headers.
- `--manufacturer`: WHO manufacturer words or source FDA recalling_firm phrase.
- `--marker`: Exact whole-word phrase in WHO's Pathogen/Disease/Marker cell.
- `--offset`: Zero-based source result offset.
- `--product-code`: Exact source FDA device recall product code.
- `--source`: Explicit source GTR, WHO IVD, FDA companion or medical device recall discovery.
- `--type`: Exact source test_type value.

## biomcp search adverse-event

```text
Search five report headers by default. MAUDE accepts device text or a manufacturer, product-code or problem filter. One report may list several devices; filters may match different devices. VAERS accepts MMR, COVID19, COVID-19 vaccine or one exact qualified native influenza code, matched without case; use influenza to list codes and labels. Tables remain separate and are never summed. One report may list several vaccines. VAERS returns five reported symptom rows by default; seriousness and age select independent tables. Counts are reports, not people. Reports do not show that a vaccine caused an event and cannot measure risk or compare vaccine safety.

Usage: biomcp search adverse-event [OPTIONS] [text] [section]

Arguments:
  [text]
          Literal drug text, MAUDE device phrase or exact supported VAERS vaccine.

  [section]
          Independent VAERS table; omitted means reported symptoms.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --limit <limit>
          Native report headers or VAERS rows per page; default five.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --manufacturer <manufacturer>
          MAUDE manufacturer phrase on device.manufacturer_d_name. A report may list several devices; filters may match different devices.

      --offset <offset>
          Native row offset. VAERS continuation requires its retained snapshot.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --problem <problem>
          MAUDE phrase on native product_problems.

      --product-code <product_code>...
          MAUDE exact native device.device_report_product_code; repeated codes join with OR.

      --reaction <reaction>
          FAERS MedDRA term phrase match, not exact equality.

      --received-from <received_from>
          Inclusive first FDA receipt lower bound, YYYY-MM-DD.

      --received-to <received_to>
          Inclusive first FDA receipt upper bound, YYYY-MM-DD.

      --offline
          Read local data without source requests.

      --serious <serious>
          FAERS report flag: any uses the source seriousness flag; the specific flags can overlap. Never a reaction outcome.

      --fresh
          Request fresh source data instead of cached replies.

      --snapshot <snapshot>
          Retained VAERS table for provider-free continuation.

      --source <source>
          FAERS by default; VAERS accepts only exact supported vaccine names.

```

Search five report headers by default. Sources: faers, maude, vaers.

- `--limit`: Source report headers or VAERS rows per page; default five.
- `--manufacturer`: MAUDE manufacturer phrase on device.manufacturer_d_name. A report may list several devices; filters may match different devices.
- `--offset`: Source row offset. VAERS continuation requires its retained snapshot.
- `--problem`: MAUDE phrase on source product_problems.
- `--product-code`: MAUDE exact source device.device_report_product_code; repeated codes join with OR.
- `--reaction`: FAERS MedDRA term phrase match, not exact equality.
- `--received-from`: Inclusive first FDA receipt lower bound, YYYY-MM-DD.
- `--received-to`: Inclusive first FDA receipt upper bound, YYYY-MM-DD.
- `section`: Independent VAERS table; omitted means reported symptoms.
- `--serious`: FAERS report flag: any uses the source seriousness flag; the specific flags can overlap. Never a reaction outcome.
- `--snapshot`: Retained VAERS table for source-free continuation.
- `--source`: FAERS by default; VAERS accepts only exact supported vaccine names.
- `text`: Exact drug text, MAUDE device phrase or exact supported VAERS vaccine.

Independent VAERS table; omitted means reported symptoms.

- `symptoms`: Source reported symptom rows.
- `seriousness`: Reports classified by source seriousness.
- `age`: Source age groups.

## biomcp search disease

```text
Literal discovery text or an explicit SEER catalog page.

Usage: biomcp search disease [OPTIONS] [text]

Arguments:
  [text]
          Literal source search text; absent for the SEER site catalog.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --limit <limit>
          Maximum source rows on one page.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Native row offset.

      --ontology <ontology>
          Exactly one ontology, default MONDO.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --source <source>
          One explicit source, default OLS.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Exact discovery text or an explicit SEER catalog page. Sources: monarch, mydisease, ols, seer.

- `--limit`: Maximum source rows on one page.
- `--offset`: Source row offset.
- `--ontology`: Exactly one ontology, default MONDO.
- `--source`: One explicit source, default OLS.
- `text`: Exact source search text; absent for the SEER site catalog.

## biomcp search drug

```text
Search one drug source. FDA drug recalls accept a name or native firm, classification, status, reason and initiation dates; counts are recall records. Other sources require a name.

Usage: biomcp search drug [OPTIONS] [query]

Arguments:
  [query]
          Literal name phrase; optional for FDA drug recalls. Drugs@FDA searches brands and ingredients,
          exact names before word matches. Harmonized recall names match but never become display names.

Options:
      --classification <classification>
          Native FDA classification; drug recalls only.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --firm <firm>
          Literal published recalling firm phrase; drug recalls only.

      --initiated-from <initiated_from>
          Inclusive initiation day, YYYY-MM-DD; drug recalls only.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --initiated-to <initiated_to>
          Inclusive initiation day, YYYY-MM-DD; drug recalls only.

      --limit <limit>
          Native candidate rows per page.

      --name-selector <name_selector>
          Explicit ChEMBL native name selector; omitted means preferred name.

      --offset <offset>
          Native rows to skip.

      --reason <reason>
          Phrase in the published recall reason; drug recalls only.

      --offline
          Read local data without source requests.

      --source <source>
          Exactly one source; default MyChem never fans out.

      --fresh
          Request fresh source data instead of cached replies.

      --status <status>
          Literal native recall status; drug recalls only.

```

Search one drug source. Sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq.

- `--classification`: Source FDA classification; drug recalls only.
- `--firm`: Exact published recalling firm phrase; drug recalls only.
- `--initiated-from`: Inclusive initiation day, YYYY-MM-DD; drug recalls only.
- `--initiated-to`: Inclusive initiation day, YYYY-MM-DD; drug recalls only.
- `--limit`: Source candidate rows per page.
- `--name-selector`: Explicit ChEMBL source name selector; omitted means preferred name.
- `--offset`: Source rows to skip.
- `query`: Exact name phrase; optional for FDA drug recalls. Drugs@FDA searches brands and ingredients,
exact names before word matches. Harmonized recall names match but never become display names.
- `--reason`: Phrase in the published recall reason; drug recalls only.
- `--source`: Exactly one source; default MyChem never fans out.
- `--status`: Exact source recall status; drug recalls only.

## biomcp search gene

```text
Search genes by text, cytoband, or their intersection.

Usage: biomcp search gene [OPTIONS] [query]

Arguments:
  [query]
          Words matched against symbols, previous symbols, aliases and names.

Options:
      --band <band>
          A cytoband prefix.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --limit <limit>
          Number of results per page.

      --locus-group <locus_group>
          Optional HGNC locus group intersected with the selector.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offset <offset>
          Number of results to skip.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Search genes by text, cytoband, or their intersection. Sources: hgnc.

- `--band`: A cytoband prefix.
- `--limit`: Number of results per page.
- `--locus-group`: Optional HGNC locus group intersected with the selector.
- `--offset`: Number of results to skip.
- `query`: Words matched against symbols, previous symbols, aliases and names.

## biomcp search article

```text
Checked search selectors and pool cursor.

Usage: biomcp search article [OPTIONS] [text]

Arguments:
  [text]
          Words sent to each named source in its default fields.

Options:
      --author <author>
          Author words sent to PubMed's author field.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --disease <disease>
          Disease words.

      --drug <drug>
          Drug words.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --gene <gene>
          Gene input; PubMed resolves it through HGNC.

      --journal <journal>
          Journal words sent to PubMed's journal field.

      --limit <limit>
          Maximum page length.

      --no-preprints
          Exclude preprints using the source's native filter.

      --offset <offset>
          Number of pool items to skip.

      --offline
          Read local data without source requests.

      --open-access
          Require the source's native open-access filter; Semantic Scholar means a public PDF.

      --fresh
          Request fresh source data instead of cached replies.

      --pool <pool>
          Stable snapshot from an earlier response.

      --exclude-retracted


      --include-retracted


      --since <since>
          Inclusive lower publication-date bound at the stated precision; PubMed matches either electronic or print date.

      --sort <sort>
          rank (default): native provider order; several sources take turns; native search cost.
          date: native search cost; PubMed means date added and Europe PMC means newest first; other sources refuse it.
          citations: most cited first across the whole pool; adds iCite calls, about 1 s per 1,000 PMIDs.
          All sources support citations.

      --source <source>
          Sources to search in caller order, comma separated. Default: pubmed.
          Several sources take turns, remove duplicates and show every native rank.
          pubmed: keywords with PubMed MeSH mapping; dropped words appear as notices.
          europepmc: words in open full text and preprints; incidental mentions are possible.
          litvar: exact variant publications; requires --variant and a protein change needs --gene; unresolved inputs return candidates.
          semantic-scholar: broad discovery; a shared keyless pool may return 429; an optional free key raises the limit.
          litsense: sentence resemblance, not a match; unrelated hits and timeouts are possible.
          pubtator: entity and word search including full text; needs the right entity ID; --gene uses HGNC Entrez identity and cannot run without that join.

      --type <type>
          Publication-type filter.

      --until <until>
          Inclusive upper publication-date bound at the stated precision; PubMed matches either electronic or print date.

      --variant <variant>
          Variant words.

```

Checked search selectors and pool cursor. Sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar.

- `--author`: Author words sent to PubMed's author field.
- `--disease`: Disease words.
- `--drug`: Drug words.
- `--gene`: Gene input; PubMed resolves it through HGNC.
- `--journal`: Journal words sent to PubMed's journal field.
- `--limit`: Maximum page length.
- `--no-preprints`: Exclude preprints using the source's source filter.
- `--offset`: Number of pool items to skip.
- `--open-access`: Require the source's source open-access filter; Semantic Scholar means a public PDF.
- `--pool`: Stable snapshot from an earlier response.
- `--exclude-retracted`: Include retracted papers by default, or exclude them using a source filter.
- `--include-retracted`: Include retracted papers by default, or exclude them using a source filter.
- `--since`: Inclusive lower publication-date bound at the stated precision; PubMed matches either electronic or print date.
- `--sort`: rank (default): source order; several sources take turns; source search cost.
date: source search cost; PubMed means date added and Europe PMC means newest first; other sources refuse it.
citations: most cited first across the whole pool; adds iCite calls, about 1 s per 1,000 PMIDs.
All sources support citations.
- `--source`: Sources to search in caller order, comma separated. Default: pubmed.
Several sources take turns, remove duplicates and show every source rank.
pubmed: keywords with PubMed MeSH mapping; dropped words appear as notices.
europepmc: words in open full text and preprints; incidental mentions are possible.
litvar: exact variant publications; requires --variant and a protein change needs --gene; unresolved inputs return candidates.
semantic-scholar: broad discovery; a shared keyless pool may return 429; an optional free key raises the limit.
litsense: sentence resemblance, not a match; unrelated hits and timeouts are possible.
pubtator: entity and word search including full text; needs the right entity ID; --gene uses HGNC Entrez identity and cannot run without that join.
- `text`: Words sent to each named source in its default fields.
- `--type`: Publication-type filter.
- `--until`: Inclusive upper publication-date bound at the stated precision; PubMed matches either electronic or print date.
- `--variant`: Variant words.

## biomcp search trial

```text
Search ClinicalTrials.gov or keyed NCI with native counts. NCI --facility matches exact organization names; a listed site is not necessarily enrolling.

Usage: biomcp search trial [OPTIONS] [query]

Arguments:
  [query]
          General registry discovery text.

Options:
      --age <age>
          Age in years; unstated source bounds count as no limit.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --biomarker-code <biomarker_code>
          Exact NCIt biomarker concept listed by NCI in trial criteria; NCI source only. This listing does not establish biomarker positivity or patient eligibility.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --biomarker-use <biomarker_use>
          Filter by NCI's native inclusion or exclusion criterion. Check each result's biomarker entry and reported problems.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --condition <condition>...
          Condition alternatives joined with OR; NCI requires NCIt codes.

      --config <config>
          Read configuration from this file.

      --condition-id <condition_id>
          Exact MONDO or DOID CURIE resolved by MyDisease; NCI also accepts explicit NCIT:C codes.

      --count-only
          Source total only; no returned study rows.

      --country <country>...
          Exact ClinicalTrials.gov country label; repeat for OR. Trials must list a site in a named country. NCI uses its native country filter.

      --criteria <criteria>
          Search literal registry eligibility text for discovery only; no eligibility judgment.

      --distance <distance>
          Source distance with explicit mi or km units.

      --offline
          Read local data without source requests.

      --exact-condition
          Ask ClinicalTrials.gov to restrict matching to the registered condition phrase; its support is partial.

      --fresh
          Request fresh source data instead of cached replies.

      --facility <facility>
          Facility phrase; ClinicalTrials.gov broadens after zero exact matches. NCI matches exact names. Cannot combine with radius.

      --gene <gene>
          Resolve one checked gene for registry text discovery only; no eligibility judgment.

      --has-results
          Require posted source results.

      --id <id>
          Exact NCT identity in a conjunction with other filters.

      --intervention <intervention>...
          Intervention alternatives joined with OR; supported by ClinicalTrials.gov.

      --keyword <keyword>...
          Repeat for ClinicalTrials.gov general text alternatives joined with OR; different filters combine with AND. NCI accepts one keyword.

      --lat <lat>
          Radius center latitude in degrees.

      --limit <limit>
          Headers per page; NCI applies its own lower publisher limit.

      --lon <lon>
          Radius center longitude in degrees.

      --next-page <next_page>
          Opaque registry continuation; keep filters unchanged.

      --offset <offset>
          Zero-based offset of the first row in this continuation page.

      --phase <phase>
          Exact source phase. ClinicalTrials.gov: 1, 2, 3, 4, early-1, not-applicable, 1/2, 2/3. NCI: NA, 0, I, I_II, II, II_III, III, IV.

      --sex <sex>
          Registry sex facet.

      --source <source>
          Explicit provider; defaults to ClinicalTrials.gov and never falls back.

      --sponsor <sponsor>
          Sponsor name; ClinicalTrials.gov broadens after zero exact matches. Use --sponsor-type for the funding category.

      --sponsor-type <sponsor_type>
          Registry sponsor funding category; use --sponsor for a sponsor name.

      --status <status>
          Exact source status. NCI: Active, Administratively Complete, Approved, Closed to Accrual, Closed to Accrual and Intervention, Complete, INACTIVE, Enrolling by Invitation, Temporarily Closed to Accrual, Temporarily Closed to Accrual and Intervention, Withdrawn. ClinicalTrials.gov uses registry labels.

      --study-type <study_type>
          Exact registry study type.

      --updated-since <updated_since>
          First included registry update date.

      --updated-until <updated_until>
          Last included registry update date.

      --variant <variant>
          Literal variant text for registry discovery only; no eligibility judgment.

```

Search ClinicalTrials.gov or keyed NCI with source counts. Sources: ctgov, nci.

- `--age`: Age in years; unstated source bounds count as no limit.
- `--biomarker-code`: Exact NCIt biomarker concept listed by NCI in trial criteria; NCI source only. This listing does not establish biomarker positivity or patient eligibility.
- `--biomarker-use`: Filter by NCI's source inclusion or exclusion criterion. Check each result's biomarker entry and reported problems.
- `--condition`: Condition alternatives joined with OR; NCI requires NCIt codes.
- `--condition-id`: Exact MONDO or DOID CURIE resolved by MyDisease; NCI also accepts explicit NCIT:C codes.
- `--count-only`: Source total only; no returned study rows.
- `--country`: Exact ClinicalTrials.gov country label; repeat for OR. Trials must list a site in a named country. NCI uses its source country filter.
- `--criteria`: Search exact registry eligibility text for discovery only; no eligibility judgment.
- `--distance`: Source distance with explicit mi or km units.
- `--exact-condition`: Ask ClinicalTrials.gov to restrict matching to the registered condition phrase; its support is partial.
- `--facility`: Facility phrase; ClinicalTrials.gov broadens after zero exact matches. NCI matches exact names. Cannot combine with radius.
- `--gene`: Resolve one checked gene for registry text discovery only; no eligibility judgment.
- `--has-results`: Require posted source results.
- `--id`: Exact NCT identity in a conjunction with other filters.
- `--intervention`: Intervention alternatives joined with OR; supported by ClinicalTrials.gov.
- `--keyword`: Repeat for ClinicalTrials.gov general text alternatives joined with OR; different filters combine with AND. NCI accepts one keyword.
- `--lat`: Radius center latitude in degrees.
- `--limit`: Headers per page; NCI applies its own lower publisher limit.
- `--lon`: Radius center longitude in degrees.
- `--next-page`: Opaque registry continuation; keep filters unchanged.
- `--offset`: Zero-based offset of the first row in this continuation page.
- `--phase`: Exact source phase. ClinicalTrials.gov: 1, 2, 3, 4, early-1, not-applicable, 1/2, 2/3. NCI: NA, 0, I, I_II, II, II_III, III, IV.
- `query`: General registry discovery text.
- `--sex`: Registry sex facet.
- `--source`: Explicit source; defaults to ClinicalTrials.gov and never falls back.
- `--sponsor`: Sponsor name; ClinicalTrials.gov broadens after zero exact matches. Use --sponsor-type for the funding category.
- `--sponsor-type`: Registry sponsor funding category; use --sponsor for a sponsor name.
- `--status`: Exact source status. NCI: Active, Administratively Complete, Approved, Closed to Accrual, Closed to Accrual and Intervention, Complete, INACTIVE, Enrolling by Invitation, Temporarily Closed to Accrual, Temporarily Closed to Accrual and Intervention, Withdrawn. ClinicalTrials.gov uses registry labels.
- `--study-type`: Exact registry study type.
- `--updated-since`: First included registry update date.
- `--updated-until`: Last included registry update date.
- `--variant`: Exact variant text for registry discovery only; no eligibility judgment.

## biomcp search variant

```text
Discover source candidates using one declared selector, without interpreting free text.

Usage: biomcp search variant [OPTIONS]

Options:
      --build <build>
          Optional caller assembly sent to MyVariant; regions require it explicitly.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --coding <coding>
          Coding HGVS change searched literally in ClinVar with --gene, without choosing a transcript.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --condition <condition>
          Literal MyVariant clinvar.rcv.conditions.name phrase; source text is not a local diagnosis. Requires a declared protein or region selector.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --gene <gene>
          Checked gene symbol for the dbNSFP substitution or ClinVar coding/synonymous selector.

      --has <has>
          Require the selected component's identifier field. Only source-proven mappings are offered.

      --limit <limit>
          Native candidates per page, default 10, maximum 50; all candidates remain candidates.

      --offset <offset>
          Native offset within this command's bounded result window; larger native totals remain visible.

      --protein <protein>
          Explicit substitution or synonymous change using checked amino-acid codes.

      --offline
          Read local data without source requests.

      --query <query>
          Raw MyVariant query, passed and echoed unchanged when component policy allows; requires --source myvariant. With disabled components use declared selectors instead.

      --fresh
          Request fresh source data instead of cached replies.

      --region <region>
          Exact one-based inclusive MyVariant region, such as chr7:55259500-55259600; requires --build.

      --review-status <review_status>
          Literal MyVariant clinvar.rcv.review_status phrase; no star conversion. Requires a declared protein or region selector.

      --source <source>
          Explicit source language; required for a raw query.

```

Discover source candidates using one declared selector, without interpreting free text. Sources: myvariant.

- `--build`: Optional caller assembly sent to MyVariant; regions require it explicitly.
- `--coding`: Coding HGVS change searched literally in ClinVar with --gene, without choosing a transcript.
- `--condition`: Exact MyVariant clinvar.rcv.conditions.name phrase; source text is not a local diagnosis. Requires a declared protein or region selector.
- `--gene`: Checked gene symbol for the dbNSFP substitution or ClinVar coding/synonymous selector.
- `--has`: Require the selected component's identifier field. Only source-proven mappings are offered.
- `--limit`: Source candidates per page, default 10, maximum 50; all candidates remain candidates.
- `--offset`: Source offset within this command's bounded result window; larger source totals remain visible.
- `--protein`: Explicit substitution or synonymous change using checked amino-acid codes.
- `--query`: Raw MyVariant query, passed and echoed unchanged when component policy allows; requires --source myvariant. With disabled components use declared selectors instead.
- `--region`: Exact one-based inclusive MyVariant region, such as chr7:55259500-55259600; requires --build.
- `--review-status`: Exact MyVariant clinvar.rcv.review_status phrase; no star conversion. Requires a declared protein or region selector.
- `--source`: Explicit source language; required for a raw query.

## biomcp get

```text
Retrieve one entity and its source sections.

Usage: biomcp get [OPTIONS] <COMMAND>

Commands:
  biomarker      Search NCI by exact NCIt code. Several native rows return candidates without choosing one. The public tier returns codes, name, type, trial count, synonyms, parent_ids, ancestor_ids and eligibility_criterion. No clinical interpretation is performed.
  author         Exact qualified records in caller order, with no name reconciliation.
  cell-line      One exact identity and an optional bounded named section.
  gwas           Source association or study and native context. Reports no personal risk, causal variant or gene, or clinical interpretation of a statistic.
  protein        Read source protein cards; a symbol returns candidates without choosing a protein.
                 Sections disclose native facts with independent row bounds; function keeps all paragraphs.
                 No variant effect, structure, interaction or clinical meaning is inferred. No bioinformatics tool runs.
  phenotype      Read an exact HP term from HPO or Monarch, or page native source sections. A term lookup answers terminology; an association answers what its publisher asserts. Neither establishes a diagnosis.
  pathway        Get an exact human Reactome R-HSA or WikiPathways WP card. Read whole summary, genes, Reactome events or bounded all sections. Membership names a source and release; it implies no therapy, response or actionability. No enrichment is calculated.
  study          Exact case-sensitive study card or bounded metadata section. No frequency, denominator, survival or comparison is calculated. Source counts keep their native names.
  pgx            PGx card or section. No phenotype, phase or dose is calculated.
  diagnostic     Get one exact source record or supported section. A listing is not authorization; a companion row is not a treatment recommendation.
  adverse-event  Exact native report identity and an optional independently paged section.
  disease        Exact concept or native SEER site with independently paged section.
  drug           Exact native ID retrieval with an explicitly selected source.
  gene           Get one to twenty human gene inputs in their original order.
  article        Get one to twenty article inputs in caller order.
  trial          Retrieve exact trial cards in input order; request bounded intact source sections with independent continuations.
  variant        Read an rsID, CAid, clinvar:N, VCV accession, reference-qualified HGVS or single-base genomic coordinates such as 7-140753336-A-T through the shared resolver. MyVariant identity lookups default to GRCh38; --build overrides it. Chromosome genomic HGVS requires --build. Use search variant --gene BRAF --protein V600E or search variant --gene PTEN --coding c.333G>T for gene shorthand. Every candidate remains available; source assertions do not classify the variant for the caller.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Retrieve one entity and its source sections.


## biomcp get biomarker

```text
Search NCI by exact NCIt code. Several native rows return candidates without choosing one. The public tier returns codes, name, type, trial count, synonyms, parent_ids, ancestor_ids and eligibility_criterion. No clinical interpretation is performed.

Usage: biomcp get biomarker [OPTIONS] <id>

Arguments:
  <id>
          Exact NCIt code, such as C96022.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --source <source>
          NCI is the biomarker concept source.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Search NCI by exact NCIt code. Sources: nci.

- `id`: Exact NCIt code, such as C96022.
- `--source`: NCI is the biomarker concept source.

## biomcp get author

```text
Exact qualified records in caller order, with no name reconciliation.

Usage: biomcp get author [OPTIONS] <ids>...

Arguments:
  <ids>...
          One or more exact qualified IDs.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Exact qualified records in caller order, with no name reconciliation. Sources: orcid, semantic-scholar.

- `ids`: One or more exact qualified IDs.

## biomcp get cell-line

```text
One exact identity and an optional bounded named section.

Usage: biomcp get cell-line [OPTIONS] <id> [section]

Arguments:
  <id>
          One exact primary accession or qualified source ID.

  [section]
          Omit for the default card; all returns bounded first pages without an assay count.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --section-limit <section_limit>
          Explicit row limit for each requested collection; native variation defaults to three.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --section-offset <section_offset>
          Offset within each independently requested collection.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

One exact identity and an optional bounded named section. Sources: cellosaurus.

- `id`: One exact primary accession or qualified source ID.
- `section`: Omit for the default card; all returns bounded first pages without an assay count.
- `--section-limit`: Explicit row limit for each requested collection; source variation defaults to three.
- `--section-offset`: Offset within each independently requested collection.

Omit for the default card; all returns bounded first pages without an assay count.

- `names`: Source display names.
- `identity`: Independent species, diseases and qualified warnings.
- `xrefs`: Complete source cross-reference rows.
- `variants`: Complete source variation rows.
- `chembl`: Exact accession-linked ChEMBL cell metadata.
- `all`: Bounded first page of every named section.

## biomcp get gwas

```text
Source association or study and native context. Reports no personal risk, causal variant or gene, or clinical interpretation of a statistic.

Usage: biomcp get gwas [OPTIONS] <id> [section]

Arguments:
  <id>
          Qualified association ID or exact GCST study accession.

  [section]
          Association loci or study associations and ancestries.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --section-limit <section_limit>
          Maximum native relation rows.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --section-offset <section_offset>
          Native relation row offset.

      --snapshot <snapshot>
          Retained ancestry reply for a local page.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Source association or study and source context. Sources: gwas.

- `id`: Qualified association ID or exact GCST study accession.
- `section`: Association loci or study associations and ancestries.
- `--section-limit`: Maximum source relation rows.
- `--section-offset`: Source relation row offset.
- `--snapshot`: Retained ancestry reply for a local page.

Association loci or study associations and ancestries.

- `loci`: Source association loci.
- `associations`: Source study association pages.
- `ancestries`: Source study ancestry rows.

## biomcp get protein

```text
Read source protein cards; a symbol returns candidates without choosing a protein.
Sections disclose native facts with independent row bounds; function keeps all paragraphs.
No variant effect, structure, interaction or clinical meaning is inferred. No bioinformatics tool runs.

Usage: biomcp get protein [OPTIONS] <input>...

Arguments:
  <input>...
          Source accessions or gene symbols, preserving caller order.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --section <section>
          Select native sections or all bounded first pages; function keeps one whole text record.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --section-limit <section_limit>
          First-page row limit; function keeps one whole native text record and does not page its paragraphs.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read source protein cards; a symbol returns candidates without choosing a protein.
Sections disclose source facts with independent row bounds; function keeps all paragraphs.
No variant effect, structure, interaction or clinical meaning is inferred. Sources: complex-portal, hgnc, interpro, uniprot.

- `input`: Source accessions or gene symbols, preserving caller order.
- `--section`: Select source sections or all bounded first pages; function keeps one whole text record.
- `--section-limit`: First-page row limit; function keeps one whole source text record and does not page its paragraphs.

Select source sections or all bounded first pages; function keeps one whole text record.

- `features`: Source UniProt features.
- `domains`: Source InterPro entry matches.
- `isoforms`: Source alternative products.
- `complexes`: Exact participant-admitted complex headers.
- `structures`: Source structure cross-reference links.
- `function`: One whole source function text record with all source paragraphs.
- `sequence`: Direct exact-identity sequence and audit metadata.
- `all`: Bounded first page of every named section.

## biomcp get phenotype

```text
Read an exact HP term from HPO or Monarch, or page native source sections. A term lookup answers terminology; an association answers what its publisher asserts. Neither establishes a diagnosis.

Usage: biomcp get phenotype [OPTIONS] <id> [section]

Arguments:
  <id>
          Exact HP identifier.

  [section]
          Absent means the source identity card.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --section-limit <section_limit>
          Default five text/link rows or one complete association row.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --section-offset <section_offset>
          Native section row offset.

      --snapshot <snapshot>
          Retained exact source reply for local text and link pages; unsupported for diseases and genes.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --source <source>
          HPO or Monarch for cards, synonyms and xrefs; Monarch only for hierarchy, diseases and genes. Defaults to HPO for cards, synonyms and xrefs, otherwise Monarch.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read an exact HP term from HPO or Monarch, or page source sections. Sources: hpo, monarch.

- `id`: Exact HP identifier.
- `section`: Absent means the source identity card.
- `--section-limit`: Default five text/link rows or one complete association row.
- `--section-offset`: Source section row offset.
- `--snapshot`: Retained exact source reply for local text and link pages; unsupported for diseases and genes.
- `--source`: HPO or Monarch for cards, synonyms and xrefs; Monarch only for hierarchy, diseases and genes. Defaults to HPO for cards, synonyms and xrefs, otherwise Monarch.

Absent means the source identity card.

- `synonyms`: Source synonyms.
- `xrefs`: Source cross references.
- `hierarchy`: Source direct parent and child links.
- `diseases`: Source disease association rows.
- `genes`: Source human gene association rows.

## biomcp get pathway

```text
Get an exact human Reactome R-HSA or WikiPathways WP card. Read whole summary, genes, Reactome events or bounded all sections. Membership names a source and release; it implies no therapy, response or actionability. No enrichment is calculated.

Usage: biomcp get pathway [OPTIONS] <id> [section]

Arguments:
  <id>
          Exact human Reactome or WikiPathways identity without version suffixes.

  [section]
          Omit for the card; WikiPathways has summary and genes sections.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --fresh
          Request a new source observation.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --pool <pool>
          Retained snapshot, when continuing a source relation.

      --section-limit <section_limit>
          Bounded initial section size; each all section has its own first page.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --section-offset <section_offset>
          Native relation offset for a selected genes or events section.

      --offline
          Read local data without source requests.

```

Get an exact human Reactome R-HSA or WikiPathways WP card. Sources: reactome, wikipathways.

- `--fresh`: Request a new source observation.
- `id`: Exact human Reactome or WikiPathways identity without version suffixes.
- `--pool`: Retained snapshot, when continuing a source relation.
- `section`: Omit for the card; WikiPathways has summary and genes sections.
- `--section-limit`: Bounded initial section size; each all section has its own first page.
- `--section-offset`: Source relation offset for a selected genes or events section.

Omit for the card; WikiPathways has summary and genes sections.

- `summary`: Complete source description.
- `genes`: Namespaced source participants with exact resolved gene joins.
- `events`: Source Reactome event objects and unresolved database references.
- `all`: Each source-supported section's bounded first page.

## biomcp get study

```text
Exact case-sensitive study card or bounded metadata section. No frequency, denominator, survival or comparison is calculated. Source counts keep their native names.

Usage: biomcp get study [OPTIONS] <id> [section]

Arguments:
  <id>
          Exact source study ID; no default or repair.

  [section]
          Omit for the card; all returns bounded first pages only.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --section-limit <section_limit>
          Maximum records in each selected section.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --section-offset <section_offset>
          Offset within one named section; all requires zero.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Exact case-sensitive study card or bounded metadata section. Sources: cbioportal.

- `id`: Exact source study ID; no default or repair.
- `section`: Omit for the card; all returns bounded first pages only.
- `--section-limit`: Maximum records in each selected section.
- `--section-offset`: Offset within one named section; all requires zero.

Omit for the card; all returns bounded first pages only.

- `profiles`: Five exact source molecular profile references.
- `sample-lists`: Five sample-list metadata records, without member sample IDs.
- `files`: Five repository file references; no automatic download.
- `all`: Each named section's bounded first page; never a gene aggregate request.

## biomcp get pgx

```text
PGx card or section. No phenotype, phase or dose is calculated.

Usage: biomcp get pgx [OPTIONS] --source <source> [section]

Arguments:
  [section]
          One named section; omitted means the CPIC gene card.

Options:
      --annotation-id <annotation_id>
          Checked ClinPGx accession, never a shared PaId.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --diplotype <diplotype>
          Unchanged declared expression required for the diplotype section only.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --drug-id <drug_id>
          Exact published CPIC drug key; recommendations only.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --gene <gene>
          Explicit source gene symbol.

      --kind <kind>
          One ClinPGx collection; clinical is the default only on annotations.

      --section-limit <section_limit>
          Rows per page: three clinical, five elsewhere.

      --section-offset <section_offset>
          Offset within the selected section.

      --snapshot <snapshot>
          Retained collection for local continuation.

      --offline
          Read local data without source requests.

      --source <source>
          Required source, checked against the section.

      --fresh
          Request fresh source data instead of cached replies.

```

PGx card or section. Sources: clinpgx, cpic.

- `--annotation-id`: Checked ClinPGx accession, never a shared PaId.
- `--diplotype`: Unchanged declared expression required for the diplotype section only.
- `--drug-id`: Exact published CPIC drug key; recommendations only.
- `--gene`: Explicit source gene symbol.
- `--kind`: One ClinPGx collection; clinical is the default only on annotations.
- `section`: One named section; omitted means the CPIC gene card.
- `--section-limit`: Rows per page: three clinical, five elsewhere.
- `--section-offset`: Offset within the selected section.
- `--snapshot`: Retained collection for local continuation.
- `--source`: Required source, checked against the section.

One named section; omitted means the CPIC gene card.

- `recommendations`: Complete source recommendation contexts.
- `guidelines`: Source guideline references.
- `diplotype`: Declared source diplotype lookup.
- `annotations`: One ClinPGx annotation kind.

## biomcp get diagnostic

```text
Get one exact source record or supported section. A listing is not authorization; a companion row is not a treatment recommendation.

Usage: biomcp get diagnostic [OPTIONS] --source <source> <id> [section]

Arguments:
  <id>
          Exact native identity, including version for GTR.

  [section]
          GTR: genes, conditions, methods. PMA: regulatory, supplements.
          WHO, 510(k) and device recalls have no sections; omit for the card.

Options:
      --bundle-revision <bundle_revision>
          Immutable stored bundle revision retained by a continuation.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --section-limit <section_limit>
          Maximum native rows; omitted means five rows, or one regulatory fragment.

      --section-offset <section_offset>
          Zero-based native row or fragment offset.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --source <source>
          Explicit source: GTR accession/version, WHO Product ID, PMA P number,
          510(k) K number or device recall Z-NNNN-YYYY.

      --supplement <supplement>
          Exact native PMA supplement; absent selects the base row.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Get one exact source record or supported section. Sources: fda-510k, fda-device-recall, fda-pma, gtr, who-ivd.

- `--bundle-revision`: Immutable stored bundle revision retained by a continuation.
- `id`: Exact source identity, including version for GTR.
- `section`: GTR: genes, conditions, methods. PMA: regulatory, supplements.
WHO, 510(k) and device recalls have no sections; omit for the card.
- `--section-limit`: Maximum source rows; omitted means five rows, or one regulatory fragment.
- `--section-offset`: Zero-based source row or fragment offset.
- `--source`: Explicit source: GTR accession/version, WHO Product ID, PMA P number,
510(k) K number or device recall Z-NNNN-YYYY.
- `--supplement`: Exact source PMA supplement; absent selects the base row.

GTR: genes, conditions, methods. PMA: regulatory, supplements.
WHO, 510(k) and device recalls have no sections; omit for the card.

- `genes`: GTR source gene rows; five per page.
- `conditions`: GTR source condition rows; five per page.
- `methods`: GTR source method entries; five per page.
- `regulatory`: Selected PMA submission statement; one fragment per page.
- `supplements`: PMA base application supplement headers; five per page.

## biomcp get adverse-event

```text
Exact native report identity and an optional independently paged section.

Usage: biomcp get adverse-event [OPTIONS] <id> [section]

Arguments:
  <id>
          Published report ID with no source prefix.

  [section]
          FAERS owns drugs, reactions and identifiers; MAUDE owns devices and narrative. Omitted means card only.

Options:
      --drug-index <drug_index>
          Absolute source-array index; required only for identifier sections.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --section-limit <section_limit>
          Whole source rows per section page.

      --section-offset <section_offset>
          Offset within the selected native section.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --snapshot <snapshot>
          Checked retained report snapshot for provider-free continuation.

      --source <source>
          FAERS by default; MAUDE is explicit.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Exact source report identity and an optional independently paged section. Sources: faers, maude.

- `--drug-index`: Absolute source-array index; required only for identifier sections.
- `id`: Published report ID with no source prefix.
- `section`: FAERS owns drugs, reactions and identifiers; MAUDE owns devices and narrative. Omitted means card only.
- `--section-limit`: Whole source rows per section page.
- `--section-offset`: Offset within the selected source section.
- `--snapshot`: Checked retained report snapshot for source-free continuation.
- `--source`: FAERS by default; MAUDE is explicit.

FAERS owns drugs, reactions and identifiers; MAUDE owns devices and narrative. Omitted means card only.

- `drugs`: Independent source drug rows.
- `reactions`: Independent source reaction rows.
- `unii`: Source UNII strings.
- `rxcui`: Source RxCUI strings.
- `product-ndc`: Source product NDC strings.
- `package-ndc`: Source package NDC strings.
- `spl-id`: Source SPL IDs.
- `spl-set-id`: Source SPL set IDs.
- `application-number`: Source application numbers.
- `devices`: Source device rows.
- `narrative`: Complete source narrative rows.

## biomcp get disease

```text
Exact concept or native SEER site with independently paged section.

Usage: biomcp get disease [OPTIONS] <id> [section]

Arguments:
  <id>
          Exact ontology CURIE or an explicit SEER catalog site ID.

  [section]
          One named section; absent means identity card only.

Options:
      --age-range <age_range>
          Native age-range code.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --dataset <dataset>
          Requested native dataset selector, not proof of returned membership.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --interval <interval>
          Survival-only interval code.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --ontology <ontology>
          One OLS ontology.

      --period <period>
          Incidence-only native period code.

      --race <race>
          Native race code.

      --section-limit <section_limit>
          Native rows per page: default 2 for Monarch genes, 1 for Monarch phenotypes, otherwise 5.

      --section-offset <section_offset>
          Native row offset within the selected section.

      --offline
          Read local data without source requests.

      --sex <sex>
          Native sex code.

      --fresh
          Request fresh source data instead of cached replies.

      --source <source>
          Default MyDisease; no automatic fallback.

```

Exact concept or source SEER site with independently paged section. Sources: monarch, mydisease, ols, seer.

- `--age-range`: Source age-range code.
- `--dataset`: Requested source dataset selector, not proof of returned membership.
- `id`: Exact ontology CURIE or an explicit SEER catalog site ID.
- `--interval`: Survival-only interval code.
- `--ontology`: One OLS ontology.
- `--period`: Incidence-only source period code.
- `--race`: Source race code.
- `section`: One named section; absent means identity card only.
- `--section-limit`: Source rows per page: default 2 for Monarch genes, 1 for Monarch phenotypes, otherwise 5.
- `--section-offset`: Source row offset within the selected section.
- `--sex`: Source sex code.
- `--source`: Default MyDisease; no automatic fallback.

One named section; absent means identity card only.

- `synonyms`: Publisher scoped synonyms.
- `xrefs`: Source cross references.
- `hierarchy`: Source parent relations.
- `genes`: Source gene association rows.
- `phenotypes`: Source phenotype association rows.
- `incidence`: Source stage incidence rows.
- `survival`: Annual survival rows.

## biomcp get drug

```text
Exact native ID retrieval with an explicitly selected source.

Usage: biomcp get drug [OPTIONS] --source <source> <input> [section]...

Arguments:
  <input>
          Exact source-native ID; WHO references retain native spaces. No name or alias resolution.

  [section]...
          Explicit named disclosures; empty means identity only.

Options:
      --civic-cursor <civic_cursor>
          Native continuation cursor for that exact drug, profile and claim collection.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --civic-kind <civic_kind>
          Independent native evidence or assertion collection within the selected profile.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --civic-profile <civic_profile>
          Selected native CIViC molecular profile for a drug-scoped claim stream.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --label-section <label_section>
          Exact native label heading, required only for the label section.

      --product-number <product_number>
          Explicit native application product number, only for approvals.

      --section-limit <section_limit>
          Named section page size, refused on an identity-only card.

      --section-offset <section_offset>
          Named section continuation, refused on an identity-only card.

      --source <source>
          Required source; there is no default exact-ID resolver.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Exact source ID retrieval with an explicitly selected source. Sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq.

- `--civic-cursor`: Source continuation cursor for that exact drug, profile and claim collection.
- `--civic-kind`: Independent source evidence or assertion collection within the selected profile.
- `--civic-profile`: Selected source CIViC molecular profile for a drug-scoped claim stream.
- `input`: Exact source ID; WHO references retain source spaces. No name or alias resolution.
- `--label-section`: Exact source label heading, required only for the label section.
- `--product-number`: Explicit source application product number, only for approvals.
- `section`: Explicit named disclosures; empty means identity only.
- `--section-limit`: Named section page size, refused on an identity-only card.
- `--section-offset`: Named section continuation, refused on an identity-only card.
- `--source`: Required source; there is no default exact-ID resolver.

Explicit named disclosures; empty means identity only.

- `xrefs`: All source identity cross-references and product links, mechanically paged.
- `targets`: Source target and mechanism relations.
- `indications`: Source indication wording.
- `civic`: Source CIViC therapy evidence and profile context.
- `label`: One exact source label heading.
- `approvals`: Source application and product history rows.
- `regulatory`: Source-scoped regulator events.
- `interactions`: Source DDInter partner rows and levels; five rows per default page, with explicit section limits preserved.

## biomcp get gene

```text
Get one to twenty human gene inputs in their original order.

Usage: biomcp get gene [OPTIONS] <input>...

Arguments:
  <input>...
          Gene symbols or checked identifiers, in input order.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --section <section>
          Requested section words, preserving caller order and repetitions.
          Trailing section words select sections. Use --section WORD to keep inputs that equal a section word as inputs.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Get one to twenty human gene inputs in their original order. Sources: civic, clingen, gencc, gnomad, hgnc, hpa, mygene, oncokb, reactome, uniprot.

- `input`: Gene symbols or checked identifiers, in input order.
- `--section`: Requested section words, preserving caller order and repetitions.
Trailing section words select sections. Use --section WORD to keep inputs that equal a section word as inputs.

Requested section words, preserving caller order and repetitions.
Trailing section words select sections. Use --section WORD to keep inputs that equal a section word as inputs.

- `protein`: Reviewed human proteins.
- `civic`: CIViC gene curation.
- `clingen`: ClinGen validity and dosage.
- `gencc`: GenCC submitter assessments.
- `constraint`: Population constraint metrics.
- `expression`: Human Protein Atlas expression.
- `pathways`: Human Reactome pathways.
- `oncokb`: Keyed OncoKB annotations.
- `all`: Every landed section.

## biomcp get article

```text
Get one to twenty article inputs in caller order.

Usage: biomcp get article [OPTIONS] <input>...

Arguments:
  <input>...
          PubMed, PMC or DOI identifiers in input order.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --section <section>
          Requested section words; trailing words and --section share this field.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Get one to twenty article inputs in caller order. Sources: europepmc, icite, pmc-oa, pubmed, pubtator, semantic-scholar.

- `input`: PubMed, PMC or DOI identifiers in input order.
- `--section`: Requested section words; trailing words and --section share this field.

Requested section words; trailing words and --section share this field.

- `annotations`: Normalized text-mined entities from PubTator3.
- `fulltext`: Licensed PMC body outline.
- `supplements`: Supplementary files confirmed by PMC metadata.
- `tldr`: Machine-written Semantic Scholar summary and counts.
- `citations`: Citation counts from NIH iCite.
- `indexing`: MeSH, keywords and author indexing from the citation.
- `pdf`: Publisher PDF link without downloading.
- `all`: Every landed article section.

## biomcp get trial

```text
Retrieve exact trial cards in input order; request bounded intact source sections with independent continuations.

Usage: biomcp get trial [OPTIONS] <input>...

Arguments:
  <input>...
          One to twenty exact NCT identifiers, in caller order.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --limit <limit>
          Whole-record count per source collection page; text fragments follow the text budget.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Character offset for ClinicalTrials.gov eligibility or NCI summary; whole-record offset for other collections.

      --pool <pool>
          Checked stored source record, supplied by a continuation.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --section <section>
          Optional intact source sections. Default cards only; diseases and summary require NCI. NCI locations, contacts, references and documents are unsupported.

      --source <source>
          Explicit provider; defaults to ClinicalTrials.gov and never falls back.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Retrieve exact trial cards in input order; request bounded intact source sections with independent continuations. Sources: ctgov, nci.

- `input`: One to twenty exact NCT identifiers, in caller order.
- `--limit`: Whole-record count per source collection page; text fragments follow the text budget.
- `--offset`: Character offset for ClinicalTrials.gov eligibility or NCI summary; whole-record offset for other collections.
- `--pool`: Checked stored source record, supplied by a continuation.
- `--section`: Optional intact source sections. Default cards only; diseases and summary require NCI. NCI locations, contacts, references and documents are unsupported.
- `--source`: Explicit source; defaults to ClinicalTrials.gov and never falls back.

Optional intact source sections. Default cards only; diseases and summary require NCI. NCI locations, contacts, references and documents are unsupported.

- `eligibility`: Intact criteria and structured age/sex fields.
- `outcomes`: Primary, secondary and other outcome records.
- `arms`: Arms and interventions with original relationships.
- `locations`: Whole source sites with their attached contacts.
- `contacts`: Central contacts and overall officials in their original roles.
- `references`: Registry publication identities and source reference types.
- `documents`: Posted document metadata and source URLs.
- `all`: Every implemented trial section, each with independent continuations.
- `diseases`: Source NCI disease concepts, including source enrichment metadata.
- `summary`: Complete NCI summary, paged by original character offsets.

## biomcp get variant

```text
Read an rsID, CAid, clinvar:N, VCV accession, reference-qualified HGVS or single-base genomic coordinates such as 7-140753336-A-T through the shared resolver. MyVariant identity lookups default to GRCh38; --build overrides it. Chromosome genomic HGVS requires --build. Use search variant --gene BRAF --protein V600E or search variant --gene PTEN --coding c.333G>T for gene shorthand. Every candidate remains available; source assertions do not classify the variant for the caller.

Usage: biomcp get variant [OPTIONS] <input> [section]...

Arguments:
  <input>
          rsID, CAid, clinvar:N, VCV accession, reference-qualified HGVS or single-base genomic coordinates.

  [section]...
          Named source sections; omitted by default. Each calls its own source. all keeps bounded pages and source skips; it never enables a missing key.

Options:
      --build <build>
          MyVariant identity assembly defaults to GRCh38; chromosome genomic HGVS requires an explicit build.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --civic-cursor <civic_cursor>
          Native CIViC cursor, retained unchanged.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --civic-kind <civic_kind>
          Native CIViC claim collection within the selected profile.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --civic-profile <civic_profile>
          Selected CIViC molecular profile for native claim paging.

      --config <config>
          Read configuration from this file.

      --limit <limit>
          Candidate rows per page; more remain reachable through typed next commands.

      --offset <offset>
          Candidate rows to skip in source order.

      --resolver <resolver>
          Resolver for reference-qualified notation; never repairs the input locally.

      --section-limit <section_limit>
          Rows per named source section page; omitted uses the native source default.

      --offline
          Read local data without source requests.

      --section-offset <section_offset>
          Source rows to skip independently of identity candidate paging.

      --fresh
          Request fresh source data instead of cached replies.

      --tumor-type <tumor_type>
          Explicit tumor context for keyed OncoKB annotation.

```

Read an rsID, CAid, clinvar:N, VCV accession, reference-qualified HGVS or single-base genomic coordinates such as 7-140753336-A-T through the shared resolver. Sources: alphagenome, car, civic, clinvar, erepo, gnomad, gwas, hotspots, mutalyzer, myvariant, oncokb, variantvalidator.

- `--build`: MyVariant identity assembly defaults to GRCh38; chromosome genomic HGVS requires an explicit build.
- `--civic-cursor`: Source CIViC cursor, retained unchanged.
- `--civic-kind`: Source CIViC claim collection within the selected profile.
- `--civic-profile`: Selected CIViC molecular profile for source claim paging.
- `input`: rsID, CAid, clinvar:N, VCV accession, reference-qualified HGVS or single-base genomic coordinates.
- `--limit`: Candidate rows per page; more remain reachable through typed next commands.
- `--offset`: Candidate rows to skip in source order.
- `--resolver`: Resolver for reference-qualified notation; never repairs the input locally.
- `section`: Named source sections; omitted by default. Each calls its own source. all keeps bounded pages and source skips; it never enables a missing key.
- `--section-limit`: Rows per named source section page; omitted uses the source default.
- `--section-offset`: Source rows to skip independently of identity candidate paging.
- `--tumor-type`: Explicit tumor context for keyed OncoKB annotation.

Named source sections; omitted by default. Each calls its own source. all keeps bounded pages and source skips; it never enables a missing key.

- `clinvar`: ClinVar source assertions.
- `erepo`: ClinGen source assertions.
- `population`: gnomAD population observations.
- `gwas`: GWAS Catalog associations.
- `civic`: CIViC molecular-profile evidence.
- `alphagenome`: Keyed AlphaGenome source numeric scorer values.
- `oncokb`: Keyed OncoKB annotation with explicit tumor context and build.
- `hotspots`: Cancer Hotspots candidate residue counts; no established exact allele join or clinical actionability.
- `predict`: MyVariant source predictor scores.
- `conservation`: MyVariant conservation scores.
- `cosmic`: Historical COSMIC fragments carried by MyVariant; parent-to-fragment allele identity remains unverified.
- `all`: Every implemented variant section.

## biomcp author

```text
Read papers attributed to a source author record.

Usage: biomcp author [OPTIONS] <COMMAND>

Commands:
  papers  One exact provider record's publication window. ORCID sends its whole public works list in one reply and pages groups locally.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read papers attributed to a source author record.


## biomcp author papers

```text
One exact provider record's publication window. ORCID sends its whole public works list in one reply and pages groups locally.

Usage: biomcp author papers [OPTIONS] <id> [section]

Arguments:
  <id>
          Exact qualified provider record.

  [section]
          Detailed disclosure when requested.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --full
          Legacy alias for details; normalized on construction.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

  -l, --limit <limit>
          Bounded row count.

      --offset <offset>
          Native or retained collection offset.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --pool <pool>
          Retained collection identity.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

One exact source record's publication window. Sources: orcid, semantic-scholar.

- `--full`: Legacy alias for details; normalized on construction.
- `id`: Exact qualified source record.
- `--limit`: Bounded row count.
- `--offset`: Source or retained collection offset.
- `--pool`: Retained collection identity.
- `section`: Detailed disclosure when requested.

Detailed disclosure when requested.

- `details`: Source detailed metadata for the same paper window.

## biomcp cell-line

```text
Read source cell line assay counts.

Usage: biomcp cell-line [OPTIONS] <COMMAND>

Commands:
  assay-count  A separate assay-count request, never implicit in metadata or all.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read source cell line assay counts.


## biomcp cell-line assay-count

```text
A separate assay-count request, never implicit in metadata or all.

Usage: biomcp cell-line assay-count [OPTIONS] <id>

Arguments:
  <id>
          Exact ChEMBL cell identity returned by metadata.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

A separate assay-count request, never implicit in metadata or all. Sources: chembl.

- `id`: Exact ChEMBL cell identity returned by metadata.

## biomcp protein

```text
Read protein sequences, features and source relationships.

Usage: biomcp protein [OPTIONS] <COMMAND>

Commands:
  features         Page five native UniProt features by exact type and displayed residue. No variant effect or clinical meaning is inferred; no bioinformatics tool runs.
  isoforms         Page five source isoforms, exact-participant complexes or PDB links. Source identities stay separate. No structure, interaction or clinical meaning is inferred; no bioinformatics tool runs.
  domains          Page five native InterPro matches with original types and fragments. No structure or clinical meaning is inferred; no bioinformatics tool runs.
  complexes        Page five source isoforms, exact-participant complexes or PDB links. Source identities stay separate. No structure, interaction or clinical meaning is inferred; no bioinformatics tool runs.
  complex-members  Page five native participants of an exact Complex Portal complex containing the anchor. No interaction or clinical meaning is inferred; no bioinformatics tool runs.
  structures       Page five source isoforms, exact-participant complexes or PDB links. Source identities stay separate. No structure, interaction or clinical meaning is inferred; no bioinformatics tool runs.
  sequence         Read one complete native sequence for an exact source accession.
                   No variant effect, structure or clinical meaning is inferred; no bioinformatics tool runs.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read protein sequences, features and source relationships.


## biomcp protein features

```text
Page five native UniProt features by exact type and displayed residue. No variant effect or clinical meaning is inferred; no bioinformatics tool runs.

Usage: biomcp protein features [OPTIONS] <accession>

Arguments:
  <accession>
          Exact source protein sequence identity.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --limit <limit>
          Five rows by default, at most 25.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Native or retained page offset.

      --pool <pool>
          Retained collection identity.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --position <position>
          Native one-based residue; zero fails during deserialization.

      --type <type>
          Exact UniProt native label, without an alias map.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Page five source UniProt features by exact type and displayed residue. Sources: uniprot.

- `accession`: Exact source protein sequence identity.
- `--limit`: Five rows by default, at most 25.
- `--offset`: Source or retained page offset.
- `--pool`: Retained collection identity.
- `--position`: Source one-based residue; zero fails during deserialization.
- `--type`: Exact UniProt source label, without an alias map.

## biomcp protein isoforms

```text
Page five source isoforms, exact-participant complexes or PDB links. Source identities stay separate. No structure, interaction or clinical meaning is inferred; no bioinformatics tool runs.

Usage: biomcp protein isoforms [OPTIONS] <accession>

Arguments:
  <accession>
          Exact source protein sequence identity.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --limit <limit>
          Five rows by default, at most 25.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Native or retained page offset.

      --pool <pool>
          Retained collection identity.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Page five source isoforms, exact-participant complexes or PDB links. Sources: uniprot.

- `accession`: Exact source protein sequence identity.
- `--limit`: Five rows by default, at most 25.
- `--offset`: Source or retained page offset.
- `--pool`: Retained collection identity.

## biomcp protein domains

```text
Page five native InterPro matches with original types and fragments. No structure or clinical meaning is inferred; no bioinformatics tool runs.

Usage: biomcp protein domains [OPTIONS] <accession>

Arguments:
  <accession>
          Exact source protein sequence identity.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --limit <limit>
          Five rows by default, at most 25.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Native or retained page offset.

      --pool <pool>
          Retained collection identity.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --type <type>
          Exact native InterPro type, not a normalized domain label.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Page five source InterPro matches with original types and fragments. Sources: interpro.

- `accession`: Exact source protein sequence identity.
- `--limit`: Five rows by default, at most 25.
- `--offset`: Source or retained page offset.
- `--pool`: Retained collection identity.
- `--type`: Exact source InterPro type, not a normalized domain label.

## biomcp protein complexes

```text
Page five source isoforms, exact-participant complexes or PDB links. Source identities stay separate. No structure, interaction or clinical meaning is inferred; no bioinformatics tool runs.

Usage: biomcp protein complexes [OPTIONS] <accession>

Arguments:
  <accession>
          Exact source protein sequence identity.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --limit <limit>
          Five rows by default, at most 25.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Native or retained page offset.

      --pool <pool>
          Retained collection identity.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Page five source isoforms, exact-participant complexes or PDB links. Sources: complex-portal.

- `accession`: Exact source protein sequence identity.
- `--limit`: Five rows by default, at most 25.
- `--offset`: Source or retained page offset.
- `--pool`: Retained collection identity.

## biomcp protein complex-members

```text
Page five native participants of an exact Complex Portal complex containing the anchor. No interaction or clinical meaning is inferred; no bioinformatics tool runs.

Usage: biomcp protein complex-members [OPTIONS] --complex <complex> <accession>

Arguments:
  <accession>
          Exact source protein sequence identity.

Options:
      --complex <complex>
          Exact complex whose native participants contain the anchor.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --limit <limit>
          Five rows by default, at most 25.

      --offset <offset>
          Native or retained page offset.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --pool <pool>
          Retained collection identity.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Page five source participants of an exact Complex Portal complex containing the anchor. Sources: complex-portal.

- `accession`: Exact source protein sequence identity.
- `--complex`: Exact complex whose source participants contain the anchor.
- `--limit`: Five rows by default, at most 25.
- `--offset`: Source or retained page offset.
- `--pool`: Retained collection identity.

## biomcp protein structures

```text
Page five source isoforms, exact-participant complexes or PDB links. Source identities stay separate. No structure, interaction or clinical meaning is inferred; no bioinformatics tool runs.

Usage: biomcp protein structures [OPTIONS] <accession>

Arguments:
  <accession>
          Exact source protein sequence identity.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --limit <limit>
          Five rows by default, at most 25.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Native or retained page offset.

      --pool <pool>
          Retained collection identity.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Page five source isoforms, exact-participant complexes or PDB links. Sources: uniprot.

- `accession`: Exact source protein sequence identity.
- `--limit`: Five rows by default, at most 25.
- `--offset`: Source or retained page offset.
- `--pool`: Retained collection identity.

## biomcp protein sequence

```text
Read one complete native sequence for an exact source accession.
No variant effect, structure or clinical meaning is inferred; no bioinformatics tool runs.

Usage: biomcp protein sequence [OPTIONS] <isoform>

Arguments:
  <isoform>
          Exact isoform or base accession; no canonical suffix is synthesized.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read one complete source sequence for an exact source accession.
No variant effect, structure or clinical meaning is inferred; no bioinformatics tool runs. Sources: uniprot.

- `isoform`: Exact isoform or base accession; no canonical suffix is synthesized.

## biomcp pathway

```text
Read pathway genes and source events.

Usage: biomcp pathway [OPTIONS] <COMMAND>

Commands:
  genes   Read Reactome exact references or WikiPathways GeneProduct data nodes. The details layer discloses complete Reactome physical contexts. Membership names a source and release; it implies no therapy, response or actionability. No enrichment is calculated.
  events  Read Reactome event objects and unresolved native integer references in source order. Membership names a source and release; it implies no therapy, response or actionability. No enrichment is calculated.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read pathway genes and source events.


## biomcp pathway genes

```text
Read Reactome exact references or WikiPathways GeneProduct data nodes. The details layer discloses complete Reactome physical contexts. Membership names a source and release; it implies no therapy, response or actionability. No enrichment is calculated.

Usage: biomcp pathway genes [OPTIONS] <id> [details]

Arguments:
  <id>
          Checked source-owned human pathway identity.

  [details]
          Include every physical entity context for these references; never clips source text.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

  -l, --limit <limit>
          Five complete relation rows by default, at most fifty.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Offset within the retained selected relation.

      --pool <pool>
          Immutable relation snapshot.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read Reactome exact references or WikiPathways GeneProduct data nodes. Sources: reactome, wikipathways.

- `details`: Include every physical entity context for these references; never clips source text.
- `id`: Checked source-owned human pathway identity.
- `--limit`: Five complete relation rows by default, at most fifty.
- `--offset`: Offset within the retained selected relation.
- `--pool`: Immutable relation snapshot.

## biomcp pathway events

```text
Read Reactome event objects and unresolved native integer references in source order. Membership names a source and release; it implies no therapy, response or actionability. No enrichment is calculated.

Usage: biomcp pathway events [OPTIONS] <id>

Arguments:
  <id>
          Checked source-owned human pathway identity.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

  -l, --limit <limit>
          Five complete relation rows by default, at most fifty.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Offset within the retained selected relation.

      --pool <pool>
          Immutable relation snapshot.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read Reactome event objects and unresolved source integer references in source order. Sources: reactome.

- `id`: Checked source-owned human pathway identity.
- `--limit`: Five complete relation rows by default, at most fifty.
- `--offset`: Offset within the retained selected relation.
- `--pool`: Immutable relation snapshot.

## biomcp article

```text
Read article text, references, citations and files.

Usage: biomcp article [OPTIONS] <COMMAND>

Commands:
  supplement         Read a checked supplement file or page its original rows.
  pdf                Download one publisher PDF to an explicit file or existing folder.
  text               Read one bounded body part, find matching rows, or save the complete Markdown document.
  citations          Page the citing or referenced PubMed papers of one checked PMID.
  references         Page the citing or referenced PubMed papers of one checked PMID.
  citation-evidence  Read every source context and optional explicit full-text citation paragraph.
  recommendations    Recommendations in the provider's recent-paper pool order.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read article text, references, citations and files.


## biomcp article supplement

```text
Read a checked supplement file or page its original rows.

Usage: biomcp article supplement [OPTIONS] <input> <file>

Arguments:
  <input>
          PubMed or PMC identity.

  <file>
          Exact file name from the supplements section.

Options:
      --find <find>
          Literal case-insensitive needle.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Matching-row or text-fragment offset.

      --replace
          Replace an existing destination only when explicitly requested with save.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --rows <rows>
          Inclusive original row range.

      --save <save>
          Save the whole original document to an explicit file or existing folder.

      --sheet <sheet>
          Sheet name for row selection.

      --variant <variant>
          Explicit protein substitution.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read a checked supplement file or page its original rows. Sources: pmc-oa, pubmed.

- `file`: Exact file name from the supplements section.
- `--find`: Exact case-insensitive needle.
- `input`: PubMed or PMC identity.
- `--offset`: Matching-row or text-fragment offset.
- `--replace`: Replace an existing destination only when explicitly requested with save.
- `--rows`: Inclusive original row range.
- `--save`: Save the whole original document to an explicit file or existing folder.
- `--sheet`: Sheet name for row selection.
- `--variant`: Explicit protein substitution.

## biomcp article pdf

```text
Download one publisher PDF to an explicit file or existing folder.

Usage: biomcp article pdf [OPTIONS] --save <save> <input>

Arguments:
  <input>
          Article identity.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --replace
          Replace an existing file only when explicitly requested.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --save <save>
          Destination file or existing folder.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Download one publisher PDF to an explicit file or existing folder. Sources: semantic-scholar.

- `input`: Article identity.
- `--replace`: Replace an existing file only when explicitly requested.
- `--save`: Destination file or existing folder.

## biomcp article text

```text
Read one bounded body part, find matching rows, or save the complete Markdown document.

Usage: biomcp article text [OPTIONS] <input>

Arguments:
  <input>
          PubMed or PMC identity. DOI selectors are not accepted by the reader.

Options:
      --find <find>
          Case-insensitive literal text to find.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Number of matching rows already returned.

      --part <part>
          One-based outline part number.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --replace
          Replace an existing destination only when explicitly requested with save.

      --save <save>
          Save the whole original document to an explicit file or existing folder.

      --variant <variant>
          Explicit checked protein substitution.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read one bounded body part, find matching rows, or save the complete Markdown document. Sources: europepmc, pmc-oa, pubmed.

- `--find`: Case-insensitive exact text to find.
- `input`: PubMed or PMC identity. DOI selectors are not accepted by the reader.
- `--offset`: Number of matching rows already returned.
- `--part`: One-based outline part number.
- `--replace`: Replace an existing destination only when explicitly requested with save.
- `--save`: Save the whole original document to an explicit file or existing folder.
- `--variant`: Explicit checked protein substitution.

## biomcp article citations

```text
Page the citing or referenced PubMed papers of one checked PMID.

Usage: biomcp article citations [OPTIONS] <pmid>

Arguments:
  <pmid>
          PMID, PMC or DOI; iCite accepts only a PMID.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --limit <limit>
          Maximum papers on one page.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Number of pool items to skip.

      --pool <pool>
          Fixed snapshot from an earlier page.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --source <source>
          Graph provider; iCite remains the default.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Page the citing or referenced PubMed papers of one checked PMID. Sources: icite, pubmed, semantic-scholar.

- `--limit`: Maximum papers on one page.
- `--offset`: Number of pool items to skip.
- `pmid`: PMID, PMC or DOI; iCite accepts only a PMID.
- `--pool`: Fixed snapshot from an earlier page.
- `--source`: Graph source; iCite remains the default.

## biomcp article references

```text
Page the citing or referenced PubMed papers of one checked PMID.

Usage: biomcp article references [OPTIONS] <pmid>

Arguments:
  <pmid>
          PMID, PMC or DOI; iCite accepts only a PMID.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --limit <limit>
          Maximum papers on one page.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Number of pool items to skip.

      --pool <pool>
          Fixed snapshot from an earlier page.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --source <source>
          Graph provider; iCite remains the default.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Page the citing or referenced PubMed papers of one checked PMID. Sources: icite, pubmed, semantic-scholar.

- `--limit`: Maximum papers on one page.
- `--offset`: Number of pool items to skip.
- `pmid`: PMID, PMC or DOI; iCite accepts only a PMID.
- `--pool`: Fixed snapshot from an earlier page.
- `--source`: Graph source; iCite remains the default.

## biomcp article citation-evidence

```text
Read every source context and optional explicit full-text citation paragraph.

Usage: biomcp article citation-evidence [OPTIONS] <citing> <cited>

Arguments:
  <citing>
          Citing PMID, PMC identity or DOI.

  <cited>
          Cited PMID, PMC identity or DOI.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --fulltext
          Include the citing paper's own explicit reference paragraphs.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --offset <offset>
          Context fragment position returned by the advancing step.

      --paragraph-offset <paragraph_offset>
          Full-text fragment position returned by the advancing step.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --pool <pool>
          Stored checked source scan and original hit positions.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read every source context and optional explicit full-text citation paragraph. Sources: pmc-oa, pubmed, semantic-scholar.

- `cited`: Cited PMID, PMC identity or DOI.
- `citing`: Citing PMID, PMC identity or DOI.
- `--fulltext`: Include the citing paper's own explicit reference paragraphs.
- `--offset`: Context fragment position returned by the advancing step.
- `--paragraph-offset`: Full-text fragment position returned by the advancing step.
- `--pool`: Stored checked source scan and original hit positions.

## biomcp article recommendations

```text
Recommendations in the provider's recent-paper pool order.

Usage: biomcp article recommendations [OPTIONS] <seeds>...

Arguments:
  <seeds>...
          One to ten positive paper identities.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --limit <limit>
          Maximum recommendations requested, in native order.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --negative <negative>...
          Up to ten negative paper identities.

      --offset <offset>
          Number of stored recommendations to skip.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --pool <pool>
          Fixed recommendation pool.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Recommendations in the source's recent-paper pool order. Sources: semantic-scholar.

- `--limit`: Maximum recommendations requested, in source order.
- `--negative`: Up to ten negative paper identities.
- `--offset`: Number of stored recommendations to skip.
- `--pool`: Fixed recommendation pool.
- `seeds`: One to ten positive paper identities.

## biomcp trial

```text
Read trial groups and public documents.

Usage: biomcp trial [OPTIONS] <COMMAND>

Commands:
  document  Download one exact ClinicalTrials.gov manifest PDF to an explicit caller path; never read its text. CLI and stdio MCP support saves; HTTP refuses with the source URL.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read trial groups and public documents.


## biomcp trial document

```text
Download one exact ClinicalTrials.gov manifest PDF to an explicit caller path; never read its text. CLI and stdio MCP support saves; HTTP refuses with the source URL.

Usage: biomcp trial document [OPTIONS] --save <save> <input> <file>

Arguments:
  <input>
          Exact registry identity.

  <file>
          Exact filename from the registry manifest; never a URL or inferred path.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --replace
          Replace an existing file only when explicitly requested.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --save <save>
          Caller-owned file or existing folder.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Download one exact ClinicalTrials.gov manifest PDF to an explicit caller path; never read its text. Sources: ctgov.

- `file`: Exact filename from the registry manifest; never a URL or inferred path.
- `input`: Exact registry identity.
- `--replace`: Replace an existing file only when explicitly requested.
- `--save`: Caller-owned file or existing folder.

## biomcp variant

```text
Read source variant assertions, populations and recurrence.

Usage: biomcp variant [OPTIONS] <COMMAND>

Commands:
  assertions  Page assertions from one explicitly selected evidence source.
  population  Read a bounded ancestry-detail page from the exact source-given GRCh38 allele.
  recurrence  Observe exact-allele samples within one caller-selected source page.
  normalize   Send unchanged reference-qualified notation to explicitly named services.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read source variant assertions, populations and recurrence.


## biomcp variant assertions

```text
Page assertions from one explicitly selected evidence source.

Usage: biomcp variant assertions [OPTIONS] --source <source> <input>

Arguments:
  <input>
          Supported variant ID; resolves one ClinVar Variation ID or ERepo CAid through the shared resolver.

Options:
      --assertion <assertion>
          Native assertion accession or document ID.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --build <build>
          Identity lookup assembly; omitted uses the shared GRCh38 default.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --limit <limit>
          Assertions per page; omitted uses the native source default.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offset <offset>
          Assertions to skip in source order.

      --source <source>
          One native assertion source; shared identity conversion precedes source access when needed.

      --version <version>
          VCV archive or ERepo document version; RCV/SCV selectors match a returned native version.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Page assertions from one explicitly selected evidence source. Sources: clinvar, erepo.

- `--assertion`: Source assertion accession or document ID.
- `--build`: Identity lookup assembly; omitted uses the shared GRCh38 default.
- `input`: Supported variant ID; resolves one ClinVar Variation ID or ERepo CAid through the shared resolver.
- `--limit`: Assertions per page; omitted uses the source default.
- `--offset`: Assertions to skip in source order.
- `--source`: One source assertion source; shared identity conversion precedes source access when needed.
- `--version`: VCV archive or ERepo document version; RCV/SCV selectors match a returned source version.

## biomcp variant population

```text
Read a bounded ancestry-detail page from the exact source-given GRCh38 allele.

Usage: biomcp variant population [OPTIONS] <input>

Arguments:
  <input>
          Exact published allele identity; ambiguous candidates must first be selected.

Options:
      --build <build>
          Explicit source assembly when required by identity input.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --detail
          Read source ancestry groups, preserving overlapping groups individually.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --limit <limit>
          Maximum rows per page.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offset <offset>
          Native rows to skip.

      --resolver <resolver>
          Resolver for reference-qualified input.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read a bounded ancestry-detail page from the exact source-given GRCh38 allele. Sources: car, gnomad, mutalyzer, myvariant, variantvalidator.

- `--build`: Explicit source assembly when required by identity input.
- `--detail`: Read source ancestry groups, preserving overlapping groups individually.
- `input`: Exact published allele identity; ambiguous candidates must first be selected.
- `--limit`: Maximum rows per page.
- `--offset`: Source rows to skip.
- `--resolver`: Resolver for reference-qualified input.

## biomcp variant recurrence

```text
Observe exact-allele samples within one caller-selected source page.

Usage: biomcp variant recurrence [OPTIONS] --gene <gene> --profile <profile> --sample-set <sample_set> --source <source> --study <study> <input>

Arguments:
  <input>
          Published variant identity; ambiguous identities are never selected silently.

Options:
      --build <build>
          Identity lookup assembly; omitted uses the shared GRCh38 default.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --gene <gene>
          HGNC symbol resolved to the source's Entrez gene identifier.

      --limit <limit>
          Source mutation rows per page, including excluded alleles.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --page <page>
          Zero-based native source page.

      --profile <profile>
          Explicit mutation molecular profile.

      --sample-set <sample_set>
          Explicit sample list; selected size is never an allele denominator.

      --source <source>
          Explicit cohort source.

      --study <study>
          Explicit study; there is no default cohort.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Observe exact-allele samples within one caller-selected source page. Sources: cbioportal.

- `--build`: Identity lookup assembly; omitted uses the shared GRCh38 default.
- `--gene`: HGNC symbol resolved to the source's Entrez gene identifier.
- `input`: Published variant identity; ambiguous identities are never selected silently.
- `--limit`: Source mutation rows per page, including excluded alleles.
- `--page`: Zero-based source page.
- `--profile`: Explicit mutation molecular profile.
- `--sample-set`: Explicit sample list; selected size is never an allele denominator.
- `--source`: Explicit cohort source.
- `--study`: Explicit study; there is no default cohort.

## biomcp variant normalize

```text
Send unchanged reference-qualified notation to explicitly named services.

Usage: biomcp variant normalize [OPTIONS] --hgvs <hgvs> --source <source>

Options:
      --build <build>
          Caller assembly; required when VariantValidator is selected.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --hgvs <hgvs>
          Reference-qualified HGVS, or chromosome genomic HGVS with an explicit build for CAR.
          Checked for reference and syntax only; no full HGVS parser.

      --limit <limit>
          Source mapping rows per service, with typed continuation for remaining rows.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offset <offset>
          Source mapping rows to skip per service.

      --source <source>
          Explicit service; all preserves independent results without consensus.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Send unchanged reference-qualified notation to explicitly named services. Sources: car, mutalyzer, variantvalidator.

- `--build`: Caller assembly; required when VariantValidator is selected.
- `--hgvs`: Reference-qualified HGVS, or chromosome genomic HGVS with an explicit build for CAR.
Checked for reference and syntax only; no full HGVS parser.
- `--limit`: Source mapping rows per service, with typed continuation for remaining rows.
- `--offset`: Source mapping rows to skip per service.
- `--source`: Explicit service; all preserves independent results without consensus.

## biomcp sync

```text
Synchronize a local source bundle.

Usage: biomcp sync [OPTIONS] [bundle]

Arguments:
  [bundle]
          Local source bundle to synchronize.

          [default: hgnc]
          [possible values: hgnc, clingen, gencc, wikipathways, cbioportal-studies, ema, ddinter, who-pq, gtr, who-ivd, fda-cdx]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Synchronize a local source bundle.

- `bundle`: Local source bundle to synchronize.

## biomcp skill

```text
Read embedded guidance and worked examples. No source requests. Overview and catalog are bounded by the embedded payload; install and status return one summary. Install writes only inside DIR and refuses symbolic links and parent traversal.

Usage: biomcp skill [OPTIONS] [COMMAND]

Commands:
  list     List embedded worked examples.
  render   Print the canonical agent guide.
  status   Report missing, unmanaged, locally_modified, stale or current guidance.
  install  Install into DIR/skills/biomcp; a skills or biomcp directory is accepted directly. Existing files remain unchanged without --force.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

Example: biomcp skill list

```

Read embedded guidance and worked examples. No source requests. Overview and catalog are bounded by the embedded payload; install and status return one summary. Install writes only inside DIR and refuses symbolic links and parent traversal.


## biomcp skill list

```text
List embedded worked examples.

Usage: biomcp skill list [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

Example: biomcp skill list

```

List embedded worked examples.


## biomcp skill render

```text
Print the canonical agent guide.

Usage: biomcp skill render [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

Example: biomcp skill render

```

Print the canonical agent guide.


## biomcp skill status

```text
Report missing, unmanaged, locally_modified, stale or current guidance.

Usage: biomcp skill status [OPTIONS] <directory>

Arguments:
  <directory>
          Directory to inspect or install BioMCP guidance.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

Example: biomcp skill status biomcp-guidance

```

Report missing, unmanaged, locally_modified, stale or current guidance.

- `directory`: Directory to inspect or install BioMCP guidance.

## biomcp skill install

```text
Install into DIR/skills/biomcp; a skills or biomcp directory is accepted directly. Existing files remain unchanged without --force.

Usage: biomcp skill install [OPTIONS] <directory>

Arguments:
  <directory>
          Directory to inspect or install BioMCP guidance.

Options:
      --force
          Repair managed files; retain unrelated files.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

Example: biomcp skill install biomcp-guidance

```

Install into DIR/skills/biomcp; a skills or biomcp directory is accepted directly. Existing files remain unchanged without --force.

- `directory`: Directory to inspect or install BioMCP guidance.
- `--force`: Repair managed files; retain unrelated files.

## biomcp version

```text
Print version; --verbose adds build, schema and path provenance. No source requests or key reads.

Usage: biomcp version [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --verbose
          Include commit, target, schema versions, config and cache paths.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

Example: biomcp version

```

Print version; --verbose adds build, schema and path provenance. No source requests or key reads.

- `--verbose`: Include commit, target, schema versions, config and cache paths.

## biomcp mcp-config

```text
Print MCP configuration with this executable’s absolute path. JSON formats name their top-level object and retain released client aliases. The codex client prints its shell registration command. No source requests or key reads.

Usage: biomcp mcp-config [OPTIONS]

Options:
      --client <client>
          Choose a JSON client or the codex shell registration command; default mcp-servers.

          [default: mcp-servers]
          [possible values: mcp-servers, servers, codex]

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

Example: biomcp mcp-config

```

Print MCP configuration with this executable’s absolute path. JSON formats name their top-level object and retain released client aliases. The codex client prints its shell registration command. No source requests or key reads.

- `--client`: Choose a JSON client or the codex shell registration command; default mcp-servers.

## biomcp health

```text
Report source terms and bounded reachability.

Usage: biomcp health [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

Example: biomcp health --offline

```

Report publisher terms and bounded source reachability; at most one page for each implemented source. BioMCP's local request deadline expired. GWAS Catalog response times can vary; retry later or use https://www.ebi.ac.uk/gwas/. Contact the GWAS Catalog helpdesk if this persists. BioMCP's local request deadline expired. The health probe uses a shorter local cap than protein requests. InterPro documents that some queries take minutes; this timeout does not establish an outage. Please retry later or check https://www.ebi.ac.uk/interpro/. See https://interpro-documentation.readthedocs.io/en/latest/faq.html. CDC WONDER can take longer than BioMCP's local command deadline. Use the interactive form at https://wonder.cdc.gov/vaers.html for a longer query. See https://wonder.cdc.gov/wonder/help/faq.html. BioMCP's local quota refused this request before contacting CDC WONDER. Retry after the reported seconds left. Automated queries must follow the spacing at https://wonder.cdc.gov/wonder/help/wonder-api.html.


## biomcp store

```text
Manage local cached replies and snapshots.

Usage: biomcp store [OPTIONS] <COMMAND>

Commands:
  clean

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Manage local cached replies and snapshots.


## biomcp store clean

```text
Usage: biomcp store clean [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

Example: biomcp store clean

```

Remove expired replies, pools and snapshots and older incompatible cache files. Return removal counts in one bounded reply. Fresh state and bundles remain.


## biomcp key

```text
Register and manage source credentials without showing values.

Usage: biomcp key [OPTIONS] <COMMAND>

Commands:
  list    List source credentials and their origins without showing values.
  set
  remove

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

Example: biomcp key list

```

Register and manage source credentials without showing values.


## biomcp key list

```text
List source credentials and their origins without showing values.

Usage: biomcp key list [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

Example: biomcp key list

```

List source credentials and their origins without showing values.


## biomcp key set

```text
Usage: biomcp key set [OPTIONS] <source>

Arguments:
  <source>
          Source credential name to register or remove.

          [possible values: ncbi, semantic-scholar, orcid, oncokb, alphagenome, nci, openfda]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --replace
          Replace an existing stored credential.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

Example: biomcp key set oncokb

```

Register one credential from a no-echo prompt or one standard-input line. Use --config PATH to choose the configuration file.

- `source`: Source credential name to register or remove.
- `--replace`: Replace an existing stored credential.

## biomcp key remove

```text
Usage: biomcp key remove [OPTIONS] <source>

Arguments:
  <source>
          Source credential name to register or remove.

          [possible values: ncbi, semantic-scholar, orcid, oncokb, alphagenome, nci, openfda]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

Example: biomcp key remove oncokb

```

Remove a stored credential while preserving other settings.

- `source`: Source credential name to register or remove.

## biomcp serve

```text
Serve MCP over standard input or HTTP.

Usage: biomcp serve [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --hosted
          Refuse host file saves, health details and store or key changes over MCP.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --rate-share <rate-share>
          Use this share of keyed source rates in hosted mode (greater than 0, at most 1).

          [default: 1]

      --http [<http>]
          Listen on an HTTP address; omit its value for the loopback default.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --http_max_calls <http_max_calls>
          Limit tool calls in flight.

      --http_max_connections <http_max_connections>
          Limit open HTTP connections; must cover http_max_calls.

      --allow-host <allow-host>
          Allow an HTTP Host authority; required for non-loopback listeners.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

Example: biomcp serve
Example: biomcp help serve

```

Serve MCP over stdio or --http [ADDR]. Non-loopback binds require --allow-host HOST or http.allowed_hosts; neither authenticates callers. HTTP refuses sync, store clean and --fresh. Stdio saves refuse --replace. --hosted refuses file saves, health details, sync, store clean, --fresh and key commands on every transport with a usage.mcp_unavailable error. MCP revisions: 2026-07-28, 2025-11-25, 2025-06-18, 2025-03-26. --rate-share scales only keyed hosted source rates. Hosted HTTP calls have bounded work and one whole-call deadline. Send MCP-Protocol-Version on HTTP requests. Without it, initialize uses its requested legacy revision; other requests default to 2025-03-26. Unsupported headers return HTTP 400.

- `--hosted`: Refuse host file saves, health details and store or key changes over MCP.
- `--rate-share`: Use this share of keyed source rates in hosted mode (greater than 0, at most 1).
- `--http`: Listen on an HTTP address; omit its value for the loopback default.
- `--http_max_calls`: Limit tool calls in flight.
- `--http_max_connections`: Limit open HTTP connections; must cover http_max_calls.
- `--allow-host`: Allow an HTTP Host authority; required for non-loopback listeners.

## biomcp schema

```text
List every schema identity and description with biomcp schema, or print one schema artifact with biomcp schema <id>.

Usage: biomcp schema [OPTIONS] [artifact-id]

Arguments:
  [artifact-id]
          Artifact ID to print; omit it to list available schemas.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

JSON reply readers ignore unknown fields and unknown growable enum values within a major. Query schemas remain closed.

Example: biomcp schema
Example: biomcp schema biomcp.help/6

```

List every schema identity and description with biomcp schema, or print one schema artifact with biomcp schema <id>.

- `artifact-id`: Artifact ID to print; omit it to list available schemas.

## biomcp help

```text
Describe commands and their arguments.

Usage: biomcp help [OPTIONS] [COMMAND]

Commands:
  search      Search source records by entity and checked filters.
  get         Retrieve one entity and its source sections.
  author      Read papers attributed to a source author record.
  cell-line   Read source cell line assay counts.
  protein     Read protein sequences, features and source relationships.
  pathway     Read pathway genes and source events.
  article     Read article text, references, citations and files.
  trial       Read trial groups and public documents.
  variant     Read source variant assertions, populations and recurrence.
  sync        Synchronize a local source bundle.
  skill
  version
  mcp-config
  health      Report source terms and bounded reachability.
  store       Manage local cached replies and snapshots.
  key         Register and manage source credentials without showing values.
  serve       Serve MCP over standard input or HTTP.
  schema
  help        Describe commands and their arguments.
  network
  sources
  config
  exit        Explain process exit and error codes.
  relations   Map entities through sections and search filters.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Print generated command help.

- `command`: Command path or discovery topic to describe.

## biomcp help search

```text
Search source records by entity and checked filters.

Usage: biomcp help search [OPTIONS] [COMMAND]

Commands:
  biomarker
  author         Read papers attributed to a source author record.
  organization
  cell-line      Read source cell line assay counts.
  gwas
  protein        Read protein sequences, features and source relationships.
  phenotype
  pathway        Read pathway genes and source events.
  study
  pgx
  diagnostic
  adverse-event
  disease
  drug
  gene
  article        Read article text, references, citations and files.
  trial          Read trial groups and public documents.
  variant        Read source variant assertions, populations and recurrence.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Search source records by entity and checked filters.


## biomcp help search biomarker

```text
Usage: biomcp help search biomarker [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help search biomarker.


## biomcp help search author

```text
Read papers attributed to a source author record.

Usage: biomcp help search author [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read papers attributed to a source author record.


## biomcp help search organization

```text
Usage: biomcp help search organization [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help search organization.


## biomcp help search cell-line

```text
Read source cell line assay counts.

Usage: biomcp help search cell-line [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read source cell line assay counts.


## biomcp help search gwas

```text
Usage: biomcp help search gwas [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help search gwas.


## biomcp help search protein

```text
Read protein sequences, features and source relationships.

Usage: biomcp help search protein [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read protein sequences, features and source relationships.


## biomcp help search phenotype

```text
Usage: biomcp help search phenotype [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help search phenotype.


## biomcp help search pathway

```text
Read pathway genes and source events.

Usage: biomcp help search pathway [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read pathway genes and source events.


## biomcp help search study

```text
Usage: biomcp help search study [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help search study.


## biomcp help search pgx

```text
Usage: biomcp help search pgx [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help search pgx.


## biomcp help search diagnostic

```text
Usage: biomcp help search diagnostic [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help search diagnostic.


## biomcp help search adverse-event

```text
Usage: biomcp help search adverse-event [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help search adverse-event.


## biomcp help search disease

```text
Usage: biomcp help search disease [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help search disease.


## biomcp help search drug

```text
Usage: biomcp help search drug [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help search drug.


## biomcp help search gene

```text
Usage: biomcp help search gene [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help search gene.


## biomcp help search article

```text
Read article text, references, citations and files.

Usage: biomcp help search article [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read article text, references, citations and files.


## biomcp help search trial

```text
Read trial groups and public documents.

Usage: biomcp help search trial [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read trial groups and public documents.


## biomcp help search variant

```text
Read source variant assertions, populations and recurrence.

Usage: biomcp help search variant [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read source variant assertions, populations and recurrence.


## biomcp help get

```text
Retrieve one entity and its source sections.

Usage: biomcp help get [OPTIONS] [COMMAND]

Commands:
  biomarker
  author         Read papers attributed to a source author record.
  cell-line      Read source cell line assay counts.
  gwas
  protein        Read protein sequences, features and source relationships.
  phenotype
  pathway        Read pathway genes and source events.
  study
  pgx
  diagnostic
  adverse-event
  disease
  drug
  gene
  article        Read article text, references, citations and files.
  trial          Read trial groups and public documents.
  variant        Read source variant assertions, populations and recurrence.
  clingen

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Retrieve one entity and its source sections.


## biomcp help get biomarker

```text
Usage: biomcp help get biomarker [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help get biomarker.


## biomcp help get author

```text
Read papers attributed to a source author record.

Usage: biomcp help get author [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read papers attributed to a source author record.


## biomcp help get cell-line

```text
Read source cell line assay counts.

Usage: biomcp help get cell-line [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read source cell line assay counts.


## biomcp help get gwas

```text
Usage: biomcp help get gwas [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help get gwas.


## biomcp help get protein

```text
Read protein sequences, features and source relationships.

Usage: biomcp help get protein [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read protein sequences, features and source relationships.


## biomcp help get phenotype

```text
Usage: biomcp help get phenotype [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help get phenotype.


## biomcp help get pathway

```text
Read pathway genes and source events.

Usage: biomcp help get pathway [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read pathway genes and source events.


## biomcp help get study

```text
Usage: biomcp help get study [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help get study.


## biomcp help get pgx

```text
Usage: biomcp help get pgx [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help get pgx.


## biomcp help get diagnostic

```text
Usage: biomcp help get diagnostic [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help get diagnostic.


## biomcp help get adverse-event

```text
Usage: biomcp help get adverse-event [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help get adverse-event.


## biomcp help get disease

```text
Usage: biomcp help get disease [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help get disease.


## biomcp help get drug

```text
Usage: biomcp help get drug [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help get drug.


## biomcp help get gene

```text
Usage: biomcp help get gene [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help get gene.


## biomcp help get article

```text
Read article text, references, citations and files.

Usage: biomcp help get article [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read article text, references, citations and files.


## biomcp help get trial

```text
Read trial groups and public documents.

Usage: biomcp help get trial [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read trial groups and public documents.


## biomcp help get variant

```text
Read source variant assertions, populations and recurrence.

Usage: biomcp help get variant [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read source variant assertions, populations and recurrence.


## biomcp help get clingen

```text
Usage: biomcp help get clingen [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help get clingen.


## biomcp help author

```text
Read papers attributed to a source author record.

Usage: biomcp help author [OPTIONS] [COMMAND]

Commands:
  papers

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read papers attributed to a source author record.


## biomcp help author papers

```text
Usage: biomcp help author papers [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help author papers.


## biomcp help cell-line

```text
Read source cell line assay counts.

Usage: biomcp help cell-line [OPTIONS] [COMMAND]

Commands:
  assay-count

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read source cell line assay counts.


## biomcp help cell-line assay-count

```text
Usage: biomcp help cell-line assay-count [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help cell-line assay-count.


## biomcp help protein

```text
Read protein sequences, features and source relationships.

Usage: biomcp help protein [OPTIONS] [COMMAND]

Commands:
  features
  isoforms
  domains
  complexes
  complex-members
  structures
  sequence

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read protein sequences, features and source relationships.


## biomcp help protein features

```text
Usage: biomcp help protein features [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help protein features.


## biomcp help protein isoforms

```text
Usage: biomcp help protein isoforms [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help protein isoforms.


## biomcp help protein domains

```text
Usage: biomcp help protein domains [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help protein domains.


## biomcp help protein complexes

```text
Usage: biomcp help protein complexes [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help protein complexes.


## biomcp help protein complex-members

```text
Usage: biomcp help protein complex-members [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help protein complex-members.


## biomcp help protein structures

```text
Usage: biomcp help protein structures [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help protein structures.


## biomcp help protein sequence

```text
Usage: biomcp help protein sequence [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help protein sequence.


## biomcp help pathway

```text
Read pathway genes and source events.

Usage: biomcp help pathway [OPTIONS] [COMMAND]

Commands:
  genes
  events

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read pathway genes and source events.


## biomcp help pathway genes

```text
Usage: biomcp help pathway genes [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help pathway genes.


## biomcp help pathway events

```text
Usage: biomcp help pathway events [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help pathway events.


## biomcp help article

```text
Read article text, references, citations and files.

Usage: biomcp help article [OPTIONS] [COMMAND]

Commands:
  supplement
  pdf
  text
  citations
  references
  citation-evidence
  recommendations

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read article text, references, citations and files.


## biomcp help article supplement

```text
Usage: biomcp help article supplement [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help article supplement.


## biomcp help article pdf

```text
Usage: biomcp help article pdf [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help article pdf.


## biomcp help article text

```text
Usage: biomcp help article text [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help article text.


## biomcp help article citations

```text
Usage: biomcp help article citations [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help article citations.


## biomcp help article references

```text
Usage: biomcp help article references [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help article references.


## biomcp help article citation-evidence

```text
Usage: biomcp help article citation-evidence [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help article citation-evidence.


## biomcp help article recommendations

```text
Usage: biomcp help article recommendations [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help article recommendations.


## biomcp help trial

```text
Read trial groups and public documents.

Usage: biomcp help trial [OPTIONS] [COMMAND]

Commands:
  document

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read trial groups and public documents.


## biomcp help trial document

```text
Usage: biomcp help trial document [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help trial document.


## biomcp help variant

```text
Read source variant assertions, populations and recurrence.

Usage: biomcp help variant [OPTIONS] [COMMAND]

Commands:
  assertions
  population
  recurrence
  normalize

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Read source variant assertions, populations and recurrence.


## biomcp help variant assertions

```text
Usage: biomcp help variant assertions [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help variant assertions.


## biomcp help variant population

```text
Usage: biomcp help variant population [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help variant population.


## biomcp help variant recurrence

```text
Usage: biomcp help variant recurrence [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help variant recurrence.


## biomcp help variant normalize

```text
Usage: biomcp help variant normalize [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help variant normalize.


## biomcp help sync

```text
Synchronize a local source bundle.

Usage: biomcp help sync [OPTIONS] [COMMAND]

Commands:
  hgnc
  clingen
  gencc
  wikipathways
  cbioportal-studies
  ema
  ddinter
  who-pq
  gtr
  who-ivd
  fda-cdx

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Synchronize a local source bundle.


## biomcp help sync hgnc

```text
Usage: biomcp help sync hgnc [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help sync hgnc.


## biomcp help sync clingen

```text
Usage: biomcp help sync clingen [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help sync clingen.


## biomcp help sync gencc

```text
Usage: biomcp help sync gencc [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help sync gencc.


## biomcp help sync wikipathways

```text
Usage: biomcp help sync wikipathways [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help sync wikipathways.


## biomcp help sync cbioportal-studies

```text
Usage: biomcp help sync cbioportal-studies [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help sync cbioportal-studies.


## biomcp help sync ema

```text
Usage: biomcp help sync ema [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help sync ema.


## biomcp help sync ddinter

```text
Usage: biomcp help sync ddinter [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help sync ddinter.


## biomcp help sync who-pq

```text
Usage: biomcp help sync who-pq [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help sync who-pq.


## biomcp help sync gtr

```text
Usage: biomcp help sync gtr [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help sync gtr.


## biomcp help sync who-ivd

```text
Usage: biomcp help sync who-ivd [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help sync who-ivd.


## biomcp help sync fda-cdx

```text
Usage: biomcp help sync fda-cdx [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help sync fda-cdx.


## biomcp help skill

```text
Usage: biomcp help skill [OPTIONS] [COMMAND]

Commands:
  list     List supported discovery topics.
  render
  status
  install

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help skill.


## biomcp help skill list

```text
List supported discovery topics.

Usage: biomcp help skill list [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

List supported discovery topics.


## biomcp help skill render

```text
Usage: biomcp help skill render [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help skill render.


## biomcp help skill status

```text
Usage: biomcp help skill status [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help skill status.


## biomcp help skill install

```text
Usage: biomcp help skill install [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help skill install.


## biomcp help version

```text
Usage: biomcp help version [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help version.


## biomcp help mcp-config

```text
Usage: biomcp help mcp-config [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help mcp-config.


## biomcp help health

```text
Report source terms and bounded reachability.

Usage: biomcp help health [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Report source terms and bounded reachability.


## biomcp help store

```text
Manage local cached replies and snapshots.

Usage: biomcp help store [OPTIONS] [COMMAND]

Commands:
  clean

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Manage local cached replies and snapshots.


## biomcp help store clean

```text
Usage: biomcp help store clean [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help store clean.


## biomcp help key

```text
Register and manage source credentials without showing values.

Usage: biomcp help key [OPTIONS] [source] [COMMAND]

Commands:
  list    List supported discovery topics.
  set
  remove

Arguments:
  [source]
          Credential source to describe.

          [possible values: ncbi, semantic-scholar, orcid, oncokb, alphagenome, nci, openfda]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Register and manage source credentials without showing values.

- `source`: Credential source to describe.

## biomcp help key list

```text
List supported discovery topics.

Usage: biomcp help key list [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

List supported discovery topics.


## biomcp help key set

```text
Usage: biomcp help key set [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help key set.


## biomcp help key remove

```text
Usage: biomcp help key remove [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help key remove.


## biomcp help serve

```text
Serve MCP over standard input or HTTP.

Usage: biomcp help serve [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Serve MCP over standard input or HTTP.


## biomcp help schema

```text
Usage: biomcp help schema [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help schema.


## biomcp help help

```text
Describe commands and their arguments.

Usage: biomcp help help [OPTIONS] [command]...

Arguments:
  [command]...
          Command help path to describe.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe commands and their arguments.

- `command`: Command help path to describe.

## biomcp help network

```text
Usage: biomcp help network [OPTIONS] [setting]

Arguments:
  [setting]
          Network setting to describe.

          [possible values: ca_bundle, proxy]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help network.

- `setting`: Network setting to describe.

## biomcp help sources

```text
Usage: biomcp help sources [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help sources.


## biomcp help config

```text
Usage: biomcp help config [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Describe help config.


## biomcp help exit

```text
Explain process exit and error codes.

Usage: biomcp help exit [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Process exit codes:
0: complete or partial reply
1: no matching record
2: invalid command or arguments
3: expired result pool
4: source request failed
5: local configuration, storage or output failed
Error codes: usage.schema, usage.arguments, usage.mcp_unavailable, usage.unknown_command, usage.retired, local.failure, pool.expired, usage.pool_mismatch, usage.too_many_pmids, usage.too_broad, usage.unsupported_filter. Source problems and reasons appear in their reply fields.


## biomcp help relations

```text
Map entities through sections and search filters.

Usage: biomcp help relations [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values.

      --config <config>
          Read configuration from this file.

      --offline
          Read local data without source requests.

      --fresh
          Request fresh source data instead of cached replies.

```

Map entities through sections and search filters.


