# Command reference

Generated from the command grammar and schema help. Run `make docs-generate` to update.

Options shown on the root command apply to its subcommands. Query sections and source rules follow each command.

Listed choices accept any letter case; replies use their listed spelling.

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
  source      List source names as JSON. No source requests or key reads.
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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

-h, --help: Print scoped command help without source requests.
--version: Print the installed version without source requests.
Entity-first order also runs on the command line: gene get BRAF runs get gene BRAF.

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
source: List source names as JSON
version: Print version; --verbose adds build, schema and path provenance
mcp-config: Print MCP configuration with this executable’s absolute path
health: Report source terms and bounded reachability.
store: Manage local cached replies and snapshots.
key: Register and manage source credentials without showing values.
serve: Serve MCP over standard input or HTTP.
schema: List every schema identity and description with biomcp schema, or print one schema artifact with biomcp schema <id>.
help: Describe commands and their arguments.

- `--format`: Choose JSON or Markdown output; configuration can change the default.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --format json. Local command option; independent of source selection.
- `--disable-sources`: Exclude source names or source terms groups. Example: --disable-sources civic. Value form: string. Local command option; independent of source selection.
- `--source_pacing`: Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'. Value form: string. Local command option; independent of source selection.
- `--config`: Read configuration from this file. Example: --config config.json. Value form: string. Local command option; independent of source selection.
- `--offline`: Read local data without source requests. Value form: boolean. Example: --offline. Local command option; independent of source selection.
- `--fresh`: Request fresh source data instead of cached replies. Value form: boolean. Example: --fresh. Local command option; independent of source selection.
- `--help`: Print scoped command help without source requests. Value form: boolean. Example: --help. Local command option; independent of source selection.
- `--version`: Print the installed version without source requests. Value form: boolean. Example: --version. Local command option; independent of source selection.

## biomcp search

```text
Search source records by entity and checked filters.

Usage: biomcp search [OPTIONS] <COMMAND>

Commands:
  biomarker      Find NCI biomarker concepts by name or synonym. The public tier returns codes, name, type, trial count, synonyms, parent_ids, ancestor_ids and eligibility_criterion. These concepts differ from diagnostic --biomarker filters and literal trial --variant text. Use search trial --source nci --biomarker-code CODE for trials that list an exact concept. Native criteria do not establish patient eligibility.
  author         Literal author name query and native window; positional text maps to query.
  organization   Find NCI organizations. Names are not unique; trial counts cover every status, including closed trials. No organization ID or get is available.
  cell-line      One native cell line search with optional text, species, disease, variation and derived-line filters.
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
  drug           Search one drug source. FDA drug recalls accept a name or native firm, classification, status, reason and initiation dates; counts are recall records. WHO prequalification accepts a name or date range. openFDA labels also accept literal indication wording. Other sources require a name.
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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --name <name>
          Native partial name or synonym search.

      --offset <offset>
          Offset within the complete retained observation.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --pool <pool>
          Snapshot identity retained by continuation commands.

      --source <source>
          Explicit keyed NCI source; help search biomarker
          Explicit keyed NCI source.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Search NCI biomarker concepts by name or synonym. Sources: nci.

- `--limit`: Maximum headers per local page. Value form: integer. Example: --limit 5. Command sources: nci; source restrictions in the field description apply.
- `--name`: Source partial name or synonym search. Value form: string. Example: --name PD-L1. Command sources: nci; source restrictions in the field description apply.
- `--offset`: Offset within the complete retained observation. Value form: integer. Example: --offset 0. Command sources: nci; source restrictions in the field description apply.
- `--pool`: Snapshot identity retained by continuation commands. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: nci; source restrictions in the field description apply.
- `--source`: Explicit keyed NCI source; help search biomarker
Explicit keyed NCI source.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source nci. Command sources: nci; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Native offset.

      --pool <pool>
          Retained page identity.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --source <source>
          Source; auto uses PubMed if Semantic Scholar is off; help search author
          Source; auto uses PubMed if Semantic Scholar is off.
          PubMed accepts names or topics and returns unresolved names on papers.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Search research paper authors. Sources: pubmed, semantic-scholar.

- `--limit`: Bounded source page size. Value form: integer. Example: --limit 5. Command sources: pubmed, semantic-scholar; source restrictions in the field description apply.
- `--offset`: Source offset. Value form: integer. Example: --offset 0. Command sources: pubmed, semantic-scholar; source restrictions in the field description apply.
- `--pool`: Retained page identity. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: pubmed, semantic-scholar; source restrictions in the field description apply.
- `query`: Exact query from the positional argument.
- `--source`: Source; auto uses PubMed if Semantic Scholar is off; help search author
Source; auto uses PubMed if Semantic Scholar is off.
PubMed accepts names or topics and returns unresolved names on papers.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source semanticscholar. Command sources: pubmed, semantic-scholar; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --distance <distance>
          Positive radius with explicit mi or km units.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

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

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --offset <offset>
          Offset within the complete retained organization snapshot.

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

      --pool <pool>
          Snapshot identity retained by continuation commands.

      --postal-code <postal_code>
          Native postal code.

      --source <source>
          Explicit keyed NCI source; help search organization
          Explicit keyed NCI source.

      --state <state>
          Native state or province.

```

Search clinical research organizations. Sources: nci.

- `--city`: Source organization city. Value form: string. Example: --city Boston. Command sources: nci; source restrictions in the field description apply.
- `--country`: Source organization country. Value form: string. Example: --country 'United States'. Command sources: nci; source restrictions in the field description apply.
- `--distance`: Positive radius with explicit mi or km units. Value form: string. Example: --distance 10mi. Command sources: nci; source restrictions in the field description apply.
- `--family`: Source organization family. Value form: string. Example: --family 'Dana-Farber/Harvard Cancer Center'. Command sources: nci; source restrictions in the field description apply.
- `--lat`: Radius center latitude in degrees; requires lon and distance. Value form: string. Example: --lat 42.3469. Command sources: nci; source restrictions in the field description apply.
- `--limit`: Five headers by default, at most fifty. Value form: integer. Example: --limit 5. Command sources: nci; source restrictions in the field description apply.
- `--lon`: Radius center longitude in degrees; requires lat and distance. Value form: string. Example: --lon -71.1025. Command sources: nci; source restrictions in the field description apply.
- `--name`: NCI partial organization name; names are not unique. Value form: string. Example: --name Dana-Farber. Command sources: nci; source restrictions in the field description apply.
- `--offset`: Offset within the complete retained organization snapshot. Value form: integer. Example: --offset 0. Command sources: nci; source restrictions in the field description apply.
- `--pool`: Snapshot identity retained by continuation commands. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: nci; source restrictions in the field description apply.
- `--postal-code`: Source postal code. Value form: string. Example: --postal-code 02215. Command sources: nci; source restrictions in the field description apply.
- `--source`: Explicit keyed NCI source; help search organization
Explicit keyed NCI source.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source nci. Command sources: nci; source restrictions in the field description apply.
- `--state`: Source state or province. Value form: string. Example: --state MA. Command sources: nci; source restrictions in the field description apply.

## biomcp search cell-line

```text
One native cell line search with optional text, species, disease, variation and derived-line filters.

Usage: biomcp search cell-line [OPTIONS] [query]

Arguments:
  [query]
          Literal source query from the positional argument.

Options:
      --disease <disease>
          Cellosaurus disease: NCIt name or ID (for example Melanoma or C3224).

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --exclude-derived[=<exclude_derived>]
          Leave out lines derived from a parent line; needs another search term.

          [default: false]
          [possible values: true, false]

      --gene <gene>
          Cellosaurus variation gene, as the source writes it (for example KRAS).

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

  -l, --limit <limit>
          Native page size.

      --mutation <mutation>
          Cellosaurus variation text, as the source writes it (for example p.Gly12Asp).

      --offset <offset>
          Native start offset.

      --protein <protein>
          Protein change, sent in the source's spelling (G12D is sent as p.Gly12Asp).

      --species <species>
          Cellosaurus species: NCBI Taxonomy name or ID (for example Homo sapiens or 9606).

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Search cell lines by name or identifier. Sources: cellosaurus.

- `--disease`: Cellosaurus disease: NCIt name or ID (for example Melanoma or C3224). Value form: string. Example: --disease Melanoma. Command sources: cellosaurus; source restrictions in the field description apply.
- `--exclude-derived`: Leave out lines derived from a parent line; needs another search term. Value form: boolean. Example: --exclude-derived. Command sources: cellosaurus; source restrictions in the field description apply.
- `--gene`: Cellosaurus variation gene, as the source writes it (for example KRAS). Value form: string. Example: --gene KRAS. Command sources: cellosaurus; source restrictions in the field description apply.
- `--limit`: Source page size. Value form: integer. Example: --limit 25. Command sources: cellosaurus; source restrictions in the field description apply.
- `--mutation`: Cellosaurus variation text, as the source writes it (for example p.Gly12Asp). Value form: string. Example: --mutation p.Gly12Asp. Command sources: cellosaurus; source restrictions in the field description apply.
- `--offset`: Source start offset. Value form: integer. Example: --offset 0. Command sources: cellosaurus; source restrictions in the field description apply.
- `--protein`: Protein change, sent in the source's spelling (G12D is sent as p.Gly12Asp). Value form: string. Example: --protein G12D. Command sources: cellosaurus; source restrictions in the field description apply.
- `query`: Exact source query from the positional argument.
- `--species`: Cellosaurus species: NCBI Taxonomy name or ID (for example Homo sapiens or 9606). Value form: string. Example: --species 'Homo sapiens'. Command sources: cellosaurus; source restrictions in the field description apply.

## biomcp search gwas

```text
Source associations in default p-value-ascending order. Reports no personal risk, causal variant or gene, or clinical interpretation of a statistic.

Usage: biomcp search gwas [OPTIONS] [query]

Arguments:
  [query]
          Exact native mapped gene; legacy positional query aliases this field.

Options:
      --extended-geneset[=<extended_geneset>]
          Source extended gene set; requires gene.

          [default: false]
          [possible values: true, false]

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

  -g, --gene <gene>
          Exact native mapped gene; legacy positional query aliases this field.

      --include-child-traits[=<include_child_traits>]
          Source child-trait expansion; requires trait_id.

          [default: false]
          [possible values: true, false]

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

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

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --studies[=<studies>]
          Return study records for the selected publication instead of associations.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

      --study <study>
          Exact source study accession.

      --trait <trait>
          Literal native efo_trait label; never searches reported_trait prose.

      --trait-id <trait_id>
          Exact native ontology identity.

      --variant <variant>
          Exact source rsID.

```

Search genetic associations in GWAS Catalog. Sources: gwas.

- `--extended-geneset`: Source extended gene set; requires gene. Value form: boolean. Example: --extended-geneset. Command sources: gwas; source restrictions in the field description apply.
- `--gene`: Exact source mapped gene; legacy positional query aliases this field. Value form: string. Example: --gene FGFR2. Command sources: gwas; source restrictions in the field description apply.
- `--include-child-traits`: Source child-trait expansion; requires trait_id. Value form: boolean. Example: --include-child-traits. Command sources: gwas; source restrictions in the field description apply.
- `--limit`: Maximum association rows. Value form: integer. Example: --limit 3. Command sources: gwas; source restrictions in the field description apply.
- `--offset`: Source association offset. Value form: integer. Example: --offset 3. Command sources: gwas; source restrictions in the field description apply.
- `--p-value`: Inclusive positive probability in scientific notation. Requires the default p-value-ascending order; no sort option is available. The qualifying total stays unknown until the ordered walk reaches its end. Value form: string. Example: --p-value 4e-254. Command sources: gwas; source restrictions in the field description apply.
- `--pool`: Retained threshold walk identity; requires a threshold. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: gwas; source restrictions in the field description apply.
- `--pubmed`: Exact source publication identity. Value form: string. Example: --pubmed 17529967. Command sources: gwas; source restrictions in the field description apply.
- `query`: Exact source mapped gene; legacy positional query aliases this field.
- `--studies`: Return study records for the selected publication instead of associations. Value form: boolean. Example: --studies. Command sources: gwas; source restrictions in the field description apply.
- `--study`: Exact source study accession. Value form: string. Example: --study GCST000035. Command sources: gwas; source restrictions in the field description apply.
- `--trait`: Exact source efo_trait label; never searches reported_trait prose. Value form: string. Example: --trait 'breast carcinoma'. Command sources: gwas; source restrictions in the field description apply.
- `--trait-id`: Exact source ontology identity. Value form: string. Example: --trait-id MONDO_0004989. Command sources: gwas; source restrictions in the field description apply.
- `--variant`: Exact source rsID. Value form: string. Example: --variant rs2981582. Command sources: gwas; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --limit <limit>
          Five rows by default; no more than 25.

      --offset <offset>
          Source/retained result offset.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --pool <pool>
          Exact retained collection identity.

      --reviewed[=<reviewed>]
          Reviewed human scope stays enabled; unreviewed search is deferred.

          [default: false]
          [possible values: true, false]

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Search reviewed human proteins in UniProt. Sources: uniprot.

- `--gene`: Exact source gene field, without identifier inference. Value form: string. Example: --gene BRAF. Command sources: uniprot; source restrictions in the field description apply.
- `--limit`: Five rows by default; no more than 25. Value form: integer. Example: --limit 25. Command sources: uniprot; source restrictions in the field description apply.
- `--offset`: Source/retained result offset. Value form: integer. Example: --offset 0. Command sources: uniprot; source restrictions in the field description apply.
- `--pool`: Exact retained collection identity. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: uniprot; source restrictions in the field description apply.
- `query`: Exact text, exclusive with gene.
- `--reviewed`: Reviewed human scope stays enabled; unreviewed search is deferred. Value form: boolean. Example: --reviewed. Command sources: uniprot; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Native source row offset.

      --source <source>
          Monarch only; HPO search is unsupported; help search phenotype
          Monarch only; HPO search is unsupported.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Search human phenotypes in HPO. Sources: hpo, monarch.

- `--limit`: Maximum source headers, default five. Value form: integer. Example: --limit 1. Command sources: hpo, monarch; source restrictions in the field description apply.
- `--offset`: Source source row offset. Value form: integer. Example: --offset 0. Command sources: hpo, monarch; source restrictions in the field description apply.
- `--source`: Monarch only; HPO search is unsupported; help search phenotype
Monarch only; HPO search is unsupported.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source monarch. Command sources: hpo, monarch; source restrictions in the field description apply.
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

      --fresh[=<fresh>]
          Request a new retained observation.

          [default: false]
          [possible values: true, false]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --gene <gene>
          Human gene input for exact resolution and UniProt mapping; Reactome only.

  -l, --limit <limit>
          Five rows by default, one to fifty. Reactome rows equals this limit without refill.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offset <offset>
          Native Reactome Start row or retained WikiPathways catalog offset.

      --pool <pool>
          Retained snapshot identity for deterministic continuation.

      --source <source>
          Sources in caller order; repeated sources take turns; help search pathway
          Sources in caller order; repeated sources take turns without title joins.

      --top-level[=<top_level>]
          Native human Reactome top-level discovery, exclusive with text and gene.

          [default: false]
          [possible values: true, false]

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

```

Search human biological pathways. Sources: reactome, wikipathways.

- `--fresh`: Request a new retained observation. Value form: boolean. Example: --fresh. Command sources: reactome, wikipathways; source restrictions in the field description apply.
- `--gene`: Human gene input for exact resolution and UniProt mapping; Reactome only. Value form: string. Example: --gene HGNC:1097. Command sources: reactome, wikipathways; source restrictions in the field description apply.
- `--limit`: Five rows by default, one to fifty. Reactome rows equals this limit without refill. Value form: integer. Example: --limit 5. Command sources: reactome, wikipathways; source restrictions in the field description apply.
- `--offset`: Source Reactome Start row or retained WikiPathways catalog offset. Value form: integer. Example: --offset 0. Command sources: reactome, wikipathways; source restrictions in the field description apply.
- `--pool`: Retained snapshot identity for deterministic continuation. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: reactome, wikipathways; source restrictions in the field description apply.
- `query`: Exact source query; exclusive with gene and top-level discovery.
- `--source`: Sources in caller order; repeated sources take turns; help search pathway
Sources in caller order; repeated sources take turns without title joins.
Listed choices accept any letter case; replies use their listed spelling. Value form: repeatable values (repeat the flag). Example: --source reactome. Command sources: reactome, wikipathways; source restrictions in the field description apply.
- `--top-level`: Source human Reactome top-level discovery, exclusive with text and gene. Value form: boolean. Example: --top-level. Command sources: reactome, wikipathways; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --limit <limit>
          Maximum headers per page.

      --offset <offset>
          Zero-based offset in the checked snapshot.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --pool <pool>
          Opaque continuation retaining the catalog revision and selectors.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Search cancer genomics studies in cBioPortal. Sources: cbioportal.

- `--cancer-type`: Exact cBioPortal cancer-type code; no OncoTree conversion. Value form: string. Example: --cancer-type mixed. Command sources: cbioportal; source restrictions in the field description apply.
- `--limit`: Maximum headers per page. Value form: integer. Example: --limit 1. Command sources: cbioportal; source restrictions in the field description apply.
- `--offset`: Zero-based offset in the checked snapshot. Value form: integer. Example: --offset 0. Command sources: cbioportal; source restrictions in the field description apply.
- `--pool`: Opaque continuation retaining the catalog revision and selectors. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: cbioportal; source restrictions in the field description apply.
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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --gene <gene>
          Explicit source gene symbol.

      --limit <limit>
          Rows per native page.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offset <offset>
          Native source offset.

      --source <source>
          Required CPIC source; ClinPGx is not supported for search; help search pgx
          Required CPIC source; ClinPGx is not supported for search.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Search pharmacogenomic gene and drug pairs. Sources: cpic.

- `--cpic-level`: Exact source level, including compound levels such as B/C. Value form: string. Example: --cpic-level B/C. Command sources: cpic; source restrictions in the field description apply.
- `--gene`: Explicit source gene symbol. Value form: string. Example: --gene DPYD. Command sources: cpic; source restrictions in the field description apply.
- `--limit`: Rows per source page. Value form: integer. Example: --limit 1. Command sources: cpic; source restrictions in the field description apply.
- `--offset`: Source source offset. Value form: integer. Example: --offset 0. Command sources: cpic; source restrictions in the field description apply.
- `--source`: Required CPIC source; ClinPGx is not supported for search; help search pgx
Required CPIC source; ClinPGx is not supported for search.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source cpic. Command sources: cpic; source restrictions in the field description apply.

## biomcp search diagnostic

```text
Search five source records by default. FDA device recalls cover every medical device; totals count recall records, without harm inference.

Usage: biomcp search diagnostic [OPTIONS] --source <source>

Options:
      --biomarker <biomarker>
          Exact table-0 cell.
          Exact table-0 biomarker cell; unsupported in the group table.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --bundle-revision <bundle_revision>
          Stored bundle revision.
          Immutable stored bundle revision retained by a continuation.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --condition-id <condition_id>
          Exact MedGen:CUI.
          Exact MedGen:CUI identifier only.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --decision-from <decision_from>
          First date, YYYY-MM-DD.
          Inclusive native decision date, YYYY-MM-DD; PMA and 510(k) only.

      --decision-to <decision_to>
          Last date, YYYY-MM-DD.
          Inclusive native decision date, YYYY-MM-DD; PMA and 510(k) only.

      --device-id <device_id>
          PMA or K identifier.
          Exact PMA application, application/supplement or native K identity.

      --drug-application <drug_application>
          NDA/BLA application.
          Cell-owned typed NDA/BLA application.

      --gene <gene>
          Exact gene symbol.
          Exact source gene_symbol field, without alias expansion.

      --gene-id <gene_id>
          Exact NCBI GeneID.
          Exact source GeneID, exclusive with gene symbol.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

      --initiated-from <initiated_from>
          First date, YYYY-MM-DD.
          Inclusive initiation date, YYYY-MM-DD; FDA device recalls only.

      --initiated-to <initiated_to>
          Last date, YYYY-MM-DD.
          Inclusive initiation date, YYYY-MM-DD; FDA device recalls only.

      --limit <limit>
          Five results by default; at most fifty source rows or product headers.

      --manufacturer <manufacturer>
          Manufacturer phrase.
          WHO manufacturer words, native FDA applicant (PMA/510(k)) or recalling_firm phrase.

      --marker <marker>
          WHO marker phrase.
          Literal whole-word phrase in WHO's Pathogen/Disease/Marker cell.

      --name <name>
          Native device name.
          Native trade_name (PMA) or device_name (510(k)) phrase.

      --offset <offset>
          Zero-based source result offset.

      --product-code <product_code>
          FDA product code.
          Exact native FDA product code; PMA, 510(k) and device recalls only.

      --source <source>
          Native diagnostic source; help search diagnostic
          Native diagnostic source.
          Explicit native GTR, WHO IVD, FDA PMA, 510(k), companion or medical device recall discovery.

      --type <type>
          Native test_type; help search diagnostic
          Native test_type.
          Exact native test_type value.

```

Search diagnostic laboratory tests and devices. Sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd.

- `--biomarker`: Exact table-0 cell.
Exact table-0 biomarker cell; unsupported in the group table. Value form: string. Example: --biomarker KRAS. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--bundle-revision`: Stored bundle revision.
Immutable stored bundle revision retained by a continuation. Value form: string. Example: --bundle-revision sha256:3eb93d3ba1a08b508955b0699d32042584fd8e0c805c72f7eeb470ef04ba959a. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--condition-id`: Exact MedGen:CUI.
Exact MedGen:CUI identifier only. Value form: string. Example: --condition-id MedGen:C2676676. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--decision-from`: First date, YYYY-MM-DD.
Inclusive source decision date, YYYY-MM-DD; PMA and 510(k) only. Value form: string. Example: --decision-from 2017-11-30. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--decision-to`: Last date, YYYY-MM-DD.
Inclusive source decision date, YYYY-MM-DD; PMA and 510(k) only. Value form: string. Example: --decision-to 2017-11-30. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--device-id`: PMA or K identifier.
Exact PMA application, application/supplement or source K identity. Value form: string. Example: --device-id P200010/S026. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--drug-application`: NDA/BLA application.
Cell-owned typed NDA/BLA application. Value form: string. Example: --drug-application BLA125514. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--gene`: Exact gene symbol.
Exact source gene_symbol field, without alias expansion. Value form: string. Example: --gene BRCA1. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--gene-id`: Exact NCBI GeneID.
Exact source GeneID, exclusive with gene symbol. Value form: string. Example: --gene-id 672. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--initiated-from`: First date, YYYY-MM-DD.
Inclusive initiation date, YYYY-MM-DD; FDA device recalls only. Value form: string. Example: --initiated-from 2024-01-01. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--initiated-to`: Last date, YYYY-MM-DD.
Inclusive initiation date, YYYY-MM-DD; FDA device recalls only. Value form: string. Example: --initiated-to 2024-12-31. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--limit`: Five results by default; at most fifty source rows or product headers. Value form: integer. Example: --limit 1. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--manufacturer`: Manufacturer phrase.
WHO manufacturer words, source FDA applicant (PMA/510(k)) or recalling_firm phrase. Value form: string. Example: --manufacturer 'Synthetic maker'. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--marker`: WHO marker phrase.
Exact whole-word phrase in WHO's Pathogen/Disease/Marker cell. Value form: string. Example: --marker hIv. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--name`: Source device name.
Source trade_name (PMA) or device_name (510(k)) phrase. Value form: string. Example: --name 'FoundationOne CDx'. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--offset`: Zero-based source result offset. Value form: integer. Example: --offset 5. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--product-code`: FDA product code.
Exact source FDA product code; PMA, 510(k) and device recalls only. Value form: string. Example: --product-code ABC. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--source`: Source diagnostic source; help search diagnostic
Source diagnostic source.
Explicit source GTR, WHO IVD, FDA PMA, 510(k), companion or medical device recall discovery.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source fda-device-recall. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--type`: Source test_type; help search diagnostic
Source test_type.
Exact source test_type value.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --type Research. Command sources: fda-510k, fda-cdx, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.

## biomcp search adverse-event

```text
Search five report headers by default. MAUDE accepts device text or a manufacturer, product-code or problem filter. One report may list several devices; filters may match different devices. VAERS accepts MMR, COVID19, COVID-19 vaccine or one exact qualified native influenza code, matched without case; use influenza to list codes and labels. Tables remain separate and are never summed. One report may list several vaccines. VAERS returns five reported symptom rows by default; seriousness and age select independent tables. Counts are reports, not people. Reports do not show that a vaccine caused an event and cannot measure risk or compare vaccine safety.

Usage: biomcp search adverse-event [OPTIONS] [text] [section]

Arguments:
  [text]
          Drug, device or vaccine.
          Literal drug text, MAUDE device phrase or exact supported VAERS vaccine.

  [section]
          VAERS table; help search adverse-event

Options:
      --aggregate <aggregate>
          FAERS reaction counts; help search adverse-event
          FAERS reaction counts.
          Use reactions for source-native count buckets across matching reports. Counts are not rates or causal associations; a report may contribute to several terms. Requires MedDRA and offset zero.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --drug <drug>...
          FAERS drug names; OR.
          FAERS literal drug name; repeat --drug for OR. A standalone OR joining names inside one --drug value is refused. Other filters AND with the names. BioMCP caps the escaped native query, including filters, at FAERS_QUERY_MAX_BYTES (4096 bytes).

      --limit <limit>
          Native report headers, counted FAERS reaction terms or VAERS rows; default five.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --manufacturer <manufacturer>
          MAUDE manufacturer.
          MAUDE manufacturer phrase on device.manufacturer_d_name. A report may list several devices; filters may match different devices.

      --offset <offset>
          Native report row offset; reaction counts require zero. VAERS needs its snapshot.

      --problem <problem>
          MAUDE product problem.
          MAUDE phrase on native product_problems.

      --product-code <product_code>...
          MAUDE product codes; OR.
          MAUDE exact native device.device_report_product_code; repeated codes join with OR.

      --reaction <reaction>
          FAERS MedDRA phrase.
          FAERS MedDRA term phrase match, not exact equality.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --received-from <received_from>
          First date, YYYY-MM[-DD].
          Inclusive FAERS or MAUDE FDA receipt lower bound, YYYY-MM-DD or YYYY-MM; a month starts on its first day.

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

      --received-to <received_to>
          Last date, YYYY-MM[-DD].
          Inclusive FAERS or MAUDE FDA receipt upper bound, YYYY-MM-DD or YYYY-MM; a month ends on its last day.

      --serious <serious>
          FAERS seriousness flag; help search adverse-event
          FAERS seriousness flag.
          FAERS report flag: any uses the source seriousness flag; the specific flags can overlap. Never a reaction outcome.

      --snapshot <snapshot>
          VAERS snapshot.
          Retained VAERS table for provider-free continuation.

      --source <source>
          Source; default FAERS; help search adverse-event
          Source; default FAERS.
          FAERS by default; VAERS accepts only exact supported vaccine names.

```

Search drug, device and vaccine adverse events. Sources: faers, maude, vaers.

- `--aggregate`: FAERS reaction counts; help search adverse-event
FAERS reaction counts.
Use reactions for source count buckets across matching reports. Counts are not rates or causal associations; a report may contribute to several terms. Requires MedDRA and offset zero.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --aggregate reactions. Command sources: faers, maude, vaers; source restrictions in the field description apply.
- `--drug`: FAERS drug names; OR.
FAERS exact drug name; repeat --drug for OR. A standalone OR joining names inside one --drug value is refused. Other filters AND with the names. BioMCP caps the escaped source query, including filters, at FAERS_QUERY_MAX_BYTES (4096 bytes). Value form: repeatable values (repeat the flag). Example: --drug "synthetic\\B". Command sources: faers, maude, vaers; source restrictions in the field description apply.
- `--limit`: Source report headers, counted FAERS reaction terms or VAERS rows; default five. Value form: integer. Example: --limit 1. Command sources: faers, maude, vaers; source restrictions in the field description apply.
- `--manufacturer`: MAUDE manufacturer.
MAUDE manufacturer phrase on device.manufacturer_d_name. A report may list several devices; filters may match different devices. Value form: string. Example: --manufacturer 'Synthetic maker'. Command sources: faers, maude, vaers; source restrictions in the field description apply.
- `--offset`: Source report row offset; reaction counts require zero. VAERS needs its snapshot. Value form: integer. Example: --offset 0. Command sources: faers, maude, vaers; source restrictions in the field description apply.
- `--problem`: MAUDE product problem.
MAUDE phrase on source product_problems. Value form: string. Example: --problem 'Synthetic problem'. Command sources: faers, maude, vaers; source restrictions in the field description apply.
- `--product-code`: MAUDE product codes; OR.
MAUDE exact source device.device_report_product_code; repeated codes join with OR. Value form: repeatable values (repeat the flag). Example: --product-code ABC. Command sources: faers, maude, vaers; source restrictions in the field description apply.
- `--reaction`: FAERS MedDRA phrase.
FAERS MedDRA term phrase match, not exact equality. Value form: string. Example: --reaction 'synthetic reaction'. Command sources: faers, maude, vaers; source restrictions in the field description apply.
- `--received-from`: First date, YYYY-MM[-DD].
Inclusive FAERS or MAUDE FDA receipt lower bound, YYYY-MM-DD or YYYY-MM; a month starts on its first day. Value form: string. Example: --received-from 2024-01-01. Command sources: faers, maude, vaers; source restrictions in the field description apply.
- `--received-to`: Last date, YYYY-MM[-DD].
Inclusive FAERS or MAUDE FDA receipt upper bound, YYYY-MM-DD or YYYY-MM; a month ends on its last day. Value form: string. Example: --received-to 2024-12-31. Command sources: faers, maude, vaers; source restrictions in the field description apply.
- `section`: VAERS table; help search adverse-event
Listed choices accept any letter case; replies use their listed spelling.
- `--serious`: FAERS seriousness flag; help search adverse-event
FAERS seriousness flag.
FAERS report flag: any uses the source seriousness flag; the specific flags can overlap. Never a reaction outcome.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --serious death. Command sources: faers, maude, vaers; source restrictions in the field description apply.
- `--snapshot`: VAERS snapshot.
Retained VAERS table for source-free continuation. Value form: string. Example: --snapshot <snapshot> (syntax; replace the placeholder with your retained value). Command sources: faers, maude, vaers; source restrictions in the field description apply.
- `--source`: Source; default FAERS; help search adverse-event
Source; default FAERS.
FAERS by default; VAERS accepts only exact supported vaccine names.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source maude. Command sources: faers, maude, vaers; source restrictions in the field description apply.
- `text`: Drug, device or vaccine.
Exact drug text, MAUDE device phrase or exact supported VAERS vaccine.

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
          Literal source search text; optional site-name filter for SEER.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --limit <limit>
          Maximum source rows on one page.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Native row offset.

      --ontology <ontology>
          Exactly one ontology, default MONDO; help search disease
          Exactly one ontology, default MONDO.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --source <source>
          One explicit source, default OLS; help search disease
          One explicit source, default OLS.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Search diseases and their identifiers. Sources: monarch, mydisease, ols, seer.

- `--limit`: Maximum source rows on one page. Value form: integer. Example: --limit 1. Command sources: monarch, mydisease, ols, seer; source restrictions in the field description apply.
- `--offset`: Source row offset. Value form: integer. Example: --offset 1. Command sources: monarch, mydisease, ols, seer; source restrictions in the field description apply.
- `--ontology`: Exactly one ontology, default MONDO; help search disease
Exactly one ontology, default MONDO.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --ontology mondo. Command sources: monarch, mydisease, ols, seer; source restrictions in the field description apply.
- `--source`: One explicit source, default OLS; help search disease
One explicit source, default OLS.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source seer. Command sources: monarch, mydisease, ols, seer; source restrictions in the field description apply.
- `text`: Exact source search text; optional site-name filter for SEER.

## biomcp search drug

```text
Search one drug source. FDA drug recalls accept a name or native firm, classification, status, reason and initiation dates; counts are recall records. WHO prequalification accepts a name or date range. openFDA labels also accept literal indication wording. Other sources require a name.

Usage: biomcp search drug [OPTIONS] [query]

Arguments:
  [query]
          Name phrase; optional with supported source filters.
          FDA drug recalls, WHO date ranges and label indication searches allow no name.
          Drugs@FDA searches brands and ingredients,
          exact names before word matches. openFDA labels search exact then word brand/generic names,
          then label text (including indications) when no names match; source matches need not be exact names.
          Harmonized recall names match but never become display names.

Options:
      --classification <classification>
          Native FDA classification; drug recalls only; help search drug
          Native FDA classification; drug recalls only.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --firm <firm>
          Literal published recalling firm phrase; drug recalls only.

      --indication <indication>
          Literal label indication; openFDA labels only.
          Search the native Indications and Usage field without disease synonym expansion.
          An optional name query further narrows the same source search.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --initiated-from <initiated_from>
          Inclusive initiation day, YYYY-MM-DD; drug recalls only.

      --initiated-to <initiated_to>
          Inclusive initiation day, YYYY-MM-DD; drug recalls only.

      --limit <limit>
          Native candidate rows per page.

      --name-selector <name_selector>
          Explicit ChEMBL native name selector; omitted means preferred; help search drug
          Explicit ChEMBL native name selector; omitted means preferred name.

      --offset <offset>
          Native rows to skip.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --prequalified-from <prequalified_from>
          Inclusive WHO prequalification lower bound, YYYY, YYYY-MM or YYYY-MM-DD; WHO PQ only.

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

      --prequalified-to <prequalified_to>
          Inclusive WHO prequalification upper bound, YYYY, YYYY-MM or YYYY-MM-DD; WHO PQ only.

      --reason <reason>
          Phrase in the published recall reason; drug recalls only.

      --source <source>
          Selected source; help search drug
          Selected source.

          After an empty name lookup, existing sources may supply aliases.

      --status <status>
          Literal native recall status; drug recalls only.

```

Search drugs and medicines by name or source label indication. Sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq.

- `--classification`: Source FDA classification; drug recalls only; help search drug
Source FDA classification; drug recalls only.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --classification 'Class II'. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--firm`: Exact published recalling firm phrase; drug recalls only. Value form: string. Example: --firm Apotex. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--indication`: Exact label indication; openFDA labels only.
Search the source Indications and Usage field without disease synonym expansion.
An optional name query further narrows the same source search. Value form: string. Example: --indication melanoma. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--initiated-from`: Inclusive initiation day, YYYY-MM-DD; drug recalls only. Value form: string. Example: --initiated-from 2024-01-01. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--initiated-to`: Inclusive initiation day, YYYY-MM-DD; drug recalls only. Value form: string. Example: --initiated-to 2024-12-31. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--limit`: Source candidate rows per page. Value form: integer. Example: --limit 1. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--name-selector`: Explicit ChEMBL source name selector; omitted means preferred; help search drug
Explicit ChEMBL source name selector; omitted means preferred name.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --name-selector preferred. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--offset`: Source rows to skip. Value form: integer. Example: --offset 1. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--prequalified-from`: Inclusive WHO prequalification lower bound, YYYY, YYYY-MM or YYYY-MM-DD; WHO PQ only. Value form: string. Example: --prequalified-from 2020. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--prequalified-to`: Inclusive WHO prequalification upper bound, YYYY, YYYY-MM or YYYY-MM-DD; WHO PQ only. Value form: string. Example: --prequalified-to 2020. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `query`: Name phrase; optional with supported source filters.
FDA drug recalls, WHO date ranges and label indication searches allow no name.
Drugs@FDA searches brands and ingredients,
exact names before word matches. openFDA labels search exact then word brand/generic names,
then label text (including indications) when no names match; source matches need not be exact names.
Harmonized recall names match but never become display names.
- `--reason`: Phrase in the published recall reason; drug recalls only. Value form: string. Example: --reason 'Lack of Sterility Assurance'. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--source`: Selected source; help search drug
Selected source.

After an empty name lookup, existing sources may supply aliases.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source ema. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--status`: Exact source recall status; drug recalls only. Value form: string. Example: --status Ongoing. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --limit <limit>
          Number of results per page.

      --locus-group <locus_group>
          Optional HGNC locus group intersected with the selector; help search gene
          Optional HGNC locus group intersected with the selector.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offset <offset>
          Number of results to skip.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Search human genes by name or identifier. Sources: hgnc.

- `--band`: A cytoband prefix. Value form: string. Example: --band 9p21.3. Command sources: hgnc; source restrictions in the field description apply.
- `--limit`: Number of results per page. Value form: integer. Example: --limit 50. Command sources: hgnc; source restrictions in the field description apply.
- `--locus-group`: Optional HGNC locus group intersected with the selector; help search gene
Optional HGNC locus group intersected with the selector.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --locus-group pseudogene. Command sources: hgnc; source restrictions in the field description apply.
- `--offset`: Number of results to skip. Value form: integer. Example: --offset 24. Command sources: hgnc; source restrictions in the field description apply.
- `query`: Words matched against symbols, previous symbols, aliases and names.

## biomcp search article

```text
Checked search selectors and pool cursor.

Usage: biomcp search article [OPTIONS] [text]

Arguments:
  [text]
          Source search words.
          Words sent to each named source in its default fields; unsupported by LitVar, which requires a variant selector.

Options:
      --author <author>
          PubMed/Europe PMC author words.
          PubMed and Europe PMC only: author words sent to the source's author field.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --disease <disease>
          Disease words.
          Available for command sources except LitVar.

      --drug <drug>
          Drug words.
          Available for command sources except LitVar.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --exact-term[=<exact_term>]
          Search the typed drug term without resolving its other names.

          [default: false]
          [possible values: true, false]

      --gene <gene>
          Gene symbol or ID.
          Gene input; PubMed resolves it through HGNC.

      --journal <journal>
          PubMed/Europe PMC journal words.
          PubMed and Europe PMC only: journal words sent to the source's journal field.

      --limit <limit>
          Maximum page length.

      --no-preprints[=<no_preprints>]
          Exclude preprints.
          PubMed and Europe PMC only: exclude preprints using the source's native filter.

          [default: false]
          [possible values: true, false]

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --offset <offset>
          Number of pool items to skip.

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

      --open-access[=<open_access>]
          Native open access.
          PubMed, Europe PMC and Semantic Scholar only: require native open access; Semantic Scholar means a public PDF.

          [default: false]
          [possible values: true, false]

      --pool <pool>
          Stable result snapshot.
          Stable snapshot from an earlier response.

      --exclude-retracted


      --include-retracted


      --since <since>
          First publication date.
          PubMed, Europe PMC and Semantic Scholar only: inclusive lower publication-date bound; PubMed matches electronic or print date.

      --sort <sort>
          rank, date or citations; help search article
          rank, date or citations.
          rank (default): native provider order; several sources take turns; native search cost.
          date: native search cost; PubMed means date added and Europe PMC means newest first; other sources refuse it.
          citations: most cited first across the whole pool; adds iCite calls, about 1 s per 1,000 PMIDs.
          All sources support citations.

      --source <source>
          Sources; default pubmed; help search article
          Sources; default pubmed.
          Sources to search in caller order, comma separated. Default: pubmed.
          Several sources take turns, remove duplicates and show every native rank.
          pubmed: keywords with PubMed MeSH mapping; dropped words appear as notices.
          europepmc: words in open full text and preprints; incidental mentions are possible.
          litvar: exact variant publications; requires --variant and a protein change needs --gene; unresolved inputs return candidates.
          semantic-scholar: broad discovery; a shared keyless pool may return 429; an optional free key raises the limit.
          litsense: sentence resemblance, not a match; unrelated hits and timeouts are possible.
          pubtator: entity and word search including full text; needs the right entity ID; --gene uses HGNC Entrez identity and cannot run without that join.

      --type <type>
          Publication type; help search article
          Publication type.
          PubMed and Europe PMC only: native publication-type filter.

      --until <until>
          Last publication date.
          PubMed, Europe PMC and Semantic Scholar only: inclusive upper publication-date bound; PubMed matches electronic or print date.

      --variant <variant>
          Variant words.

  -q, --query <query_alias>
          Alias for positional article text.

```

Search research articles and biomedical papers. Sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar.

- `--author`: PubMed/Europe PMC author words.
PubMed and Europe PMC only: author words sent to the source's author field. Value form: string. Example: --author Carpten. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--disease`: Disease words.
Available for command sources except LitVar. Value form: string. Example: --disease meningioma. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--drug`: Drug words.
Available for command sources except LitVar. Value form: string. Example: --drug 'drug words'. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--exact-term`: Search the typed drug term without resolving its other names. Value form: boolean. Example: --exact-term. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--gene`: Gene symbol or ID.
Gene input; PubMed resolves it through HGNC. Value form: string. Example: --gene NF2. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--journal`: PubMed/Europe PMC journal words.
PubMed and Europe PMC only: journal words sent to the source's journal field. Value form: string. Example: --journal 'journal words'. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--limit`: Maximum page length. Value form: integer. Example: --limit 5. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--no-preprints`: Exclude preprints.
PubMed and Europe PMC only: exclude preprints using the source's source filter. Value form: boolean. Example: --no-preprints. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--offset`: Number of pool items to skip. Value form: integer. Example: --offset 5. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--open-access`: Source open access.
PubMed, Europe PMC and Semantic Scholar only: require source open access; Semantic Scholar means a public PDF. Value form: boolean. Example: --open-access. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--pool`: Stable result snapshot.
Stable snapshot from an earlier response. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--exclude-retracted`: Retraction filter; help search article
Retraction filter.
Include retracted papers by default. Exclusion is supported only by PubMed and Europe PMC. Value form: boolean. Example: --exclude-retracted. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--include-retracted`: Retraction filter; help search article
Retraction filter.
Include retracted papers by default. Exclusion is supported only by PubMed and Europe PMC. Value form: boolean. Example: --include-retracted. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--since`: First publication date.
PubMed, Europe PMC and Semantic Scholar only: inclusive lower publication-date bound; PubMed matches electronic or print date. Value form: string. Example: --since 2024. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--sort`: rank, date or citations; help search article
rank, date or citations.
rank (default): source order; several sources take turns; source search cost.
date: source search cost; PubMed means date added and Europe PMC means newest first; other sources refuse it.
citations: most cited first across the whole pool; adds iCite calls, about 1 s per 1,000 PMIDs.
All sources support citations.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --sort date. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--source`: Sources; default pubmed; help search article
Sources; default pubmed.
Sources to search in caller order, comma separated. Default: pubmed.
Several sources take turns, remove duplicates and show every source rank.
pubmed: keywords with PubMed MeSH mapping; dropped words appear as notices.
europepmc: words in open full text and preprints; incidental mentions are possible.
litvar: exact variant publications; requires --variant and a protein change needs --gene; unresolved inputs return candidates.
semantic-scholar: broad discovery; a shared keyless pool may return 429; an optional free key raises the limit.
litsense: sentence resemblance, not a match; unrelated hits and timeouts are possible.
pubtator: entity and word search including full text; needs the right entity ID; --gene uses HGNC Entrez identity and cannot run without that join.
Listed choices accept any letter case; replies use their listed spelling. Value form: repeatable values (repeat the flag). Example: --source europepmc. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `text`: Source search words.
Words sent to each named source in its default fields; unsupported by LitVar, which requires a variant selector.
- `--type`: Publication type; help search article
Publication type.
PubMed and Europe PMC only: source publication-type filter.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --type review. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--until`: Last publication date.
PubMed, Europe PMC and Semantic Scholar only: inclusive upper publication-date bound; PubMed matches electronic or print date. Value form: string. Example: --until 2020. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--variant`: Variant words. Value form: string. Example: --variant rs121434592. Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.
- `--query`: Alias for positional article text. Value form: string. Example: --query <query> (syntax; replace the placeholder with your retained value). Command sources: europepmc, litsense, litvar, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.

## biomcp search trial

```text
Search ClinicalTrials.gov or keyed NCI with native counts. NCI --facility matches exact organization names; a listed site is not necessarily enrolling.

Usage: biomcp search trial [OPTIONS] [query]

Arguments:
  [query]
          Trial text.
          General registry discovery text.

Options:
      --age <age>
          Age in years.
          ClinicalTrials.gov only; unstated source bounds count as no limit.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --biomarker-code <biomarker_code>
          NCI NCIt code.
          Exact NCIt biomarker concept listed by NCI in trial criteria; NCI source only. This listing does not establish biomarker positivity or patient eligibility.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --biomarker-use <biomarker_use>
          NCI criterion; help search trial
          NCI criterion.
          Filter by NCI's native inclusion or exclusion criterion. Check each result's biomarker entry and reported problems.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --condition <condition>...
          Disease; NCI needs NCIt.
          Conditions joined with OR. NCI needs NCIt codes such as C3224; use --condition-id for MONDO or DOID identifiers.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --condition-id <condition_id>
          MONDO/DOID/NCIT:C.
          Exact MONDO or DOID CURIE resolved by MyDisease; NCI also accepts explicit NCIT:C codes.

      --count-only[=<count_only>]
          Total.
          Source total only; no returned study rows.

          [default: false]
          [possible values: true, false]

      --country <country>...
          Country label; help search trial
          Country label.
          Exact ClinicalTrials.gov country label; repeat for OR. Trials must list a site in a named country. NCI uses its native country filter.

      --criteria <criteria>
          Criteria text.
          ClinicalTrials.gov only: search exact registry eligibility text for discovery; no eligibility judgment.

      --distance <distance>
          Radius, mi or km.
          ClinicalTrials.gov only: source distance with explicit mi or km units.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --exact-condition[=<exact_condition>]
          Exact condition.
          Ask ClinicalTrials.gov to restrict matching to the registered condition phrase; its support is partial.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

      --exact-term[=<exact_term>]
          Search typed treatment terms without resolving their other names.

          [default: false]
          [possible values: true, false]

      --facility <facility>
          Facility.
          Facility phrase; ClinicalTrials.gov broadens after zero exact matches. NCI matches exact names. Cannot combine with radius.

      --gene <gene>
          Gene symbol/ID.
          ClinicalTrials.gov only: resolve one checked gene for registry text discovery; no eligibility judgment.

      --has-results[=<has_results>]
          Posted results.
          ClinicalTrials.gov only: require posted source results.

          [default: false]
          [possible values: true, false]

      --id <id>
          NCT identifier.
          ClinicalTrials.gov only: exact NCT identity in a conjunction with other filters.

      --intervention <intervention>...
          Treatment terms; OR.
          Intervention alternatives joined with OR; supported by ClinicalTrials.gov.

      --keyword <keyword>...
          Text; OR.
          Repeat for ClinicalTrials.gov general text alternatives joined with OR; different filters combine with AND. NCI accepts one keyword.

      --lat <lat>
          Latitude degrees.
          ClinicalTrials.gov only: radius center latitude in degrees.

      --limit <limit>
          Page size.
          Headers per page; NCI applies its own lower publisher limit.

      --lon <lon>
          Longitude degrees.
          ClinicalTrials.gov only: radius center longitude in degrees.

      --next-page <next_page>
          Page token.
          Opaque registry continuation; keep filters unchanged.

      --offset <offset>
          Page offset.
          Zero-based offset of the first row in this continuation page.

      --phase <phase>
          Native phase; help search trial
          Native phase.
          Exact source phase. ClinicalTrials.gov: 1, 2, 3, 4, early-1, not-applicable, 1/2, 2/3. NCI: NA, 0, I, I_II, II, II_III, III, IV.

      --sex <sex>
          ClinicalTrials.gov only: registry sex facet; help search trial
          ClinicalTrials.gov only: registry sex facet.

      --source <source>
          clinicaltrials/nci; help search trial
          clinicaltrials/nci.
          Explicit provider; defaults to ClinicalTrials.gov and never falls back.

      --sponsor <sponsor>
          Sponsor.
          ClinicalTrials.gov only: sponsor name; broadens after zero exact matches. Use --sponsor-type for the funding category.

      --sponsor-type <sponsor_type>
          Funding class; help search trial
          Funding class.
          ClinicalTrials.gov only: registry sponsor funding category; use --sponsor for a sponsor name.

      --status <status>
          Native status; help search trial
          Native status.
          Exact source status. NCI: Active, Administratively Complete, Approved, Closed to Accrual, Closed to Accrual and Intervention, Complete, INACTIVE, Enrolling by Invitation, Temporarily Closed to Accrual, Temporarily Closed to Accrual and Intervention, Withdrawn. ClinicalTrials.gov uses registry labels.

      --study-type <study_type>
          Study type; help search trial
          Study type.
          ClinicalTrials.gov only: exact registry study type.

      --updated-since <updated_since>
          First date.
          ClinicalTrials.gov only: first included registry update date.

      --updated-until <updated_until>
          Last date.
          ClinicalTrials.gov only: last included registry update date.

      --variant <variant>
          Variant text.
          ClinicalTrials.gov only: literal variant text for registry discovery; no eligibility judgment.

```

Search clinical trials. Sources: ctgov, nci.

- `--age`: Age in years.
ClinicalTrials.gov only; unstated source bounds count as no limit. Value form: integer. Example: --age 18. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--biomarker-code`: NCI NCIt code.
Exact NCIt biomarker concept listed by NCI in trial criteria; NCI source only. This listing does not establish biomarker positivity or patient eligibility. Value form: string. Example: --biomarker-code C96022. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--biomarker-use`: NCI criterion; help search trial
NCI criterion.
Filter by NCI's source inclusion or exclusion criterion. Check each result's biomarker entry and reported problems.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --biomarker-use inclusion. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--condition`: Disease; NCI needs NCIt.
Conditions joined with OR. NCI needs NCIt codes such as C3224; use --condition-id for MONDO or DOID identifiers. Value form: repeatable values (repeat the flag). Example: --condition melanoma. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--condition-id`: MONDO/DOID/NCIT:C.
Exact MONDO or DOID CURIE resolved by MyDisease; NCI also accepts explicit NCIT:C codes. Value form: string. Example: --condition-id NCIT:C3224. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--count-only`: Total.
Source total only; no returned study rows. Value form: boolean. Example: --count-only. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--country`: Country label; help search trial
Country label.
Exact ClinicalTrials.gov country label; repeat for OR. Trials must list a site in a named country. NCI uses its source country filter.
Listed choices accept any letter case; replies use their listed spelling. Value form: repeatable values (repeat the flag). Example: --country 'United States'. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--criteria`: Criteria text.
ClinicalTrials.gov only: search exact registry eligibility text for discovery; no eligibility judgment. Value form: string. Example: --criteria 'exact criteria'. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--distance`: Radius, mi or km.
ClinicalTrials.gov only: source distance with explicit mi or km units. Value form: string. Example: --distance 50mi. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--exact-condition`: Exact condition.
Ask ClinicalTrials.gov to restrict matching to the registered condition phrase; its support is partial. Value form: boolean. Example: --exact-condition. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--exact-term`: Search typed treatment terms without resolving their other names. Value form: boolean. Example: --exact-term. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--facility`: Facility.
Facility phrase; ClinicalTrials.gov broadens after zero exact matches. NCI matches exact names. Cannot combine with radius. Value form: string. Example: --facility 'Dana-Farber Cancer Institute'. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--gene`: Gene symbol/ID.
ClinicalTrials.gov only: resolve one checked gene for registry text discovery; no eligibility judgment. Value form: string. Example: --gene BRAF. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--has-results`: Posted results.
ClinicalTrials.gov only: require posted source results. Value form: boolean. Example: --has-results. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--id`: NCT identifier.
ClinicalTrials.gov only: exact NCT identity in a conjunction with other filters. Value form: string. Example: --id NCT05273554. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--intervention`: Treatment terms; OR.
Intervention alternatives joined with OR; supported by ClinicalTrials.gov. Value form: repeatable values (repeat the flag). Example: --intervention pembrolizumab. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--keyword`: Text; OR.
Repeat for ClinicalTrials.gov general text alternatives joined with OR; different filters combine with AND. NCI accepts one keyword. Value form: repeatable values (repeat the flag). Example: --keyword survival. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--lat`: Latitude degrees.
ClinicalTrials.gov only: radius center latitude in degrees. Value form: string. Example: --lat 42.3601. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--limit`: Page size.
Headers per page; NCI applies its own lower publisher limit. Value form: integer. Example: --limit 1. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--lon`: Longitude degrees.
ClinicalTrials.gov only: radius center longitude in degrees. Value form: string. Example: --lon -71.0589. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--next-page`: Page token.
Opaque registry continuation; keep filters unchanged. Value form: string. Example: --next-page <next-page> (syntax; replace the placeholder with your retained value). Command sources: ctgov, nci; source restrictions in the field description apply.
- `--offset`: Page offset.
Zero-based offset of the first row in this continuation page. Value form: integer. Example: --offset 0. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--phase`: Source phase; help search trial
Source phase.
Exact source phase. ClinicalTrials.gov: 1, 2, 3, 4, early-1, not-applicable, 1/2, 2/3. NCI: NA, 0, I, I_II, II, II_III, III, IV.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --phase 3. Command sources: ctgov, nci; source restrictions in the field description apply.
- `query`: Trial text.
General registry discovery text.
- `--sex`: ClinicalTrials.gov only: registry sex facet; help search trial
ClinicalTrials.gov only: registry sex facet.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --sex ALL. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--source`: clinicaltrials/nci; help search trial
clinicaltrials/nci.
Explicit source; defaults to ClinicalTrials.gov and never falls back.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source nci. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--sponsor`: Sponsor.
ClinicalTrials.gov only: sponsor name; broadens after zero exact matches. Use --sponsor-type for the funding category. Value form: string. Example: --sponsor 'MD Anderson'. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--sponsor-type`: Funding class; help search trial
Funding class.
ClinicalTrials.gov only: registry sponsor funding category; use --sponsor for a sponsor name.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --sponsor-type other. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--status`: Source status; help search trial
Source status.
Exact source status. NCI: Active, Administratively Complete, Approved, Closed to Accrual, Closed to Accrual and Intervention, Complete, INACTIVE, Enrolling by Invitation, Temporarily Closed to Accrual, Temporarily Closed to Accrual and Intervention, Withdrawn. ClinicalTrials.gov uses registry labels.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --status RECRUITING. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--study-type`: Study type; help search trial
Study type.
ClinicalTrials.gov only: exact registry study type.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --study-type INTERVENTIONAL. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--updated-since`: First date.
ClinicalTrials.gov only: first included registry update date. Value form: string. Example: --updated-since <updated-since> (syntax; replace the placeholder with your retained value). Command sources: ctgov, nci; source restrictions in the field description apply.
- `--updated-until`: Last date.
ClinicalTrials.gov only: last included registry update date. Value form: string. Example: --updated-until <updated-until> (syntax; replace the placeholder with your retained value). Command sources: ctgov, nci; source restrictions in the field description apply.
- `--variant`: Variant text.
ClinicalTrials.gov only: exact variant text for registry discovery; no eligibility judgment. Value form: string. Example: --variant L576P. Command sources: ctgov, nci; source restrictions in the field description apply.

## biomcp search variant

```text
Discover source candidates using one declared selector, without interpreting free text.

Usage: biomcp search variant [OPTIONS]

Options:
      --build <build>
          Optional caller assembly sent to MyVariant; regions; help search variant
          Optional caller assembly sent to MyVariant; regions require it explicitly.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --coding <coding>
          Coding HGVS change searched literally in ClinVar with --gene, without choosing a transcript.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --condition <condition>
          Literal MyVariant clinvar.rcv.conditions.name phrase; source text is not a local diagnosis. Requires a declared protein or region selector.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --gene <gene>
          Checked gene symbol for the dbNSFP substitution or ClinVar coding/synonymous selector.

      --has <has>
          Require the selected component's identifier field. Only; help search variant
          Require the selected component's identifier field. Only source-proven mappings are offered.

      --limit <limit>
          Native candidates per page, default 10, maximum 50; all candidates remain candidates.

      --offset <offset>
          Native offset within this command's bounded result window; larger native totals remain visible.

      --protein <protein>
          Explicit substitution or synonymous change using checked amino-acid codes.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --query <query>
          Raw MyVariant query, passed and echoed unchanged when component policy allows; requires --source myvariant. With disabled components use declared selectors instead.

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

      --region <region>
          Exact one-based inclusive MyVariant region, such as chr7:55259500-55259600; requires --build.

      --review-status <review_status>
          Literal MyVariant clinvar.rcv.review_status phrase; no star conversion. Requires a declared protein or region selector.

      --source <source>
          Explicit source language; required for a raw query; help search variant
          Explicit source language; required for a raw query.

```

Search genetic variants by gene, change or region. Sources: myvariant.

- `--build`: Optional caller assembly sent to MyVariant; regions; help search variant
Optional caller assembly sent to MyVariant; regions require it explicitly.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --build GRCh37. Command sources: myvariant; source restrictions in the field description apply.
- `--coding`: Coding HGVS change searched literally in ClinVar with --gene, without choosing a transcript. Value form: string. Example: --coding 'c.333G>T'. Command sources: myvariant; source restrictions in the field description apply.
- `--condition`: Exact MyVariant clinvar.rcv.conditions.name phrase; source text is not a local diagnosis. Requires a declared protein or region selector. Value form: string. Example: --condition 'Carcinoma of colon (CRC)'. Command sources: myvariant; source restrictions in the field description apply.
- `--gene`: Checked gene symbol for the dbNSFP substitution or ClinVar coding/synonymous selector. Value form: string. Example: --gene EGFR. Command sources: myvariant; source restrictions in the field description apply.
- `--has`: Require the selected component's identifier field. Only; help search variant
Require the selected component's identifier field. Only source-proven mappings are offered.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --has clinvar. Command sources: myvariant; source restrictions in the field description apply.
- `--limit`: Source candidates per page, default 10, maximum 50; all candidates remain candidates. Value form: integer. Example: --limit 50. Command sources: myvariant; source restrictions in the field description apply.
- `--offset`: Source offset within this command's bounded result window; larger source totals remain visible. Value form: integer. Example: --offset 0. Command sources: myvariant; source restrictions in the field description apply.
- `--protein`: Explicit substitution or synonymous change using checked amino-acid codes. Value form: string. Example: --protein L858R. Command sources: myvariant; source restrictions in the field description apply.
- `--query`: Raw MyVariant query, passed and echoed unchanged when component policy allows; requires --source myvariant. With disabled components use declared selectors instead. Value form: string. Example: --query CA123643. Command sources: myvariant; source restrictions in the field description apply.
- `--region`: Exact one-based inclusive MyVariant region, such as chr7:55259500-55259600; requires --build. Value form: string. Example: --region chr7:140453136-140453136. Command sources: myvariant; source restrictions in the field description apply.
- `--review-status`: Exact MyVariant clinvar.rcv.review_status phrase; no star conversion. Requires a declared protein or region selector. Value form: string. Example: --review-status 'no assertion criteria provided'. Command sources: myvariant; source restrictions in the field description apply.
- `--source`: Explicit source language; required for a raw query; help search variant
Explicit source language; required for a raw query.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source myvariant. Command sources: myvariant; source restrictions in the field description apply.

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
  drug           Retrieve a drug by source-native ID, brand or generic name, with explicit source-scoped sections.
  gene           Get one to twenty human gene inputs in their original order.
  article        Get one to twenty article inputs in caller order.
  trial          Retrieve exact trial cards in input order; request bounded intact source sections with independent continuations.
  variant        Read an rsID, CAid, clinvar:N, VCV accession, reference-qualified HGVS or VCF-style genomic coordinates such as 7-140753336-A-T through the shared resolver. MyVariant identity lookups default to GRCh38; --build overrides it. Chromosome genomic HGVS requires --build. Use search variant --gene BRAF --protein V600E or search variant --gene PTEN --coding c.333G>T for gene shorthand. Every candidate remains available; source assertions do not classify the variant for the caller.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          NCI is the biomarker concept source; help get biomarker
          NCI is the biomarker concept source.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get NCI biomarker concepts by code. Sources: nci.

- `id`: Exact NCIt code, such as C96022.
- `--source`: NCI is the biomarker concept source; help get biomarker
NCI is the biomarker concept source.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source nci. Command sources: nci; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get research author details. Sources: orcid, semantic-scholar.

- `ids`: One or more exact qualified IDs.

## biomcp get cell-line

```text
One exact identity and an optional bounded named section.

Usage: biomcp get cell-line [OPTIONS] <id> [section]

Arguments:
  <id>
          One exact primary accession or qualified source ID.

  [section]
          Omit for the default card; all returns bounded first pages; help get cell-line

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --section-limit <section_limit>
          Explicit row limit for each requested collection; native variation defaults to three.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --section-offset <section_offset>
          Offset within each independently requested collection.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get cell line details and cross-references. Sources: cellosaurus.

- `id`: One exact primary accession or qualified source ID.
- `section`: Omit for the default card; all returns bounded first pages; help get cell-line
Listed choices accept any letter case; replies use their listed spelling.
- `--section-limit`: Explicit row limit for each requested collection; source variation defaults to three. Value form: integer. Example: --section-limit 25. Command sources: cellosaurus; source restrictions in the field description apply.
- `--section-offset`: Offset within each independently requested collection. Value form: integer. Example: --section-offset 1. Command sources: cellosaurus; source restrictions in the field description apply.

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
          Association loci or study associations and ancestries; help get gwas

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --section-limit <section_limit>
          Maximum native relation rows.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --section-offset <section_offset>
          Native relation row offset.

      --snapshot <snapshot>
          Retained ancestry reply for a local page.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get GWAS Catalog associations or studies. Sources: gwas.

- `id`: Qualified association ID or exact GCST study accession.
- `section`: Association loci or study associations and ancestries; help get gwas
Listed choices accept any letter case; replies use their listed spelling.
- `--section-limit`: Maximum source relation rows. Value form: integer. Example: --section-limit 1. Command sources: gwas; source restrictions in the field description apply.
- `--section-offset`: Source relation row offset. Value form: integer. Example: --section-offset 0. Command sources: gwas; source restrictions in the field description apply.
- `--snapshot`: Retained ancestry reply for a local page. Value form: string. Example: --snapshot <snapshot> (syntax; replace the placeholder with your retained value). Command sources: gwas; source restrictions in the field description apply.

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
          Select native sections or all bounded first pages; function; help get protein

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --section-limit <section_limit>
          First-page row limit; function keeps one whole native text record and does not page its paragraphs.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get protein details and source annotations. Sources: complex-portal, hgnc, interpro, uniprot.

- `input`: Source accessions or gene symbols, preserving caller order.
- `--section`: Select source sections or all bounded first pages; function; help get protein
Listed choices accept any letter case; replies use their listed spelling. Value form: repeatable values (repeat the flag). Example: --section function. Command sources: complex-portal, hgnc, interpro, uniprot; source restrictions in the field description apply.
- `--section-limit`: First-page row limit; function keeps one whole source text record and does not page its paragraphs. Value form: integer. Example: --section-limit 2. Command sources: complex-portal, hgnc, interpro, uniprot; source restrictions in the field description apply.

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
          Absent means the source identity card; help get phenotype

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --section-limit <section_limit>
          Default five text/link rows or one complete association row.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --section-offset <section_offset>
          Native section row offset.

      --snapshot <snapshot>
          Retained exact source reply for local text and link pages; unsupported for diseases and genes.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --source <source>
          HPO or Monarch for cards, synonyms and xrefs; Monarch only; help get phenotype
          HPO or Monarch for cards, synonyms and xrefs; Monarch only for hierarchy, diseases and genes. Defaults to HPO for cards, synonyms and xrefs, otherwise Monarch.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get human phenotype details and relationships. Sources: hpo, monarch.

- `id`: Exact HP identifier.
- `section`: Absent means the source identity card; help get phenotype
Listed choices accept any letter case; replies use their listed spelling.
- `--section-limit`: Default five text/link rows or one complete association row. Value form: integer. Example: --section-limit 1. Command sources: hpo, monarch; source restrictions in the field description apply.
- `--section-offset`: Source section row offset. Value form: integer. Example: --section-offset 0. Command sources: hpo, monarch; source restrictions in the field description apply.
- `--snapshot`: Retained exact source reply for local text and link pages; unsupported for diseases and genes. Value form: string. Example: --snapshot <snapshot> (syntax; replace the placeholder with your retained value). Command sources: hpo, monarch; source restrictions in the field description apply.
- `--source`: HPO or Monarch for cards, synonyms and xrefs; Monarch only; help get phenotype
HPO or Monarch for cards, synonyms and xrefs; Monarch only for hierarchy, diseases and genes. Defaults to HPO for cards, synonyms and xrefs, otherwise Monarch.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source monarch. Command sources: hpo, monarch; source restrictions in the field description apply.

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
          Omit for the card; WikiPathways has summary and genes; help get pathway

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --fresh[=<fresh>]
          Request a new source observation.

          [default: false]
          [possible values: true, false]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --pool <pool>
          Retained snapshot, when continuing a source relation.

      --section-limit <section_limit>
          Bounded initial section size; each all section has its own first page.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --section-offset <section_offset>
          Native relation offset for a selected genes or events section.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

```

Get human pathway details. Sources: reactome, wikipathways.

- `--fresh`: Request a new source observation. Value form: boolean. Example: --fresh. Command sources: reactome, wikipathways; source restrictions in the field description apply.
- `id`: Exact human Reactome or WikiPathways identity without version suffixes.
- `--pool`: Retained snapshot, when continuing a source relation. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: reactome, wikipathways; source restrictions in the field description apply.
- `section`: Omit for the card; WikiPathways has summary and genes; help get pathway
Listed choices accept any letter case; replies use their listed spelling.
- `--section-limit`: Bounded initial section size; each all section has its own first page. Value form: integer. Example: --section-limit 5. Command sources: reactome, wikipathways; source restrictions in the field description apply.
- `--section-offset`: Source relation offset for a selected genes or events section. Value form: integer. Example: --section-offset 0. Command sources: reactome, wikipathways; source restrictions in the field description apply.

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
          Omit for the card; all returns bounded first pages only; help get study

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --gene <gene>
          Exact cBioPortal HUGO gene symbol or Entrez ID; required for mutations.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --molecular-profile <molecular_profile>
          Exact mutation profile ID; omitted selects the study's first mutation profile.

      --section-limit <section_limit>
          Maximum records in each selected section.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --section-offset <section_offset>
          Offset within one named section; all requires zero.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get cancer genomics study details. Sources: cbioportal.

- `--gene`: Exact cBioPortal HUGO gene symbol or Entrez ID; required for mutations. Value form: string. Example: --gene BRAF. Command sources: cbioportal; source restrictions in the field description apply.
- `id`: Exact source study ID; no default or repair.
- `--molecular-profile`: Exact mutation profile ID; omitted selects the study's first mutation profile. Value form: string. Example: --molecular-profile skcm_tcga_pan_can_atlas_2018_mutations. Command sources: cbioportal; source restrictions in the field description apply.
- `section`: Omit for the card; all returns bounded first pages only; help get study
Listed choices accept any letter case; replies use their listed spelling.
- `--section-limit`: Maximum records in each selected section. Value form: integer. Example: --section-limit 1. Command sources: cbioportal; source restrictions in the field description apply.
- `--section-offset`: Offset within one named section; all requires zero. Value form: integer. Example: --section-offset 45. Command sources: cbioportal; source restrictions in the field description apply.

Omit for the card; all returns bounded first pages only.

- `profiles`: Five exact source molecular profile references.
- `sample-lists`: Five sample-list metadata records, without member sample IDs.
- `mutations`: Source mutation rows for the required gene and selected mutation profile.
- `samples`: Source sample identities and types only.
- `files`: Five repository file references; no automatic download.
- `all`: Bounded first pages; mutations are included only when gene is supplied.

## biomcp get pgx

```text
PGx card or section. No phenotype, phase or dose is calculated.

Usage: biomcp get pgx [OPTIONS] --source <source> [section]

Arguments:
  [section]
          One named section; omitted means the CPIC gene card; help get pgx

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --drug-id <drug_id>
          Exact published CPIC drug key; recommendations only.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --gene <gene>
          Explicit source gene symbol.

      --kind <kind>
          One ClinPGx collection; clinical is the default only on; help get pgx
          One ClinPGx collection; clinical is the default only on annotations.

      --section-limit <section_limit>
          Rows per page: three clinical, five elsewhere.

      --section-offset <section_offset>
          Offset within the selected section.

      --snapshot <snapshot>
          Retained collection for local continuation.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --source <source>
          Required source, checked against the section; help get pgx
          Required source, checked against the section.

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get pharmacogenomic evidence and source sections. Sources: clinpgx, cpic.

- `--annotation-id`: Checked ClinPGx accession, never a shared PaId. Value form: string. Example: --annotation-id <annotation-id> (syntax; replace the placeholder with your retained value). Command sources: clinpgx, cpic; source restrictions in the field description apply.
- `--diplotype`: Unchanged declared expression required for the diplotype section only. Value form: string. Example: --diplotype '*1/*2A'. Command sources: clinpgx, cpic; source restrictions in the field description apply.
- `--drug-id`: Exact published CPIC drug key; recommendations only. Value form: string. Example: --drug-id <drug-id> (syntax; replace the placeholder with your retained value). Command sources: clinpgx, cpic; source restrictions in the field description apply.
- `--gene`: Explicit source gene symbol. Value form: string. Example: --gene DPYD. Command sources: clinpgx, cpic; source restrictions in the field description apply.
- `--kind`: One ClinPGx collection; clinical is the default only on; help get pgx
One ClinPGx collection; clinical is the default only on annotations.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --kind clinical. Command sources: clinpgx, cpic; source restrictions in the field description apply.
- `section`: One named section; omitted means the CPIC gene card; help get pgx
Listed choices accept any letter case; replies use their listed spelling.
- `--section-limit`: Rows per page: three clinical, five elsewhere. Value form: integer. Example: --section-limit 1. Command sources: clinpgx, cpic; source restrictions in the field description apply.
- `--section-offset`: Offset within the selected section. Value form: integer. Example: --section-offset 0. Command sources: clinpgx, cpic; source restrictions in the field description apply.
- `--snapshot`: Retained collection for local continuation. Value form: string. Example: --snapshot <snapshot> (syntax; replace the placeholder with your retained value). Command sources: clinpgx, cpic; source restrictions in the field description apply.
- `--source`: Required source, checked against the section; help get pgx
Required source, checked against the section.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source cpic. Command sources: clinpgx, cpic; source restrictions in the field description apply.

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
          GTR: genes, conditions, methods. PMA: regulatory,; help get diagnostic

Options:
      --bundle-revision <bundle_revision>
          Immutable stored bundle revision retained by a continuation.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --section-limit <section_limit>
          Maximum native rows; omitted means five rows, or one regulatory fragment.

      --section-offset <section_offset>
          Zero-based native row or fragment offset.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --source <source>
          Explicit source: GTR accession/version, WHO Product ID,; help get diagnostic
          Explicit source: GTR accession/version, WHO Product ID, PMA P number,
          510(k) K number or device recall Z-NNNN-YYYY.

      --supplement <supplement>
          Exact native PMA supplement; absent selects the base row.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get diagnostic test or device details. Sources: fda-510k, fda-device-recall, fda-pma, gtr, who-ivd.

- `--bundle-revision`: Immutable stored bundle revision retained by a continuation. Value form: string. Example: --bundle-revision <bundle-revision> (syntax; replace the placeholder with your retained value). Command sources: fda-510k, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `id`: Exact source identity, including version for GTR.
- `section`: GTR: genes, conditions, methods. PMA: regulatory,; help get diagnostic
Listed choices accept any letter case; replies use their listed spelling.
- `--section-limit`: Maximum source rows; omitted means five rows, or one regulatory fragment. Value form: integer. Example: --section-limit 2. Command sources: fda-510k, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--section-offset`: Zero-based source row or fragment offset. Value form: integer. Example: --section-offset 5. Command sources: fda-510k, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--source`: Explicit source: GTR accession/version, WHO Product ID,; help get diagnostic
Explicit source: GTR accession/version, WHO Product ID, PMA P number,
510(k) K number or device recall Z-NNNN-YYYY.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source fda-device-recall. Command sources: fda-510k, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.
- `--supplement`: Exact source PMA supplement; absent selects the base row. Value form: string. Example: --supplement S020. Command sources: fda-510k, fda-device-recall, fda-pma, gtr, who-ivd; source restrictions in the field description apply.

GTR: genes, conditions, methods. PMA: regulatory, supplements, summary.
WHO, 510(k) and device recalls have no sections; omit for the card.

- `genes`: GTR source gene rows; five per page.
- `conditions`: GTR source condition rows; five per page.
- `methods`: GTR source method entries; five per page.
- `summary`: FDA SSED PDF text: 25 MiB, 100 pages and 6144 escaped bytes per fragment; scans without text are explicit.
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
          FAERS owns drugs, reactions and identifiers; MAUDE owns; help get adverse-event

Options:
      --drug-index <drug_index>
          Absolute source-array index; required only for identifier sections.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --section-limit <section_limit>
          Whole source rows per section page.

      --section-offset <section_offset>
          Offset within the selected native section.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --snapshot <snapshot>
          Checked retained report snapshot for provider-free continuation.

      --source <source>
          FAERS by default; MAUDE is explicit; help get adverse-event
          FAERS by default; MAUDE is explicit.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get a drug, device or vaccine adverse event report. Sources: faers, maude.

- `--drug-index`: Absolute source-array index; required only for identifier sections. Value form: integer. Example: --drug-index 0. Command sources: faers, maude; source restrictions in the field description apply.
- `id`: Published report ID with no source prefix.
- `section`: FAERS owns drugs, reactions and identifiers; MAUDE owns; help get adverse-event
Listed choices accept any letter case; replies use their listed spelling.
- `--section-limit`: Whole source rows per section page. Value form: integer. Example: --section-limit 1. Command sources: faers, maude; source restrictions in the field description apply.
- `--section-offset`: Offset within the selected source section. Value form: integer. Example: --section-offset 0. Command sources: faers, maude; source restrictions in the field description apply.
- `--snapshot`: Checked retained report snapshot for source-free continuation. Value form: string. Example: --snapshot <snapshot> (syntax; replace the placeholder with your retained value). Command sources: faers, maude; source restrictions in the field description apply.
- `--source`: FAERS by default; MAUDE is explicit; help get adverse-event
FAERS by default; MAUDE is explicit.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source maude. Command sources: faers, maude; source restrictions in the field description apply.

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
          Exact ontology CURIE or a SEER site code or name.

  [section]
          One named section; absent means identity card only; help get disease

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --interval <interval>
          Survival-only interval code.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --ontology <ontology>
          One OLS ontology; help get disease
          One OLS ontology.

      --period <period>
          Incidence-only native period code.

      --race <race>
          Native race code.

      --section-limit <section_limit>
          Native rows per page: default 2 for Monarch genes, 1 for Monarch phenotypes, otherwise 5.

      --section-offset <section_offset>
          Native row offset within the selected section.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --sex <sex>
          Native sex code.

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

      --source <source>
          Default MyDisease; no automatic fallback; help get disease
          Default MyDisease; no automatic fallback.

```

Get disease details and source statistics. Sources: monarch, mydisease, ols, seer.

- `--age-range`: Source age-range code. Value form: string. Example: --age-range 1. Command sources: monarch, mydisease, ols, seer; source restrictions in the field description apply.
- `--dataset`: Requested source dataset selector, not proof of returned membership. Value form: string. Example: --dataset 4. Command sources: monarch, mydisease, ols, seer; source restrictions in the field description apply.
- `id`: Exact ontology CURIE or a SEER site code or name.
- `--interval`: Survival-only interval code. Value form: string. Example: --interval 5. Command sources: monarch, mydisease, ols, seer; source restrictions in the field description apply.
- `--ontology`: One OLS ontology; help get disease
One OLS ontology.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --ontology mondo. Command sources: monarch, mydisease, ols, seer; source restrictions in the field description apply.
- `--period`: Incidence-only source period code. Value form: string. Example: --period 5. Command sources: monarch, mydisease, ols, seer; source restrictions in the field description apply.
- `--race`: Source race code. Value form: string. Example: --race 1. Command sources: monarch, mydisease, ols, seer; source restrictions in the field description apply.
- `section`: One named section; absent means identity card only; help get disease
Listed choices accept any letter case; replies use their listed spelling.
- `--section-limit`: Source rows per page: default 2 for Monarch genes, 1 for Monarch phenotypes, otherwise 5. Value form: integer. Example: --section-limit 1. Command sources: monarch, mydisease, ols, seer; source restrictions in the field description apply.
- `--section-offset`: Source row offset within the selected section. Value form: integer. Example: --section-offset 1. Command sources: monarch, mydisease, ols, seer; source restrictions in the field description apply.
- `--sex`: Source sex code. Value form: string. Example: --sex 3. Command sources: monarch, mydisease, ols, seer; source restrictions in the field description apply.
- `--source`: Default MyDisease; no automatic fallback; help get disease
Default MyDisease; no automatic fallback.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source seer. Command sources: monarch, mydisease, ols, seer; source restrictions in the field description apply.

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
Retrieve a drug by source-native ID, brand or generic name, with explicit source-scoped sections.

Usage: biomcp get drug [OPTIONS] <input> [section]...

Arguments:
  <input>
          Source ID, brand or generic name.
          Source-native ID or complete brand or generic name; WHO references retain native spaces. Exact brands rank before generics; Drugs@FDA prefers NDA/BLA before ANDA, then earliest approval. Other sources retain search order. The chosen record reports up to five alternatives and their total. Generic names also accept one trailing listed salt or ester term: mesylate, hydrochloride, sodium, potassium, calcium, sulfate, maleate, tartrate, citrate, phosphate, acetate, besylate, fumarate, succinate, tosylate, bromide, chloride. Brand names identical to their own ingredient count as generic. USAN nomenclature: https://www.ama-assn.org/about/united-states-adopted-names-usan/united-states-adopted-names-naming-guidelines

  [section]...
          Source sections; help get drug

Options:
      --civic-cursor <civic_cursor>
          CIViC continuation.
          Native continuation cursor for that exact drug, profile and claim collection.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --civic-kind <civic_kind>
          CIViC collection; help get drug
          CIViC collection.
          Independent native evidence or assertion collection within the selected profile.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --civic-profile <civic_profile>
          Selected native CIViC molecular profile for a drug-scoped claim stream.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --document-id <document_id>
          FDA letter document ID.
          Exact native Drugs@FDA approval letter document ID. Text is bounded to 25 MiB, 100 pages
          and 6144 escaped bytes per fragment. No-text scans are explicit.

      --evidence-level <evidence_level>...
          A–E; help get drug
          A–E
          Native CIViC evidence levels; repeat to include several levels.

      --evidence-status <evidence_status>
          Status; help get drug
          Status
          Native CIViC evidence status; omitted preserves the source's existing default.

      --label-section <label_section>
          Exact openFDA SPL field.
          Exact published openFDA SPL section field, including table fields; required for the label section.

      --product-number <product_number>
          FDA product number.
          Explicit native application product number, only for approvals.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --section-limit <section_limit>
          Named section page size, refused on an identity-only card.

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

      --section-offset <section_offset>
          Named section continuation, refused on an identity-only card.

      --source <source>
          Omit to infer the source of an unambiguous section; identity or; help get drug
          Omit to infer the source of an unambiguous section; identity or mixed sections use MyChem.

```

Get drug and medicine details. Sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq.

- `--civic-cursor`: CIViC continuation.
Source continuation cursor for that exact drug, profile and claim collection. Value form: string. Example: --civic-cursor 0:A2. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--civic-kind`: CIViC collection; help get drug
CIViC collection.
Independent source evidence or assertion collection within the selected profile.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --civic-kind evidence. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--civic-profile`: Selected source CIViC molecular profile for a drug-scoped claim stream. Value form: integer. Example: --civic-profile 1. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--document-id`: FDA letter document ID.
Exact source Drugs@FDA approval letter document ID. Text is bounded to 25 MiB, 100 pages
and 6144 escaped bytes per fragment. No-text scans are explicit. Value form: string. Example: --document-id 64255. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--evidence-level`: A–E; help get drug
A–E
Source CIViC evidence levels; repeat to include several levels.
Listed choices accept any letter case; replies use their listed spelling. Value form: repeatable values (repeat the flag). Example: --evidence-level A. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--evidence-status`: Status; help get drug
Status
Source CIViC evidence status; omitted preserves the source's existing default.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --evidence-status accepted. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `input`: Source ID, brand or generic name.
Source-source ID or complete brand or generic name; WHO references retain source spaces. Exact brands rank before generics; Drugs@FDA prefers NDA/BLA before ANDA, then earliest approval. Other sources retain search order. The chosen record reports up to five alternatives and their total. Generic names also accept one trailing listed salt or ester term: mesylate, hydrochloride, sodium, potassium, calcium, sulfate, maleate, tartrate, citrate, phosphate, acetate, besylate, fumarate, succinate, tosylate, bromide, chloride. Brand names identical to their own ingredient count as generic. USAN nomenclature: https://www.ama-assn.org/about/united-states-adopted-names-usan/united-states-adopted-names-naming-guidelines
- `--label-section`: Exact openFDA SPL field.
Exact published openFDA SPL section field, including table fields; required for the label section. Value form: string. Example: --label-section warnings_and_cautions. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--product-number`: FDA product number.
Explicit source application product number, only for approvals. Value form: string. Example: --product-number 001. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `section`: Source sections; help get drug
Listed choices accept any letter case; replies use their listed spelling.
- `--section-limit`: Named section page size, refused on an identity-only card. Value form: integer. Example: --section-limit 2. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--section-offset`: Named section continuation, refused on an identity-only card. Value form: integer. Example: --section-offset 1. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.
- `--source`: Omit to infer the source of an unambiguous section; identity or; help get drug
Omit to infer the source of an unambiguous section; identity or mixed sections use MyChem.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source ema. Command sources: chembl, civic, ddinter, drugsfda, ema, fda-drug-recall, mychem, openfda-label, opentargets, who-pq; source restrictions in the field description apply.

One named disclosure per call for every source; empty means identity only.

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
          Requested section words, preserving caller order and repetitions; help get gene

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get gene identities and source annotations. Sources: civic, clingen, gencc, gnomad, hgnc, hpa, mygene, oncokb, reactome, uniprot.

- `input`: Gene symbols or checked identifiers, in input order.
- `--section`: Requested section words, preserving caller order and repetitions; help get gene
Listed choices accept any letter case; replies use their listed spelling. Value form: repeatable values (repeat the flag). Example: --section protein. Command sources: civic, clingen, gencc, gnomad, hgnc, hpa, mygene, oncokb, reactome, uniprot; source restrictions in the field description apply.

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
          Requested section words; trailing words and --section share; help get article

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get research article details. Sources: europepmc, icite, pmc-oa, pubmed, pubtator, semantic-scholar.

- `input`: PubMed, PMC or DOI identifiers in input order.
- `--section`: Requested section words; trailing words and --section share; help get article
Listed choices accept any letter case; replies use their listed spelling. Value form: repeatable values (repeat the flag). Example: --section annotations. Command sources: europepmc, icite, pmc-oa, pubmed, pubtator, semantic-scholar; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Character offset for ClinicalTrials.gov eligibility or NCI summary; whole-record offset for other collections.

      --pool <pool>
          Checked stored source record, supplied by a continuation.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --section <section>
          Optional intact source sections. Default cards only; diseases; help get trial

      --source <source>
          Explicit provider; defaults to ClinicalTrials.gov and never; help get trial
          Explicit provider; defaults to ClinicalTrials.gov and never falls back.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get clinical trial details. Sources: ctgov, nci.

- `input`: One to twenty exact NCT identifiers, in caller order.
- `--limit`: Whole-record count per source collection page; text fragments follow the text budget. Value form: integer. Example: --limit 1. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--offset`: Character offset for ClinicalTrials.gov eligibility or NCI summary; whole-record offset for other collections. Value form: integer. Example: --offset 0. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--pool`: Checked stored source record, supplied by a continuation. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: ctgov, nci; source restrictions in the field description apply.
- `--section`: Optional intact source sections. Default cards only; diseases; help get trial
Listed choices accept any letter case; replies use their listed spelling. Value form: repeatable values (repeat the flag). Example: --section outcomes. Command sources: ctgov, nci; source restrictions in the field description apply.
- `--source`: Explicit source; defaults to ClinicalTrials.gov and never; help get trial
Explicit source; defaults to ClinicalTrials.gov and never falls back.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source nci. Command sources: ctgov, nci; source restrictions in the field description apply.

Optional intact source sections. Default cards only; diseases and summary require NCI. NCI locations, contacts, references and documents are unsupported.

- `eligibility`: Intact criteria and structured age/sex fields.
- `outcomes`: Primary, secondary and other outcome records.
- `results`: Posted outcome values, analyses and adverse-event tables (ClinicalTrials.gov).
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
Read an rsID, CAid, clinvar:N, VCV accession, reference-qualified HGVS or VCF-style genomic coordinates such as 7-140753336-A-T through the shared resolver. MyVariant identity lookups default to GRCh38; --build overrides it. Chromosome genomic HGVS requires --build. Use search variant --gene BRAF --protein V600E or search variant --gene PTEN --coding c.333G>T for gene shorthand. Every candidate remains available; source assertions do not classify the variant for the caller.

Usage: biomcp get variant [OPTIONS] <input> [section]...

Arguments:
  <input>
          rs/CA/clinvar:N/VCV/HGVS/VCF
          rsID, CAid, clinvar:N, VCV accession, reference-qualified HGVS or VCF-style genomic coordinates.

  [section]...
          Sections; help get variant

Options:
      --build <build>
          Assembly; help get variant
          Assembly
          MyVariant identity assembly defaults to GRCh38; chromosome genomic HGVS requires an explicit build.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --civic-cursor <civic_cursor>
          Cursor
          Native CIViC cursor, retained unchanged.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --civic-kind <civic_kind>
          Claims; help get variant
          Claims
          Native CIViC claim collection within the selected profile.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --civic-profile <civic_profile>
          Selected CIViC molecular profile for native claim paging.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --evidence-level <evidence_level>...
          A–E; help get variant
          A–E
          Native CIViC evidence levels; repeat to include several levels.

      --evidence-status <evidence_status>
          Status; help get variant
          Status
          Native CIViC evidence status; omitted preserves the source's existing default.

      --limit <limit>
          Candidate rows per page; more remain reachable through typed next commands.

      --offset <offset>
          Candidate rows to skip in source order.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --resolver <resolver>
          Resolver; help get variant
          Resolver.
          Resolver for reference-qualified notation; never repairs the input locally.

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

      --section-limit <section_limit>
          Rows per named source section page; omitted uses the native source default.

      --section-offset <section_offset>
          Source rows to skip independently of identity candidate paging.

      --tumor-type <tumor_type>
          Tumor
          Explicit tumor context for keyed OncoKB annotation.

```

Get genetic variants. Sources: alphagenome, car, civic, clinvar, erepo, gnomad, gwas, hotspots, mutalyzer, myvariant, oncokb, variantvalidator.

- `--build`: Assembly; help get variant
Assembly
MyVariant identity assembly defaults to GRCh38; chromosome genomic HGVS requires an explicit build.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --build GRCh37. Command sources: alphagenome, car, civic, clinvar, erepo, gnomad, gwas, hotspots, mutalyzer, myvariant, oncokb, variantvalidator; source restrictions in the field description apply.
- `--civic-cursor`: Cursor
Source CIViC cursor, retained unchanged. Value form: string. Example: --civic-cursor 0:E2. Command sources: alphagenome, car, civic, clinvar, erepo, gnomad, gwas, hotspots, mutalyzer, myvariant, oncokb, variantvalidator; source restrictions in the field description apply.
- `--civic-kind`: Claims; help get variant
Claims
Source CIViC claim collection within the selected profile.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --civic-kind evidence. Command sources: alphagenome, car, civic, clinvar, erepo, gnomad, gwas, hotspots, mutalyzer, myvariant, oncokb, variantvalidator; source restrictions in the field description apply.
- `--civic-profile`: Selected CIViC molecular profile for source claim paging. Value form: integer. Example: --civic-profile 12. Command sources: alphagenome, car, civic, clinvar, erepo, gnomad, gwas, hotspots, mutalyzer, myvariant, oncokb, variantvalidator; source restrictions in the field description apply.
- `--evidence-level`: A–E; help get variant
A–E
Source CIViC evidence levels; repeat to include several levels.
Listed choices accept any letter case; replies use their listed spelling. Value form: repeatable values (repeat the flag). Example: --evidence-level A. Command sources: alphagenome, car, civic, clinvar, erepo, gnomad, gwas, hotspots, mutalyzer, myvariant, oncokb, variantvalidator; source restrictions in the field description apply.
- `--evidence-status`: Status; help get variant
Status
Source CIViC evidence status; omitted preserves the source's existing default.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --evidence-status submitted. Command sources: alphagenome, car, civic, clinvar, erepo, gnomad, gwas, hotspots, mutalyzer, myvariant, oncokb, variantvalidator; source restrictions in the field description apply.
- `input`: rs/CA/clinvar:N/VCV/HGVS/VCF
rsID, CAid, clinvar:N, VCV accession, reference-qualified HGVS or VCF-style genomic coordinates.
- `--limit`: Candidate rows per page; more remain reachable through typed next commands. Value form: integer. Example: --limit 10. Command sources: alphagenome, car, civic, clinvar, erepo, gnomad, gwas, hotspots, mutalyzer, myvariant, oncokb, variantvalidator; source restrictions in the field description apply.
- `--offset`: Candidate rows to skip in source order. Value form: integer. Example: --offset 0. Command sources: alphagenome, car, civic, clinvar, erepo, gnomad, gwas, hotspots, mutalyzer, myvariant, oncokb, variantvalidator; source restrictions in the field description apply.
- `--resolver`: Resolver; help get variant
Resolver.
Resolver for reference-qualified notation; never repairs the input locally.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --resolver variantvalidator. Command sources: alphagenome, car, civic, clinvar, erepo, gnomad, gwas, hotspots, mutalyzer, myvariant, oncokb, variantvalidator; source restrictions in the field description apply.
- `section`: Sections; help get variant
Listed choices accept any letter case; replies use their listed spelling.
- `--section-limit`: Rows per named source section page; omitted uses the source default. Value form: integer. Example: --section-limit 2. Command sources: alphagenome, car, civic, clinvar, erepo, gnomad, gwas, hotspots, mutalyzer, myvariant, oncokb, variantvalidator; source restrictions in the field description apply.
- `--section-offset`: Source rows to skip independently of identity candidate paging. Value form: integer. Example: --section-offset 0. Command sources: alphagenome, car, civic, clinvar, erepo, gnomad, gwas, hotspots, mutalyzer, myvariant, oncokb, variantvalidator; source restrictions in the field description apply.
- `--tumor-type`: Tumor
Explicit tumor context for keyed OncoKB annotation. Value form: string. Example: --tumor-type <tumor-type> (syntax; replace the placeholder with your retained value). Command sources: alphagenome, car, civic, clinvar, erepo, gnomad, gwas, hotspots, mutalyzer, myvariant, oncokb, variantvalidator; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Detailed disclosure when requested; help author papers

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --full[=<full>]
          Legacy alias for details; normalized on construction.

          [default: false]
          [possible values: true, false]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

  -l, --limit <limit>
          Bounded row count.

      --offset <offset>
          Native or retained collection offset.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --pool <pool>
          Retained collection identity.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

List an author's research papers. Sources: orcid, semantic-scholar.

- `--full`: Legacy alias for details; normalized on construction. Value form: boolean. Example: --full. Command sources: orcid, semantic-scholar; source restrictions in the field description apply.
- `id`: Exact qualified source record.
- `--limit`: Bounded row count. Value form: integer. Example: --limit 5. Command sources: orcid, semantic-scholar; source restrictions in the field description apply.
- `--offset`: Source or retained collection offset. Value form: integer. Example: --offset 0. Command sources: orcid, semantic-scholar; source restrictions in the field description apply.
- `--pool`: Retained collection identity. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: orcid, semantic-scholar; source restrictions in the field description apply.
- `section`: Detailed disclosure when requested; help author papers
Listed choices accept any letter case; replies use their listed spelling.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get a cell line's ChEMBL assay count. Sources: chembl.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Native or retained page offset.

      --pool <pool>
          Retained collection identity.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --position <position>
          Native one-based residue; zero fails during deserialization.

      --type <type>
          Exact UniProt native label, without an alias map.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get protein features by residue or region. Sources: uniprot.

- `accession`: Exact source protein sequence identity.
- `--limit`: Five rows by default, at most 25. Value form: integer. Example: --limit 1. Command sources: uniprot; source restrictions in the field description apply.
- `--offset`: Source or retained page offset. Value form: integer. Example: --offset 0. Command sources: uniprot; source restrictions in the field description apply.
- `--pool`: Retained collection identity. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: uniprot; source restrictions in the field description apply.
- `--position`: Source one-based residue; zero fails during deserialization. Value form: integer. Example: --position 25. Command sources: uniprot; source restrictions in the field description apply.
- `--type`: Exact UniProt source label, without an alias map. Value form: string. Example: --type Chain. Command sources: uniprot; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Native or retained page offset.

      --pool <pool>
          Retained collection identity.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get protein isoforms and alternative products. Sources: uniprot.

- `accession`: Exact source protein sequence identity.
- `--limit`: Five rows by default, at most 25. Value form: integer. Example: --limit 1. Command sources: uniprot; source restrictions in the field description apply.
- `--offset`: Source or retained page offset. Value form: integer. Example: --offset 0. Command sources: uniprot; source restrictions in the field description apply.
- `--pool`: Retained collection identity. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: uniprot; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Native or retained page offset.

      --pool <pool>
          Retained collection identity.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --type <type>
          Exact native InterPro type, not a normalized domain label.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get protein domains and families from InterPro. Sources: interpro.

- `accession`: Exact source protein sequence identity.
- `--limit`: Five rows by default, at most 25. Value form: integer. Example: --limit 1. Command sources: interpro; source restrictions in the field description apply.
- `--offset`: Source or retained page offset. Value form: integer. Example: --offset 0. Command sources: interpro; source restrictions in the field description apply.
- `--pool`: Retained collection identity. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: interpro; source restrictions in the field description apply.
- `--type`: Exact source InterPro type, not a normalized domain label. Value form: string. Example: --type <type> (syntax; replace the placeholder with your retained value). Command sources: interpro; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Native or retained page offset.

      --pool <pool>
          Retained collection identity.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

List protein complexes containing a protein. Sources: complex-portal.

- `accession`: Exact source protein sequence identity.
- `--limit`: Five rows by default, at most 25. Value form: integer. Example: --limit 1. Command sources: complex-portal; source restrictions in the field description apply.
- `--offset`: Source or retained page offset. Value form: integer. Example: --offset 0. Command sources: complex-portal; source restrictions in the field description apply.
- `--pool`: Retained collection identity. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: complex-portal; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --limit <limit>
          Five rows by default, at most 25.

      --offset <offset>
          Native or retained page offset.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --pool <pool>
          Retained collection identity.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

List the members of a protein complex. Sources: complex-portal.

- `accession`: Exact source protein sequence identity.
- `--complex`: Exact complex whose source participants contain the anchor. Value form: string. Example: --complex CPX-715. Command sources: complex-portal; source restrictions in the field description apply.
- `--limit`: Five rows by default, at most 25. Value form: integer. Example: --limit 1. Command sources: complex-portal; source restrictions in the field description apply.
- `--offset`: Source or retained page offset. Value form: integer. Example: --offset 0. Command sources: complex-portal; source restrictions in the field description apply.
- `--pool`: Retained collection identity. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: complex-portal; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Native or retained page offset.

      --pool <pool>
          Retained collection identity.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get protein structure references from PDB. Sources: uniprot.

- `accession`: Exact source protein sequence identity.
- `--limit`: Five rows by default, at most 25. Value form: integer. Example: --limit 1. Command sources: uniprot; source restrictions in the field description apply.
- `--offset`: Source or retained page offset. Value form: integer. Example: --offset 0. Command sources: uniprot; source restrictions in the field description apply.
- `--pool`: Retained collection identity. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: uniprot; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get a source protein sequence. Sources: uniprot.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Include every physical entity context for these references;; help pathway genes
          Include every physical entity context for these references; never clips source text.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

  -l, --limit <limit>
          Five complete relation rows by default, at most fifty.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Offset within the retained selected relation.

      --pool <pool>
          Immutable relation snapshot.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

List genes in a biological pathway. Sources: reactome, wikipathways.

- `details`: Include every physical entity context for these references;; help pathway genes
Include every physical entity context for these references; never clips source text.
Listed choices accept any letter case; replies use their listed spelling.
- `id`: Checked source-owned human pathway identity.
- `--limit`: Five complete relation rows by default, at most fifty. Value form: integer. Example: --limit 5. Command sources: reactome, wikipathways; source restrictions in the field description apply.
- `--offset`: Offset within the retained selected relation. Value form: integer. Example: --offset 0. Command sources: reactome, wikipathways; source restrictions in the field description apply.
- `--pool`: Immutable relation snapshot. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: reactome, wikipathways; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Offset within the retained selected relation.

      --pool <pool>
          Immutable relation snapshot.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

List events within a biological pathway. Sources: reactome.

- `id`: Checked source-owned human pathway identity.
- `--limit`: Five complete relation rows by default, at most fifty. Value form: integer. Example: --limit 50. Command sources: reactome; source restrictions in the field description apply.
- `--offset`: Offset within the retained selected relation. Value form: integer. Example: --offset 0. Command sources: reactome; source restrictions in the field description apply.
- `--pool`: Immutable relation snapshot. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: reactome; source restrictions in the field description apply.

## biomcp article

```text
Read article text, references, citations and files.

Usage: biomcp article [OPTIONS] <COMMAND>

Commands:
  supplement         Read a checked supplement file or page its original rows.
  pdf                Download one publisher PDF to an explicit file or existing folder.
  text               Return the abstract card by default, or explicitly read body text.
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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Matching-row or text-fragment offset.

      --replace[=<replace>]
          Replace an existing destination only when explicitly requested with save.

          [default: false]
          [possible values: true, false]

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --rows <rows>
          Inclusive original row range.

      --save <save>
          Save the whole original document to an explicit file or existing folder. Example: --save article-output.

      --sheet <sheet>
          Sheet name for row selection.

      --variant <variant>
          Explicit protein substitution.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Read a research article supplementary file. Sources: pmc-oa, pubmed.

- `file`: Exact file name from the supplements section.
- `--find`: Exact case-insensitive needle. Value form: string. Example: --find F1761I. Command sources: pmc-oa, pubmed; source restrictions in the field description apply.
- `input`: PubMed or PMC identity.
- `--offset`: Matching-row or text-fragment offset. Value form: integer. Example: --offset 3. Command sources: pmc-oa, pubmed; source restrictions in the field description apply.
- `--replace`: Replace an existing destination only when explicitly requested with save. Value form: boolean. Example: --replace. Command sources: pmc-oa, pubmed; source restrictions in the field description apply.
- `--rows`: Inclusive original row range. Value form: string. Example: --rows 84-86. Command sources: pmc-oa, pubmed; source restrictions in the field description apply.
- `--save`: Save the whole original document to an explicit file or existing folder. Example: --save article-output. Value form: string. Command sources: pmc-oa, pubmed; source restrictions in the field description apply.
- `--sheet`: Sheet name for row selection. Value form: string. Example: --sheet ST2. Command sources: pmc-oa, pubmed; source restrictions in the field description apply.
- `--variant`: Explicit protein substitution. Value form: string. Example: --variant P245S. Command sources: pmc-oa, pubmed; source restrictions in the field description apply.

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

      --replace[=<replace>]
          Replace an existing file only when explicitly requested.

          [default: false]
          [possible values: true, false]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --save <save>
          Destination file or existing folder.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Save a research article PDF. Sources: semantic-scholar.

- `input`: Article identity.
- `--replace`: Replace an existing file only when explicitly requested. Value form: boolean. Example: --replace. Command sources: semantic-scholar; source restrictions in the field description apply.
- `--save`: Destination file or existing folder. Value form: string. Example: --save owned.pdf. Command sources: semantic-scholar; source restrictions in the field description apply.

## biomcp article text

```text
Return the abstract card by default, or explicitly read body text.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Number of matching rows already returned.

      --part <part>
          One-based outline part; omit reader selectors for the abstract card.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --replace[=<replace>]
          Replace an existing destination only when explicitly requested with save.

          [default: false]
          [possible values: true, false]

      --save <save>
          Save the whole original document to an explicit file or existing folder. Example: --save article-output.

      --variant <variant>
          Explicit checked protein substitution. Example: --variant p.V600E.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get the abstract card by default, or explicitly read article body text. Sources: europepmc, pmc-oa, pubmed.

- `--find`: Case-insensitive exact text to find. Value form: string. Example: --find AXIN2. Command sources: europepmc, pmc-oa, pubmed; source restrictions in the field description apply.
- `input`: PubMed or PMC identity. DOI selectors are not accepted by the reader.
- `--offset`: Number of matching rows already returned. Value form: integer. Example: --offset 0. Command sources: europepmc, pmc-oa, pubmed; source restrictions in the field description apply.
- `--part`: One-based outline part; omit reader selectors for the abstract card. Value form: integer. Example: --part 1. Command sources: europepmc, pmc-oa, pubmed; source restrictions in the field description apply.
- `--replace`: Replace an existing destination only when explicitly requested with save. Value form: boolean. Example: --replace. Command sources: europepmc, pmc-oa, pubmed; source restrictions in the field description apply.
- `--save`: Save the whole original document to an explicit file or existing folder. Example: --save article-output. Value form: string. Command sources: europepmc, pmc-oa, pubmed; source restrictions in the field description apply.
- `--variant`: Explicit checked protein substitution. Example: --variant p.V600E. Value form: string. Command sources: europepmc, pmc-oa, pubmed; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Number of pool items to skip.

      --pool <pool>
          Fixed snapshot from an earlier page.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --source <source>
          Graph provider; iCite remains the default; help article citations
          Graph provider; iCite remains the default.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

List research papers citing an article. Sources: icite, pubmed, semantic-scholar.

- `--limit`: Maximum papers on one page. Value form: integer. Example: --limit 1. Command sources: icite, pubmed, semantic-scholar; source restrictions in the field description apply.
- `--offset`: Number of pool items to skip. Value form: integer. Example: --offset 0. Command sources: icite, pubmed, semantic-scholar; source restrictions in the field description apply.
- `pmid`: PMID, PMC or DOI; iCite accepts only a PMID.
- `--pool`: Fixed snapshot from an earlier page. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: icite, pubmed, semantic-scholar; source restrictions in the field description apply.
- `--source`: Graph source; iCite remains the default; help article citations
Graph source; iCite remains the default.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source semantic-scholar. Command sources: icite, pubmed, semantic-scholar; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Number of pool items to skip.

      --pool <pool>
          Fixed snapshot from an earlier page.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --source <source>
          Graph provider; iCite remains the default; help article citations
          Graph provider; iCite remains the default.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

List research papers referenced by an article. Sources: icite, pubmed, semantic-scholar.

- `--limit`: Maximum papers on one page. Value form: integer. Example: --limit 1. Command sources: icite, pubmed, semantic-scholar; source restrictions in the field description apply.
- `--offset`: Number of pool items to skip. Value form: integer. Example: --offset 112. Command sources: icite, pubmed, semantic-scholar; source restrictions in the field description apply.
- `pmid`: PMID, PMC or DOI; iCite accepts only a PMID.
- `--pool`: Fixed snapshot from an earlier page. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: icite, pubmed, semantic-scholar; source restrictions in the field description apply.
- `--source`: Graph source; iCite remains the default; help article citations
Graph source; iCite remains the default.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source semantic-scholar. Command sources: icite, pubmed, semantic-scholar; source restrictions in the field description apply.

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

      --fulltext[=<fulltext>]
          Include the citing paper's own explicit reference paragraphs.

          [default: false]
          [possible values: true, false]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --offset <offset>
          Context fragment position returned by the advancing step.

      --paragraph-offset <paragraph_offset>
          Full-text fragment position returned by the advancing step.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --pool <pool>
          Stored checked source scan and original hit positions.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get research article citation evidence. Sources: pmc-oa, pubmed, semantic-scholar.

- `cited`: Cited PMID, PMC identity or DOI.
- `citing`: Citing PMID, PMC identity or DOI.
- `--fulltext`: Include the citing paper's own explicit reference paragraphs. Value form: boolean. Example: --fulltext. Command sources: pmc-oa, pubmed, semantic-scholar; source restrictions in the field description apply.
- `--offset`: Context fragment position returned by the advancing step. Value form: integer. Example: --offset 0. Command sources: pmc-oa, pubmed, semantic-scholar; source restrictions in the field description apply.
- `--paragraph-offset`: Full-text fragment position returned by the advancing step. Value form: integer. Example: --paragraph-offset 0. Command sources: pmc-oa, pubmed, semantic-scholar; source restrictions in the field description apply.
- `--pool`: Stored checked source scan and original hit positions. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: pmc-oa, pubmed, semantic-scholar; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --negative <negative>...
          Up to ten negative paper identities.

      --offset <offset>
          Number of stored recommendations to skip.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --pool <pool>
          Fixed recommendation pool.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get related research article recommendations. Sources: semantic-scholar.

- `--limit`: Maximum recommendations requested, in source order. Value form: integer. Example: --limit 5. Command sources: semantic-scholar; source restrictions in the field description apply.
- `--negative`: Up to ten negative paper identities. Value form: repeatable values (repeat the flag). Example: --negative 33433946. Command sources: semantic-scholar; source restrictions in the field description apply.
- `--offset`: Number of stored recommendations to skip. Value form: integer. Example: --offset 1. Command sources: semantic-scholar; source restrictions in the field description apply.
- `--pool`: Fixed recommendation pool. Value form: string. Example: --pool <pool> (syntax; replace the placeholder with your retained value). Command sources: semantic-scholar; source restrictions in the field description apply.
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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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

      --replace[=<replace>]
          Replace an existing file only when explicitly requested.

          [default: false]
          [possible values: true, false]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --save <save>
          Caller-owned file or existing folder.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Save a clinical trial document. Sources: ctgov.

- `file`: Exact filename from the registry manifest; never a URL or inferred path.
- `input`: Exact registry identity.
- `--replace`: Replace an existing file only when explicitly requested. Value form: boolean. Example: --replace. Command sources: ctgov; source restrictions in the field description apply.
- `--save`: Caller-owned file or existing folder. Value form: string. Example: --save protocol.pdf. Command sources: ctgov; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Identity lookup assembly; omitted uses the shared; help variant assertions
          Identity lookup assembly; omitted uses the shared GRCh38 default.

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --limit <limit>
          Assertions per page; omitted uses the native source default.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offset <offset>
          Assertions to skip in source order.

      --source <source>
          One native assertion source; shared identity; help variant assertions
          One native assertion source; shared identity conversion precedes source access when needed.

      --version <version>
          VCV archive or ERepo document version; RCV/SCV selectors match a returned native version.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get genetic variant assertions from ClinVar or ClinGen. Sources: clinvar, erepo.

- `--assertion`: Source assertion accession or document ID. Value form: string. Example: --assertion PCER000001171. Command sources: clinvar, erepo; source restrictions in the field description apply.
- `--build`: Identity lookup assembly; omitted uses the shared; help variant assertions
Identity lookup assembly; omitted uses the shared GRCh38 default.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --build GRCh37. Command sources: clinvar, erepo; source restrictions in the field description apply.
- `input`: Supported variant ID; resolves one ClinVar Variation ID or ERepo CAid through the shared resolver.
- `--limit`: Assertions per page; omitted uses the source default. Value form: integer. Example: --limit 1. Command sources: clinvar, erepo; source restrictions in the field description apply.
- `--offset`: Assertions to skip in source order. Value form: integer. Example: --offset 0. Command sources: clinvar, erepo; source restrictions in the field description apply.
- `--source`: One source assertion source; shared identity; help variant assertions
One source assertion source; shared identity conversion precedes source access when needed.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source clinvar. Command sources: clinvar, erepo; source restrictions in the field description apply.
- `--version`: VCV archive or ERepo document version; RCV/SCV selectors match a returned source version. Value form: string. Example: --version 1.0.0. Command sources: clinvar, erepo; source restrictions in the field description apply.

## biomcp variant population

```text
Read a bounded ancestry-detail page from the exact source-given GRCh38 allele.

Usage: biomcp variant population [OPTIONS] <input>

Arguments:
  <input>
          Exact published allele identity; ambiguous candidates must first be selected.

Options:
      --build <build>
          Explicit source assembly when required by identity; help variant population
          Explicit source assembly when required by identity input.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --detail[=<detail>]
          Read source ancestry groups, preserving overlapping groups individually.

          [default: false]
          [possible values: true, false]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --limit <limit>
          Maximum rows per page.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offset <offset>
          Native rows to skip.

      --resolver <resolver>
          Resolver for reference-qualified input; help variant population
          Resolver for reference-qualified input.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get genetic variant population frequencies. Sources: car, gnomad, mutalyzer, myvariant, variantvalidator.

- `--build`: Explicit source assembly when required by identity; help variant population
Explicit source assembly when required by identity input.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --build GRCh38. Command sources: car, gnomad, mutalyzer, myvariant, variantvalidator; source restrictions in the field description apply.
- `--detail`: Read source ancestry groups, preserving overlapping groups individually. Value form: boolean. Example: --detail. Command sources: car, gnomad, mutalyzer, myvariant, variantvalidator; source restrictions in the field description apply.
- `input`: Exact published allele identity; ambiguous candidates must first be selected.
- `--limit`: Maximum rows per page. Value form: integer. Example: --limit 3. Command sources: car, gnomad, mutalyzer, myvariant, variantvalidator; source restrictions in the field description apply.
- `--offset`: Source rows to skip. Value form: integer. Example: --offset 10. Command sources: car, gnomad, mutalyzer, myvariant, variantvalidator; source restrictions in the field description apply.
- `--resolver`: Resolver for reference-qualified input; help variant population
Resolver for reference-qualified input.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --resolver variantvalidator. Command sources: car, gnomad, mutalyzer, myvariant, variantvalidator; source restrictions in the field description apply.

## biomcp variant recurrence

```text
Observe exact-allele samples within one caller-selected source page.

Usage: biomcp variant recurrence [OPTIONS] --gene <gene> --profile <profile> --sample-set <sample_set> --source <source> --study <study> <input>

Arguments:
  <input>
          Variant identity resolving to one unique source-qualified genomic replacement with nonempty reference and alternate alleles (pure insertions and deletions are unavailable).

Options:
      --build <build>
          Identity lookup assembly; omitted uses the shared; help variant recurrence
          Identity lookup assembly; omitted uses the shared GRCh38 default.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --gene <gene>
          HGNC symbol resolved to the source's Entrez gene identifier.

      --limit <limit>
          Source mutation rows per page, including excluded alleles.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --page <page>
          Zero-based native source page.

      --profile <profile>
          Explicit mutation molecular profile.

      --sample-set <sample_set>
          Explicit sample list; selected size is never an allele denominator.

      --source <source>
          Explicit cohort source; help variant recurrence
          Explicit cohort source.

      --study <study>
          Explicit study; there is no default cohort.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Get genetic variant observations in a cohort. Sources: cbioportal.

- `--build`: Identity lookup assembly; omitted uses the shared; help variant recurrence
Identity lookup assembly; omitted uses the shared GRCh38 default.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --build GRCh37. Command sources: cbioportal; source restrictions in the field description apply.
- `--gene`: HGNC symbol resolved to the source's Entrez gene identifier. Value form: string. Example: --gene BRAF. Command sources: cbioportal; source restrictions in the field description apply.
- `input`: Variant identity resolving to one unique source-qualified genomic replacement with nonempty reference and alternate alleles (pure insertions and deletions are unavailable).
- `--limit`: Source mutation rows per page, including excluded alleles. Value form: integer. Example: --limit 3. Command sources: cbioportal; source restrictions in the field description apply.
- `--page`: Zero-based source page. Value form: integer. Example: --page 0. Command sources: cbioportal; source restrictions in the field description apply.
- `--profile`: Explicit mutation molecular profile. Value form: string. Example: --profile msk_impact_2017_mutations. Command sources: cbioportal; source restrictions in the field description apply.
- `--sample-set`: Explicit sample list; selected size is never an allele denominator. Value form: string. Example: --sample-set msk_impact_2017_all. Command sources: cbioportal; source restrictions in the field description apply.
- `--source`: Explicit cohort source; help variant recurrence
Explicit cohort source.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source cbioportal. Command sources: cbioportal; source restrictions in the field description apply.
- `--study`: Explicit study; there is no default cohort. Value form: string. Example: --study msk_impact_2017. Command sources: cbioportal; source restrictions in the field description apply.

## biomcp variant normalize

```text
Send unchanged reference-qualified notation to explicitly named services.

Usage: biomcp variant normalize [OPTIONS] --hgvs <hgvs> --source <source>

Options:
      --build <build>
          Caller assembly; required when VariantValidator is; help variant normalize
          Caller assembly; required when VariantValidator is selected.

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --hgvs <hgvs>
          Reference-qualified HGVS, or chromosome genomic HGVS with an explicit build for CAR.
          Checked for reference and syntax only; no full HGVS parser.

      --limit <limit>
          Source mapping rows per service, with typed continuation for remaining rows.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offset <offset>
          Source mapping rows to skip per service.

      --source <source>
          Explicit service; all preserves independent results; help variant normalize
          Explicit service; all preserves independent results without consensus.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Normalize reference-qualified genetic variant HGVS. Sources: car, mutalyzer, variantvalidator.

- `--build`: Caller assembly; required when VariantValidator is; help variant normalize
Caller assembly; required when VariantValidator is selected.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --build GRCh38. Command sources: car, mutalyzer, variantvalidator; source restrictions in the field description apply.
- `--hgvs`: Reference-qualified HGVS, or chromosome genomic HGVS with an explicit build for CAR.
Checked for reference and syntax only; no full HGVS parser. Value form: string. Example: --hgvs 'NM_004333.4:c.1799T>A'. Command sources: car, mutalyzer, variantvalidator; source restrictions in the field description apply.
- `--limit`: Source mapping rows per service, with typed continuation for remaining rows. Value form: integer. Example: --limit 10. Command sources: car, mutalyzer, variantvalidator; source restrictions in the field description apply.
- `--offset`: Source mapping rows to skip per service. Value form: integer. Example: --offset 0. Command sources: car, mutalyzer, variantvalidator; source restrictions in the field description apply.
- `--source`: Explicit service; all preserves independent results; help variant normalize
Explicit service; all preserves independent results without consensus.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --source mutalyzer. Command sources: car, mutalyzer, variantvalidator; source restrictions in the field description apply.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Synchronize a local source bundle.

- `bundle`: Local source bundle to synchronize.
Listed choices accept any letter case; replies use their listed spelling.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
      --force[=<force>]
          Repair managed files; retain unrelated files.

          [default: false]
          [possible values: true, false]

      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

Example: biomcp skill install biomcp-guidance

```

Install into DIR/skills/biomcp; a skills or biomcp directory is accepted directly. Existing files remain unchanged without --force.

- `directory`: Directory to inspect or install BioMCP guidance.
- `--force`: Repair managed files; retain unrelated files. Value form: boolean. Example: --force. Local command option; independent of source selection.

## biomcp source

```text
List source names as JSON. No source requests or key reads.

Usage: biomcp source [OPTIONS] <COMMAND>

Commands:
  list  List the source names a hosted request may leave out.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

Example: biomcp source list

```

List source names as JSON. No source requests or key reads.


## biomcp source list

```text
List the source names a hosted request may leave out.

Usage: biomcp source list [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

Example: biomcp source list

```

List the source names a hosted request may leave out.


## biomcp version

```text
Print version; --verbose adds build, schema and path provenance. No source requests or key reads.

Usage: biomcp version [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --verbose[=<verbose>]
          Include commit, target, schema versions, config and cache paths.

          [default: false]
          [possible values: true, false]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

Example: biomcp version

```

Print version; --verbose adds build, schema and path provenance. No source requests or key reads.

- `--verbose`: Include commit, target, schema versions, config and cache paths. Value form: boolean. Example: --verbose. Local command option; independent of source selection.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

Example: biomcp mcp-config

```

Print MCP configuration with this executable’s absolute path. JSON formats name their top-level object and retain released client aliases. The codex client prints its shell registration command. No source requests or key reads.

- `--client`: Choose a JSON client or the codex shell registration command; default mcp-servers.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --client mcp-servers. Local command option; independent of source selection.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

Example: biomcp health --offline

```

Report publisher terms and bounded source reachability; at most one page for each implemented source.


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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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

      --replace[=<replace>]
          Replace an existing stored credential.

          [default: false]
          [possible values: true, false]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

Example: biomcp key set oncokb

```

Register one credential from a no-echo prompt or one standard-input line. Use --config PATH to choose the configuration file.

- `source`: Source credential name to register or remove.
Listed choices accept any letter case; replies use their listed spelling.
- `--replace`: Replace an existing stored credential. Value form: boolean. Example: --replace. Local command option; independent of source selection.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

Example: biomcp key remove oncokb

```

Remove a stored credential while preserving other settings.

- `source`: Source credential name to register or remove.
Listed choices accept any letter case; replies use their listed spelling.

## biomcp serve

```text
Serve MCP over standard input or HTTP.

Usage: biomcp serve [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --tool-mode <tool-mode>
          Advertise one CLI tool or generated typed command tools; default single.

          [possible values: single, typed]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --hosted[=<hosted>]
          Refuse host file saves, health details and store or key changes over MCP.

          [default: false]
          [possible values: true, false]

      --rate-share <rate-share>
          Use this share of keyed source rates in hosted mode (greater than 0, at most 1).

          [default: 1]

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --http [<http>]
          Listen on an HTTP address; omit its value for the loopback default.

      --http_max_calls <http_max_calls>
          Limit tool calls in flight. Example: --http_max_calls 8.

      --http_max_connections <http_max_connections>
          Limit open HTTP connections; must cover http_max_calls. Example: --http_max_connections 64.

      --allow-host <allow-host>
          Allow an HTTP Host authority; required for non-loopback listeners. Example: --allow-host api.example.org.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

Example: biomcp serve
Example: biomcp help serve

```

Serve MCP over stdio or --http [ADDR]. Non-loopback binds require --allow-host HOST or http.allowed_hosts; neither authenticates callers. HTTP refuses sync, store clean and --fresh. Stdio saves refuse --replace. --hosted refuses file saves, health details, sync, store clean, --fresh and key commands on every transport with a usage.mcp_unavailable error. MCP revisions: 2026-07-28, 2025-11-25, 2025-06-18, 2025-03-26. --rate-share scales only keyed hosted source rates. Hosted HTTP calls have bounded work and one whole-call deadline. Send MCP-Protocol-Version on HTTP requests. Without it, initialize uses its requested legacy revision; other requests default to 2025-03-26. Unsupported headers return HTTP 400.

- `--tool-mode`: Advertise one CLI tool or generated typed command tools; default single.
Listed choices accept any letter case; replies use their listed spelling. Value form: string. Example: --tool-mode single. Local command option; independent of source selection.
- `--hosted`: Refuse host file saves, health details and store or key changes over MCP. Value form: boolean. Example: --hosted. Local command option; independent of source selection.
- `--rate-share`: Use this share of keyed source rates in hosted mode (greater than 0, at most 1). Value form: string. Example: --rate-share 1. Local command option; independent of source selection.
- `--http`: Listen on an HTTP address; omit its value for the loopback default. Value form: string. Example: --http 127.0.0.1:8080. Local command option; independent of source selection.
- `--http_max_calls`: Limit tool calls in flight. Example: --http_max_calls 8. Value form: string. Local command option; independent of source selection.
- `--http_max_connections`: Limit open HTTP connections; must cover http_max_calls. Example: --http_max_connections 64. Value form: string. Local command option; independent of source selection.
- `--allow-host`: Allow an HTTP Host authority; required for non-loopback listeners. Example: --allow-host api.example.org. Value form: string. Local command option; independent of source selection.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
  search             Search source records by entity and checked filters.
  get                Retrieve one entity and its source sections.
  author             Read papers attributed to a source author record.
  cell-line          Read source cell line assay counts.
  protein            Read protein sequences, features and source relationships.
  pathway            Read pathway genes and source events.
  article            Read article text, references, citations and files.
  trial              Read trial groups and public documents.
  variant            Read source variant assertions, populations and recurrence.
  sync               Synchronize a local source bundle.
  skill
  source
  version
  mcp-config
  health             Report source terms and bounded reachability.
  store              Manage local cached replies and snapshots.
  key                Register and manage source credentials without showing values.
  serve              Serve MCP over standard input or HTTP.
  schema
  biomarker
  papers
  organization
  assay-count
  gwas
  features
  isoforms
  domains
  complexes
  complex-members
  structures
  sequence
  phenotype
  genes
  events
  study
  pgx
  diagnostic
  adverse-event
  disease
  drug
  gene
  supplement
  pdf
  text
  citations
  references
  citation-evidence
  recommendations
  document
  assertions
  population
  recurrence
  normalize
  help               Describe commands and their arguments.
  network
  sources
  config
  exit               Explain process exit and error codes.
  relations          Map entities through sections and search filters.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help skill install.


## biomcp help source

```text
Usage: biomcp help source [OPTIONS] [COMMAND]

Commands:
  list  List supported discovery topics.

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help source.


## biomcp help source list

```text
List supported discovery topics.

Usage: biomcp help source list [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

List supported discovery topics.


## biomcp help version

```text
Usage: biomcp help version [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Register and manage source credentials without showing values.

- `source`: Credential source to describe.
Listed choices accept any letter case; replies use their listed spelling.

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help schema.


## biomcp help biomarker

```text
Usage: biomcp help biomarker [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help biomarker.


## biomcp help papers

```text
Usage: biomcp help papers [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help papers.


## biomcp help organization

```text
Usage: biomcp help organization [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help organization.


## biomcp help assay-count

```text
Usage: biomcp help assay-count [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help assay-count.


## biomcp help gwas

```text
Usage: biomcp help gwas [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help gwas.


## biomcp help features

```text
Usage: biomcp help features [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help features.


## biomcp help isoforms

```text
Usage: biomcp help isoforms [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help isoforms.


## biomcp help domains

```text
Usage: biomcp help domains [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help domains.


## biomcp help complexes

```text
Usage: biomcp help complexes [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help complexes.


## biomcp help complex-members

```text
Usage: biomcp help complex-members [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help complex-members.


## biomcp help structures

```text
Usage: biomcp help structures [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help structures.


## biomcp help sequence

```text
Usage: biomcp help sequence [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help sequence.


## biomcp help phenotype

```text
Usage: biomcp help phenotype [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help phenotype.


## biomcp help genes

```text
Usage: biomcp help genes [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help genes.


## biomcp help events

```text
Usage: biomcp help events [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help events.


## biomcp help study

```text
Usage: biomcp help study [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help study.


## biomcp help pgx

```text
Usage: biomcp help pgx [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help pgx.


## biomcp help diagnostic

```text
Usage: biomcp help diagnostic [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help diagnostic.


## biomcp help adverse-event

```text
Usage: biomcp help adverse-event [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help adverse-event.


## biomcp help disease

```text
Usage: biomcp help disease [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help disease.


## biomcp help drug

```text
Usage: biomcp help drug [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help drug.


## biomcp help gene

```text
Usage: biomcp help gene [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help gene.


## biomcp help supplement

```text
Usage: biomcp help supplement [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help supplement.


## biomcp help pdf

```text
Usage: biomcp help pdf [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help pdf.


## biomcp help text

```text
Usage: biomcp help text [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help text.


## biomcp help citations

```text
Usage: biomcp help citations [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help citations.


## biomcp help references

```text
Usage: biomcp help references [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help references.


## biomcp help citation-evidence

```text
Usage: biomcp help citation-evidence [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help citation-evidence.


## biomcp help recommendations

```text
Usage: biomcp help recommendations [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help recommendations.


## biomcp help document

```text
Usage: biomcp help document [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help document.


## biomcp help assertions

```text
Usage: biomcp help assertions [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help assertions.


## biomcp help population

```text
Usage: biomcp help population [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help population.


## biomcp help recurrence

```text
Usage: biomcp help recurrence [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help recurrence.


## biomcp help normalize

```text
Usage: biomcp help normalize [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help normalize.


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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Describe help network.

- `setting`: Network setting to describe.
Listed choices accept any letter case; replies use their listed spelling.

## biomcp help sources

```text
Usage: biomcp help sources [OPTIONS]

Options:
      --format <format>
          Choose JSON or Markdown output; configuration can change the default.

          [default: json]
          [possible values: json, md]

      --disable-sources <disable-sources>
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

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
          Exclude source names or source terms groups. Example: --disable-sources civic.

      --source_pacing <source_pacing>
          Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.

      --config <config>
          Read configuration from this file. Example: --config config.json.

      --offline[=<offline>]
          Read local data without source requests.

          [default: false]
          [possible values: true, false]

      --fresh[=<fresh>]
          Request fresh source data instead of cached replies.

          [default: false]
          [possible values: true, false]

```

Map entities through sections and search filters.


