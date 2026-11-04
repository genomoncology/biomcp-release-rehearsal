# BioMCP 1.0 restructure: target architecture and test strategy

Snapshot read: `origin/main` at c0a937899 (2026-10-28), plus the open ticket branches for 0407, 0408, 0413 and 0417. Inputs: `sdlc/planning/architecture.md`, the decisions it cites, `sdlc/records/2026-10-28-code-quality-review.md`, tickets 0407 to 0453, the adversarial review of the first draft, and the code. Ian ruled that this work ships in 1.0 (`decisions/2026-10-28-restructure-before-1-0.md`). Every step keeps the existing cases passing and changes no output unless this document names the change.

This document says what to build and in what order. It records no status. The PM rulings of 2026-10-28 are listed at the end and cited by number as "ruling N".

## 0. Summary of the design

1. Every rule lives in one layer, and a compiler, Cargo or clippy check enforces the placement. The binary parses, calls one query function, spells steps and prints. The query layer owns status, next steps, paging and capability checks. Adapters own wire syntax and source mapping. The transport names no source.
2. One `commands!` table in `biomcp-types` declares each command once. Grammar, decoding, spelling, schema IDs and help derive from it. The query layer returns each reply through a generic `Reply` visitor that ties the body type to the request type, so `QueryBody` and its mismatch arms go.
3. One `SourceDecl` per adapter declares each source once. An exhaustive `match` from `Source` to its declaration replaces every hand-kept source table.
4. One adapter value rule, with shared `Field<T>` and `Problems` helpers, and an audit that sees every drop form.
5. One `Leg::call(endpoint, input, read)` replaces the combination methods.
6. Typed errors: `UsageError` replaces usage bodies, and `QueryError` is the query result's error.
7. Proof runs outside-in against the binary. Coverage counts only product-binary processes. The installed release binary runs the whole suite. Each refactor ticket proves no behavior change with a reply diff between its base build and its branch build.

---

## (a) Layer ownership

### Current shape

The crate graph holds, because Cargo refuses reverse edges, but behavior leaks across layers.

- The binary pairs bodies with requests in `crates/biomcp/src/command.rs` (`response_body`, `response_common`, with `io::Error::other("gwas response mismatch")` fallbacks). `command/steps.rs` orders and caps steps and inserts `help network` configure steps. `command/step_sources.rs` drops disabled-source follow-ups for article and author only. `command/drug_label.rs` trims label fragments until the whole envelope, spelled steps and `--config` prefix included, fits 8,192 bytes. `command/disease.rs` decides section support, rewrites SEER queries (`prepare`, `with_demographics`) and builds SEER selector errors. `command/unsupported.rs` checks cell line databases through `biomcp_adapters::cellosaurus`. `command.rs` and `disease.rs` hide a failed body unless `condition_resolution` is set.
- The binary imports adapter modules directly: `registry` (`restore_outcome`, `records::{RecordKind, declaration, record_url}`, `display_name`, `disabled`, `disabled_uses`), `seer`, `cellosaurus`, `mydisease`, `hgnc`, `guidance` (`deadline`, `WONDER_QUOTA`) and the bundle modules. `config/deferred_keys_tests.rs` uses `biomcp_adapters::alphagenome` and `biomcp_fetch::grpc::alphagenome::DnaseRequest`.
- `biomcp-query` builds provider syntax: PubMed tags in `article/term.rs`, Europe PMC syntax in `article/sources.rs`, ClinicalTrials.gov parameters in `trial.rs`. ChEMBL and Open Targets mapping sits in `drug/native/identity.rs`. Seven adapters export raw modules.
- `biomcp-fetch` names sources: `cache_hgnc.rs`, called from `bundle/transfer.rs:73` on import and from `cache/schema.rs:178` at open; `secret.rs` `SecretName::source()`; `grpc/alphagenome/`.
- `config.rs` rereads the config file on each call (`read_file` at lines 13, 63, 67, 237 and 250), and the grammar builds its clap tree on each parse.

### Target shape

| Layer | Owns | Never holds |
| --- | --- | --- |
| `biodata` | Checked values, identifiers, vocabularies, one `Error` enum with stable codes, their schemas | Commands, sources, I/O |
| `biomcp-types` | `Source`, the `commands!` table and `Request`, query types and checked constructors, response bodies, `Envelope`, `Status`, `Step`, `UsageError`, `ErrorMessage` and the error envelope, the paging window | I/O, serde_json, source declarations |
| `biomcp-fetch` | Requests, quotas, deadlines, retry, decoding, reply checks, cache, record and replay, bundles, the generic gRPC stream, bounded decompression | Any source name, provider messages, provider query syntax |
| `biomcp-adapters` | One folder per source: `SourceDecl`, endpoints, wire syntax, private raw types, mapping with `Problems`, the AlphaGenome codec | Status, next steps, merging, prose other than guidance strings in its declaration |
| `biomcp-query` | `run`, resolution, fan-out, merge, rank, pools and paging, status, next steps with order, caps and source filtering, capability checks, the query echo, body hiding, label bounding, the store | Provider syntax, raw types, clap, argv, file writes |
| `biomcp` | argv, MCP and HTTP edges, config read once per process, the clap tree built once per process, spelling, CLI global prefixes on steps, output format, exit codes, file saves | Any rule about a source, a section, a status or a step order |

Identifier objects use `namespace` and `id`. An unstated authority serializes as an explicit `namespace: "unknown"`; omission never stands for an unknown authority. Namespace spellings and native scalar values remain unchanged. The shared `Identifier<T>` groups an authority with an existing scalar type without adding validation or rewriting the value. Reactome database IDs and ClinPGx numeric IDs keep their native field names and carry that object as the field value. Reactome stable IDs remain separate from database IDs. GWAS association replies use the existing `GwasAssociationId` spelling accepted by the get query. `ArticleKey` already uses the agreed pair and keeps its spelling. Schema metadata marks identifier objects with `x-identifier`; the existing schema contract check validates required names in roots and nested definitions.

Placement rules:

- Wire syntax lives in the adapter function that sends it. The query layer passes typed values to adapter functions.
- Raw types are private to their adapter. Mapping lives in the adapter's `map.rs`.
- Status lives in `biomcp-query/src/status.rs` only (area g).
- Next steps leave each entity's `next.rs` ordered and capped. The binary only spells them and adds the `--config` and `--disable-sources` prefixes.
- Capability checks run in the query layer before any request and return `QueryError::Usage`. This covers unsupported filters and sections, cell line databases with no native mapping, and SEER selector validity.
- The query echo belongs to the entity, so SEER's normalized query stays the echo.
- Display text: help comes from query type docs, fixed messages from `biomcp-types`, guidance strings from each `SourceDecl`, Markdown from the binary.

`biomcp-query/src/lib.rs` exports exactly what the binary uses: `run`, `Ctx`, `Reply`, `Outcome`, `QueryError`, `Action`, `Bundle`, `health::run`, the save downloads (`article::{pdf::download, text, supplement}`, `trial::document::download`), `store::{SchemaOwner, OWNER, health, clean, CACHE_NAMESPACES, CACHE_NAMESPACE_VERSION}`, and one `sources` module. `sources` re-exports what the binary reads about sources today: restored outcomes, record kinds, declarations and URLs, display names, disable names and uses, the key list, deadline guidance and the WONDER quota. `deferred_keys_tests.rs` moves to `crates/biomcp-adapters/tests/`.

### Invariants and checks

| Invariant | Check |
| --- | --- |
| The binary never calls an adapter | `crates/biomcp/Cargo.toml` drops `biomcp-adapters`; Cargo refuses the import |
| The query layer sends no wire syntax | `crates/biomcp-query/clippy.toml` disallows `biomcp_fetch::Arg`, `Endpoint` and `Input` |
| Raw types stay in their adapter | `mod raw` is private; the compiler refuses outside names |
| The transport names no source | A ratchet fails when a `Source` spelling appears as a word in `crates/biomcp-fetch/src`. It starts with an allow list of `secret.rs`, `cache_hgnc.rs`, `bundle/transfer.rs` and `cache/schema.rs`, and 0422 empties it |
| The query surface is its entry points | The export list above; `unreachable_pub`, already denied, flags leftovers |
| Config and grammar are built once | A ratchet allows `config::file::read_file` and the grammar's tree builder to be called only from `main.rs` and the MCP server's setup |

### Migration steps

1. Status rules move into `status.rs` (0419).
2. Typed next steps, order, caps, source filtering and network steps move into the query layer (0458).
3. Wire syntax and drug identity mapping move into adapters; raw modules become private; capability checks, body hiding, label bounding and SEER handling move into the query layer; the binary drops adapters (0420).
4. The AlphaGenome codec moves into its adapter (0423). HGNC re-keying leaves the transport (0422).
5. Config and grammar build once per process (0427).

### Stays unchanged

The six crates and their direction, the store in `biomcp-query/src/store/`, the binary's view, the MCP and HTTP servers, `config.rs` as the only environment reader, the blocking transport and the AlphaGenome worker exception.

---

## (b) One command declaration

### Current shape

Each command is described in eight hand-kept places: `Request` (58 variants, `crates/biomcp-types/src/query/request.rs`), `query_schema` and `QUERIES` in `crates/biomcp/src/grammar/queries.rs`, `request` in `grammar.rs` and `entity::request` in `grammar/entity.rs`, `query_value` and `command_verb` in `spell.rs`, `run` in `crates/biomcp-query/src/run.rs`, `QueryBody` (about 117 variants in `crates/biomcp-types/src/response/gene.rs`), and the binary's `response_body`, `response_common` and per-entity encode modules. The `contracts!` lists in `crates/biomcp-types/tests/schemas.rs` restate schema-to-type pairs.

The grammar splits and rewrites some paths:

- `get pgx` parses with `biomcp.pgx-get-command/1` and splits into `GetPgx` or `GetPgxAnnotation` by the presence of `annotation_id`, renaming it to `id`. Spelling uses `biomcp.pgx-get/1`. The two IDs are deliberate.
- `get drug` splits on `--source who-pq` into `GetWhoMedicine`.
- `get gwas` splits into `GetGwasAssociation` or `GetGwasStudy` by the `association:` prefix, under one schema `biomcp.gwas-get/1` (`grammar/gwas.rs:34`).
- `search gwas` renames its positional query to `gene`.
- `search cell-line` and `search author` receive an injected `query_option` in `request_arguments`.
- Spelling drops defaults: `section_limit` for `GetPgx`, and the default source and `section_limit` for phenotype (`phenotype_value`).

Usage errors travel as `QueryBody` variants. `usage_with_argument` rescans argv text for messages, such as `--facility` with `--lat`. The gwas threshold message comes from parsing `ScientificPValue` out of argv in `grammar/gwas.rs invalid_threshold`. `validation::decode` maps a field error to `QueryInvalid::new(&[&key])` and discards the inner error. Save commands send `TrialDocument`, `ArticlePdf`, `ArticleText` and `ArticleSupplement` to `crate::save::*`, which builds and edits `QueryBody`.

### Target shape

**The table.** One `macro_rules! commands` invocation in `crates/biomcp-types/src/query/commands.rs`, one row per variant:

```rust
commands! {
    /// Search gene headers.
    SearchGene(GeneSearch) = ["search", "gene"], "<gene search schema ID>";
    GetPgx(PgxGet) = ["get", "pgx"], "biomcp.pgx-get/1", via PgxGet;
    GetPgxAnnotation(PgxAnnotationGet) = ["get", "pgx"], "biomcp.pgx-get/1", via PgxGet;
    GetGwasAssociation(GwasAssociationGet) = ["get", "gwas"], "biomcp.gwas-get/1", via GwasGet;
    SearchGwas(GwasSearch) = ["search", "gwas"], "biomcp.gwas-search/1", via GwasSearch;
    // ...
}
```

It generates:

- `enum Request` with the same variants, docs and serde tag, so the request schema keeps its shape. Schema IDs are whatever 0444 leaves. 0444 moves BioMCP-owned IDs out of `biodata.`, and 0421 depends on it.
- `Request::path`, `Request::schema` and `COMMANDS`, one entry per distinct path with its parse schema.
- `enum Rewrite`, one variant per `via` name. The rows that share a `via` name are the variants of one split.

The binary's `grammar/rewrite.rs` holds one exhaustive `match` over `Rewrite` in each direction. Parsing turns the clap argument object into the variant's object: the pgx split and rename, the drug split, the gwas split and positional rename, and the `query_option` injection for cell line and author. Spelling reverses that and drops the defaults listed above. These rules live in one file with no wildcard arm, and the compiler forces a new `via` name to get both directions. Rows without `via` decode with serde from the argument object directly.

**Grammar, spelling and help derive from the table.** `grammar.rs` builds clap by iterating `COMMANDS`. `request`, `entity::request`, `query_schema`, `QUERIES` and `command_verb` go. Help and the MCP command tree read `COMMANDS`. `contracts!` in `tests/schemas.rs` takes its query rows from `COMMANDS`. Its reply rows come from the schema-ID-to-body registry that 0439 adds. `build.rs EXAMPLE_CASES` stays, because it lists help examples, not commands.

**The type link.** The query layer returns each reply through a generic visitor:

```rust
// biomcp-query
pub trait Reply {
    type Out;
    fn body<B: ResponseBody>(self, echo: B::Query, out: Outcome<B>) -> Self::Out;
    fn error(self, error: QueryError) -> Self::Out;
}
pub fn run<R: Reply>(cx: &Ctx<'_>, request: Request, reply: R) -> R::Out;
```

`run` stays one match. Each entity function takes `reply` and returns `reply.body(echo, outcome)`, so the entity owns its echo, which keeps SEER's normalized query. A command whose body type depends on its query, such as study sections, gwas get or drug label, picks the type from the query before fetching. A wrong pairing is a compile error. The binary implements `Reply` twice. The envelope reply builds, spells and serializes generically. The save reply takes the typed body of `TrialDocument` and the article save commands and writes the file. `QueryBody`, `response_body`, `response_common`, the per-entity encode modules and every `io::Error::other("... mismatch")` go. The query layer applies the body-hiding rule in `status.rs` by setting `Outcome::body` to `None`.

**Typed usage errors.**

```rust
// biomcp-types
pub enum UsageError {
    Arguments { argument: Option<Box<str>>, message: Option<ErrorMessage> },
    UnknownCommand { help: [&'static str; 2] },
    Retired(ErrorMessage),
    UnsupportedFilter { source: Source, filter: Filter },
    TooManyPmids { count: u64 },
    PoolMismatch { restart: Step },
    SaveRefused { url: Text },
    McpUnavailable(ErrorMessage),
}
impl From<UsageError> for ErrorEnvelope { /* the one builder of usage envelopes */ }
// biomcp-query
pub enum QueryError { Usage(UsageError), PoolExpired { restart: Action } }
```

`ErrorMessage` stays public, because `retired.rs`, `retired/*.rs`, `command/mcp_policy.rs` and `command.rs` name about 255 of its variants. `QueryInvalid` gains `message: Option<ErrorMessage>`. `validation::decode` keeps the field's parse error and maps it to its message. The `ScientificPValue` parse error gives the gwas threshold message. `TrialSearch`'s checked constructor refuses `--facility` with `--lat` and gives that message, so `usage_with_argument` goes. When a command has several errors, which message wins follows constructor order. The reply diff shows any message that changes.

### Invariants and checks

| Invariant | Check |
| --- | --- |
| Every variant has a path and a schema | The macro requires both |
| A shared path is a declared split | `crates/biomcp-types/tests/commands.rs` asserts that rows sharing a path share a `via` name, and that every schema ID names a checked-in file |
| No fallback arms over commands or rewrites | `#![deny(clippy::wildcard_enum_match_arm)]` in `run.rs`, `spell.rs` and `grammar/rewrite.rs` |
| A body cannot pair with the wrong request | `Reply::body` takes `B::Query`; a mismatch does not compile |
| Usage envelopes have one builder | The other usage constructors on `ErrorEnvelope` become private |

### Migration steps (0421, three commits on one branch)

1. Add `UsageError` and `QueryError`, and route the usage bodies through them.
2. Add the table and `grammar/rewrite.rs`, and delete the restated tables.
3. Add `Reply`, convert entities family by family, then delete `QueryBody` and the encode modules.

### Stays unchanged

Every CLI spelling, the request schema's shape, query echo bytes, error envelope bytes, `retired.rs`, the clap builder, `x-positional`, `x-trailing-section` and the `ResponseBody` trait.

### Rejected

- A proc-macro crate adds a crate for a table that `macro_rules!` expresses.
- Generating `Request` from schema files reverses the direction of truth.
- Keeping `QueryBody` with a generated pairing still needs a mismatch arm.
- Making `ErrorMessage` crate-private fails to compile, because public `UsageError` variants hold it. 0434's dead-item ratchet catches never-built variants instead.

---

## (c) One source declaration

### Current shape

Source facts live in many tables, most with a `_` fallback. `registry::SOURCES` and `AUXILIARY`. Probe chains with `_ => disease_probe` and then `Reason::Rejected`. `bundle_state` with `_ => Ok(None)`. `registry/names.rs::display_name`. `guidance.rs` matches. `command.rs::key_help` with `_ => OncokbApiKey`, which clap's value parser makes unreachable from the CLI. `SecretName::ALL` and `source()` in `crates/biomcp-fetch/src/secret.rs`. `BUNDLE_HELP`, the sync match with `_ => Bundle::Hgnc`, `biomcp_query::Bundle`, spell bundle names and `bundle_source`. `registry::outcome` with `expect`. HGNC knowledge in the transport at two call sites.

`biodata::canonical_enum!` marks `Source` `#[non_exhaustive]`, and `SecretName` carries it at `biomcp-fetch/src/declaration.rs:123`. Outside its crate the compiler then requires a `_` arm on every match.

### Target shape

```rust
// crates/biomcp-adapters/src/decl.rs
pub struct SourceDecl {
    pub spec: &'static SourceSpec,
    pub display: &'static str,
    pub probe: Probe,                                // Get | GraphQl | Custom(fn)
    pub bundles: &'static [&'static dyn BundleDecl],
    pub components: &'static [Component],
    pub auxiliary: &'static [&'static SourceSpec],
    pub guidance: Guidance,
}
pub trait BundleDecl: Sync {
    fn name(&self) -> &'static str;
    fn source(&self) -> Source;
    fn sync(&self, fetch: &Fetch) -> Result<BundleInfo, SourceError>;
    fn state(&self, leg: &Leg<'_>) -> Result<LocalBundleState, RequestError>;
    fn rekey(&self) -> Option<Rekey>;               // HGNC only
}
pub const fn decl(source: Source) -> &'static SourceDecl { match source { /* every variant */ } }
```

Derivations:

- The registry, display names, probes, guidance, components and auxiliary specs all come from `decl(source)`. `Source::ALL` comes from the macro.
- The bundle list, sync words, sync help and the spelled sync step come from the declarations' bundles. Bare `sync` keeps HGNC as an explicit, documented default.
- Key help: `Action::KeyHelp` carries a `SecretName` instead of a `Source`, so the fallback arm goes. The key list derives from `decl(source).spec.key`. `SecretName::source()` leaves the transport. `config()` and `environment()` stay on `SecretName`, because they are the binary's config contract.
- `outcome` reads `decl(source).spec.terms` with no `expect`.
- HGNC re-keying: the query layer's `SchemaOwner` list, which already reaches the transport at open, carries each bundle's `rekey` function. Both the import path and the open-time migration call it. `cache_hgnc.rs` and both HGNC branches leave the transport. Ruling 7 accepts the cost of one decode pass. If the lane finds that store generations 1 to 3 never shipped in a release, it deletes the migration instead and records that in the ticket.
- `Source` is exhaustive after 0426 removes `#[non_exhaustive]` from workspace enums.

### Invariants and checks

| Invariant | Check |
| --- | --- |
| Every source declares every fact | `SourceDecl` has no optional probe, display or guidance; `decl` is exhaustive |
| No fallback over sources | `#![deny(clippy::wildcard_enum_match_arm)]` in `decl.rs`, the health module and `biomcp_query::sources` |
| Bundle names are unique | A test in `crates/biomcp-adapters/tests/` compares the derived list with a literal list |
| The transport names no source | The area (a) ratchet with an empty allow list |

### Stays unchanged

`SourceSpec`, `Quota`, `quotas.rs`, source spellings, terms, disable-sources expansion, health output and key command bytes.

### Schema and user-visible changes

None.

---

## (d) The adapter contract

### Current shape

`mapped.rs` holds `Mapped<T>`, `Problems`, `field_text` and `push_problem`. `raw.rs` holds `OneOrMany<T>` and `Field<T>`. The 0417 branch adds `Problems::field`, `required_field`, `fields`, the native-text helpers and `Field::as_ref`. About 32 adapters do not use these helpers. The audit `crates/biomcp/tests/ratchets/adapter_values.rs` fingerprints only `.ok()` and still has 120 pending sites. `ratchets/source.rs` caps only `mod.rs`, `raw.rs` and `map.rs` at 300 lines. Copies: 29 local `fn bad`, seven openFDA envelopes, two GraphQL alias batchers, two `received_date` parsers and eight XML root searches. The AlphaGenome codec sits in `crates/biomcp-fetch/src/grpc/alphagenome/`, `grpc/generated/` and `grpc/tensor.rs`.

### Target shape

The value rule, from architecture 5.1:

| Field kind | Raw type | On a bad value |
| --- | --- | --- |
| Identity (record ID, requested join key, native admission) | Strict, checked in `map.rs` | The record is refused, the page keeps its other records, and the source page marks `dropped` |
| Protocol (counts, cursors, totals, envelope) | Strict | `bad_reply` |
| Other present value | `Option<Field<T>>` | Omitted with one problem naming the field |
| Collection member | `Vec<Field<T>>` | That member omitted with a problem |
| Absent optional value | `Option<..>` | Absent, no problem |

The source field assignments below distinguish record identity from measurement content.

| Source | Record identity | Independent values and admission |
| --- | --- | --- |
| ClinGen validity | Checked HGNC ID, MONDO ID and native MOI identify one curation record. | Disease label, classification, date, GCEP and report link are optional. A bad identity refuses that curation, while other curations remain. No missing-MONDO fallback is established. [Publisher SOP v11, pp. 10–11](https://clinicalgenome.org/site/assets/files/9851/gene-disease_validity_standard_operating_procedures-_version_11_docx.pdf), [native export](https://search.clinicalgenome.org/kb/gene-validity/download). |
| ClinGen Allele Registry | Checked canonical `@id` and the requested CA ID identify the candidate. A qualified HGVS expression identifies its own nested representation. | Genomic coordinates form one complete definition bound to a reference sequence; multiple coordinate members are subsequences of that definition. A bad member invalidates the complete coordinate projection. A valid HGVS expression can remain. Assembly/chromosome labels, MANE forms and external rs/ClinVar references do not determine canonical CA admission. Bad nested expressions or references refuse only themselves. [Publisher API v0.08.07, pp. 5–7](https://calculator.clinicalgenome.org/redmine/attachments/download/264/AlleleRegistry_0.8.7_api.pdf). The MANE extension is classified by its nested representation role; that older API document does not specify its current wire fields. |
| gnomAD | Requested gene and transcript identify constraint context. Exact requested `variant_id` and dataset identify a variant; population/cohort IDs identify nested rows. | Constraint metrics and variant allele counts are measurements. Nullable `ac`, `an` and filters preserve null as absence. A supplied `[]` remains an empty filter list, without inferred PASS, zero or frequency. Bad filter members omit only those members. Count coherence does not define a GraphQL envelope. A parsed count pair with `ac > an` retains both native integers, adds one dataset/group `counts` problem even without a release, and omits checked frequency; it never guesses which integer is wrong. A VA ALL/subset mismatch remains a separate `va.<dataset>.counts` problem. The current request does not collect `homozygote_count`. [Variant schema](https://github.com/broadinstitute/gnomad-browser/blob/main/graphql-api/src/graphql/types/variant.graphql), [VA schema](https://github.com/broadinstitute/gnomad-browser/blob/main/graphql-api/src/graphql/types/va.graphql), [gene schema](https://github.com/broadinstitute/gnomad-browser/blob/main/graphql-api/src/graphql/types/gene.graphql). |
| LitVar | Native `_id` identifies an autocomplete candidate and its publications lookup. Exact selection additionally requires the valid rsID for an rs query, or valid gene and name for a gene/change query. | Names, rsIDs and publication counts are independent optional annotations. Bad annotations omit only themselves with a named field problem, without broadening exact selection. A malformed or incoherent publication count is omitted while valid PMID records remain; no zero, count repair or guessed source total is supplied. [Publisher LitVar2](https://www.ncbi.nlm.nih.gov/research/litvar2/). |
| MyVariant component provenance | The selected component name identifies the component; source declaration supplies its license terms category. | Native version, copied license URL, upload metadata, aggregator build/date and notes are independent. A bad copied URL omits that link with its named problem; the component name and original declared license terms remain available. Annotation URL validation checks HTTP(S) syntax, preserves native spelling, and does not apply download destination, fragment or IP-host restrictions. [Publisher metadata API](https://docs.myvariant.info/en/latest/doc/data.html#data-source-specific-annotation). |
| AlphaGenome | Requested variant, requested scorer ordinal, tensor coordinate and matching ordered track name/strand identify a scalar; gene-centric rows also require a checked native Ensembl `gene_id`, retaining its complete version when supplied. The proto comment says unversioned, but retained actual producer metadata supplies versioned gene IDs; the string field and checked Ensembl version syntax admit both without stripping or inferred equivalence. | FLOAT32 finite, NaN and infinity values are native measurement content. NaN is documented for strand-mismatched tracks/genes and masked cases; infinity retains its explicit category without an invented interpretation. Optional `nonzero_mean` has no documented nonfinite sentinel and follows the finite metadata contract. Optional gene and track metadata can be omitted independently; bad track/gene identity refuses affected scalars without shifting tensor positions. A structurally valid mismatched variant echo refuses only that scorer’s scalar rows with `scorers.<ordinal>.variant`; valid other scorer groups remain. Paging consumes native tensor positions, including refused identities; returned counts admitted scalars and next advances past the consumed window. Invalid nested ontology identity refuses that ontology object. Shape, dtype, counts, chunks, byte integrity and terminal gRPC status are protocol. [Pinned publisher proto](https://raw.githubusercontent.com/google-deepmind/alphagenome/55943112b83439e0c2512110f1ae607d4b5b492a/src/alphagenome/protos/v1/dna_model.proto), [pinned scoring utilities](https://raw.githubusercontent.com/google-deepmind/alphagenome/55943112b83439e0c2512110f1ae607d4b5b492a/src/alphagenome/models/v1/variant_scoring_utils.py), [Ensembl stable ID versioning](https://jun2026.archive.ensembl.org/info/genome/stable_ids/index.html). |

#### Reviewed interpretation-source fields

| Source record | Identity and actual request joins | Independent values |
| --- | --- | --- |
| CPIC gene | The native symbol must match the requested gene. | Cross identifiers, lookup method and version remain source values. |
| CPIC pair | `pairid` identifies the row. `genesymbol` must match the request. An explicit CPIC level filter must match the native level. | Drug references, names, levels, guideline references, publications and flags remain independent values. |
| CPIC recommendation | `recommendationid` identifies a view row. A table row uses `id` and the exact requested `drugid`. Both preserve the complete native `lookupkey` and its requested gene join. | Guidance, phenotype, activity, population and other clinical context remain native values. A missing lookup component cannot imply a normal gene or broaden applicability. |
| CPIC guideline and diplotype | A guideline uses its native unique `guideline_name` and a checked requested gene in `genes`. A diplotype uses its native gene and the unchanged requested expression. | URLs and result content remain independent. The full joint native lookup and diplotype maps remain intact when valid. |
| ClinPGx annotation | Numeric `id` and `accessionId` identify the record. An accession get checks the requested accession. A gene collection also checks the requested symbol in native `location.genes`. Native class values remain source text. | Locations, evidence, guidance references, scores, phenotype records and history retain independent field boundaries. Chemical logic stays absent when missing; no AND or OR is inferred. |
| ClinPGx class | A guideline reference requires `GuidelineAnnotation`; a label reference requires `LabelAnnotation`. The concrete reference route checks that class and its accession for each row. | Clinical detail class is optional literal metadata. The native detail uses `SummaryAnnotation`; the publisher's summary schema declares no class requirement. Generic object references keep open optional class text. |
| ClinPGx nested record | Native IDs identify locations, evidence terms, object references, cross references, phenotype records and history events. A class, external code or symbol is an admission field only when the operation uses that actual join. | A bad nested identity refuses that member. Other native labels, coordinates, dates, clinical values and flags remain independent values. |
| GenCC submission | `sgc_id` and `version_number` identify the submission. The gene read uses the requested HGNC identity. An invalid HGNC value can be scoped for rejection only through an independently checked native gene symbol. | Disease, classification, inheritance, submitter and date fields remain independent values. A row without either usable gene key has no safe gene-query scope. |
| Erepo document | A summary uses CAid, `PCERDocID` and `docVersion`. A historical read also checks the exact UUID, native document ID, requested version, CAid and document type. | Classification, conditions, criteria and assessment content remain independent values. A bad conclusion keeps document provenance and valid detail without a common statement or invented classification. |
| Cancer Hotspots candidate | The native gene and residue identify the parent row. An alternate amino-acid key identifies its nested member. | Transcript spelling and counts remain independent values. A bad count does not remove a valid row or alternate. Candidate data does not establish an allele or transcript join. |
| CIViC profile and claim | Profile, member, evidence and assertion IDs identify their native records. Requested CAid, profile and therapy selectors check their actual source fields. | Names, links, complexity, HGVS, type, direction, significance and strength remain source values. Missing or bad complexity or member context prevents a derived subject. Bad disease or therapy context prevents a broader normalized claim. |
| OncoKB annotation | The requested genomic variant, assembly and tumor context must match the native query echo. | Gene, protein, flags, labels, effects, citations and release fields remain source content. Bad resolved gene or alteration prevents a derived protein statement. Treatment and implication groups expose their source collection and zero-based native response `ordinal` because the publisher supplies no row ID. Optional evidence levels and independently retained concept IDs and labels cannot erase the group. Native resolved gene and protein values remain independent of the checked genomic request. Missing joint applicability context cannot become a broader statement or a single-drug claim. |
| OncoKB gene | The native gene ID and requested symbol join identify the gene. | The five published gene-type codes describe annotation. The optional `gene_type` retains exact native text; `gene_type_code` is present only for the five published literals. An unknown literal emits one `geneType` problem and omits coding. A future literal establishes no equivalence to a known code, and no missing or unknown code becomes `NEITHER` or `INSUFFICIENT_EVIDENCE`. |
| GTR version and relation | The complete accession and version identify a version and join relation rows. The native object selects the target namespace. Requested gene symbol, GeneID or MedGen selectors check their own native fields; an explicit test-type filter checks that native value. | Lab, name, date, method and status cells remain content. A bad current flag cannot select a current card. Empty cells and independent pipe lists retain their native meaning. Duplicate version conflicts remain conflicts. GeneID and SNOMED values never exchange namespaces. |

CPIC keys and joins follow the publisher's [schema](https://raw.githubusercontent.com/cpicpgx/cpic-data/main/src/main/resources/db/migration/V1__Initial_bootstrap.sql), [views](https://raw.githubusercontent.com/cpicpgx/cpic-data/main/src/main/resources/db/migration/V3__views.sql) and [guideline summary view](https://raw.githubusercontent.com/cpicpgx/cpic-data/main/src/main/resources/db/migration/V71__fix_guideline_summary_view.sql). ClinPGx identity fields follow its [OpenAPI contract](https://api.clinpgx.org/openapi.json). GenCC defines its submission identity in the [export specification](https://thegencc.org/download). Erepo separates document identity from content in the [interpretation model](https://dataexchange.clinicalgenome.org/interpretation/entities/VariantPathogenicityInterpretation.html) and [criterion assessment model](https://dataexchange.clinicalgenome.org/interpretation/entities/CriterionAssessment.html). The Hotspots publisher marks transcript as optional in its [native model](https://raw.githubusercontent.com/cBioPortal/cancerhotspots/master/model/src/main/java/org/cmo/cancerhotspots/model/HotspotMutation.java).

CIViC `rawName` supplies the exact `#VID` Boolean expression. Member records use `civic:VID`; feature records use `civic-feature:FID` and keep their native names. The optional native profile `subject` is independent of CAid membership and attached claims. A candidate remains a candidate and carries no attached claim. A valid single protein subject keeps its existing representation. Source-only members carry identity and optional source-stated forms. Missing or malformed expression context never supplies Boolean logic from a display label. The [profile naming contract](https://docs.civicdb.org/en/latest/model/molecular_profiles/name.html) permits unary exclusions as well as conjunction and disjunction.

CIViC AMP `null` and `NA` are legitimate absence or non-applicability values; `NA` remains literal. A present unsupported or malformed AMP value retains the assertion, emits an `ampLevel` problem and withholds the common statement. It never enters the absent-value significance fallback. The [AMP enum](https://raw.githubusercontent.com/griffithlab/civic-v2/main/server/app/graphql/types/amp_level_type.rb) and [assertion validator](https://raw.githubusercontent.com/griffithlab/civic-v2/main/server/app/validators/assertion_validator.rb) define those distinctions. Native multi-therapy `COMBINATION`, `SEQUENTIAL` and `SUBSTITUTES` are distinct relationships. Their group remains native because the common statement's therapy list cannot represent the relationship. Missing or invalid interaction on a multi-therapy group produces one named problem. A null interaction on a single intact therapy is legitimate. The [therapy model](https://docs.civicdb.org/en/latest/model/evidence/therapy.html) and [evidence validator](https://raw.githubusercontent.com/griffithlab/civic-v2/main/server/app/validators/evidence_item_validator.rb) define that cardinality.

CIViC separates profile identity and claim context in its [profile model](https://docs.civicdb.org/en/latest/model/molecular_profiles/overview.html), [direction model](https://docs.civicdb.org/en/latest/model/evidence/direction.html), [significance model](https://docs.civicdb.org/en/latest/model/evidence/significance.html) and [native evidence fields](https://raw.githubusercontent.com/griffithlab/civic-v2/main/server/app/graphql/types/entities/evidence_item_type.rb). OncoKB defines the query echo in [Query](https://raw.githubusercontent.com/oncokb/oncokb/master/core/src/main/java/org/mskcc/cbio/oncokb/model/Query.java), joint treatment context in [IndicatorQueryTreatment](https://raw.githubusercontent.com/oncokb/oncokb/master/core/src/main/java/org/mskcc/cbio/oncokb/model/IndicatorQueryTreatment.java) and [Implication](https://raw.githubusercontent.com/oncokb/oncokb/master/core/src/main/java/org/mskcc/cbio/oncokb/apiModels/Implication.java), and gene annotation codes in [GeneType](https://raw.githubusercontent.com/oncokb/oncokb/master/core/src/main/java/org/mskcc/cbio/oncokb/model/GeneType.java). GTR defines its native column namespaces and version joins in its [data README](https://ftp.ncbi.nlm.nih.gov/pub/GTR/data/_README.html); the [diagnostic field review](slices/diagnostic.md) records empty cells, independent lists and version conflicts.

Adapters use the helpers 0417 adds. Reply types in `biomcp-types` drop `deny_unknown_fields` under 0437. Adapter raw types refuse it under this rule. The two rules apply to different types and agree.

Shared helpers: a `bad_reply` constructor in `biomcp-fetch`, `openfda.rs` (one envelope), `graphql.rs` (one alias batch), `received.rs`, and `xml_element.rs`.

AlphaGenome moves to `crates/biomcp-adapters/src/alphagenome/`: request, envelope, score limits, tensor message decoding and generated messages, behind `score(leg, request)`. The transport keeps the worker, deadlines, recording, cache admission, `Limits`, the stream decoder trait and `grpc::decompress`, which owns ZSTD. `biomcp-adapters` adds `prost` with its derive feature. The generated code's `#![allow(dead_code)]` becomes an `#[expect]` with a reason under 0434.

### Invariants and checks

| Invariant | Check |
| --- | --- |
| No silent drop in adapters | The audit fingerprints `.ok()`, `if let Ok`, `let Ok .. else` and `filter_map(..ok())` in adapter files, and `unwrap_or_default` in `raw.rs` and `map.rs` and on parse results. The query layer joins the audit in 0420, after 0419, 0458 and 0420 rewrite its modules |
| No `serde(default)` or `deny_unknown_fields` in adapters | `ratchets/source.rs` checks every adapter file |
| 300-line limit | Every adapter file; `ctgov/detail.rs` splits |
| One bad value keeps the reply | One replay case per adapter edits one non-identity value in a real recording |
| One copy of each helper | `ratchets/source.rs` refuses `fn bad`, `fn bad_reply`, `fn received_date` and `fn root` outside each helper's home |

### Migration steps

0417 (ctgov). 0418 (adapters outside the openFDA and NCI families). 0455 (openFDA and NCI). 0423 (AlphaGenome). 0424 (helper copies and variant sections through `fan_out::run`).

### Schema and user-visible changes

Replies that failed whole now return with problems. A reply property that becomes optional is a removal from the required set, which 0437's gate makes a major bump. Each lane lists its bumps.

---

## (e) The transport request API

### Current shape

`Leg` has about 30 public request methods named by option combinations (`json_received_checked_cache`, `post_json_checked_cache` and others). Each builds the private `Decoder` in `xml.rs`. `Decoded<T>` is a six-element tuple. Adapters and the query layer make 174 calls. Paging repeats in 20 `at_offset` and 12 `restart` copies and six `page!` macros.

### Target shape

```rust
pub struct Read<'a, T>;          // the current Decoder, in read.rs, public
impl Read<'_, T> { json(), json_string(), xml(), bytes(), check(f), cache_if(f), counted(f), media(m) }
pub enum Input<'a> { Params(&'a [(&'a str, Arg<'a>)]), Json(JsonBody) }
pub struct Received<T> { pub value: T, pub meta: ReplyMetadata }
impl Leg<'_> {
    pub fn call<T>(&self, ep: &Endpoint, input: Input<'_>, read: Read<'_, T>) -> Result<Received<T>, RequestError>;
    pub fn probe(&self, ep: &Endpoint, input: Input<'_>) -> Result<(), RequestError>;
}
pub trait Absent<T> { fn absent(self) -> Result<Option<T>, RequestError>; }   // replaces json_opt
```

`probe`, the bundle reads, `batch` and `grpc` stay. `Decoded<T>` becomes a named struct. One `Paged` trait with default `at_offset` and `restart` replaces the copies. The `page!` macros merge only if `$defs` names stay those 0438 fixes.

### Invariants and checks

Removing the old methods makes every unconverted call a compile error. The reply diff and the transport's own tests prove the same behavior, and recordings still match because reply keys stay.

---

## (f) Error types and codes

| Crate | Error type | Rule |
| --- | --- | --- |
| `biodata` | `Error`, every code declared once | 0426 folds stray types in, with BioData ask 20 |
| `biomcp-types` | `UsageError`, `QueryInvalid { arguments, message }`, `ErrorEnvelope`, public `ErrorMessage` | Usage envelopes come only from `UsageError` (0421) |
| `biomcp-fetch` | `FetchError`, `RequestError`, `SourceError`, `BadReply`, `InputError`, `RecordError`, `CacheError` | Keep `io::ErrorKind`; recording time is `Option<i64>` (0426) |
| `biomcp-adapters` | none | Returns `RequestError` and problems |
| `biomcp-query` | `QueryError` | Local failures stay in `Outcome::local` |
| `biomcp` | `ConfigError`, `StartupError`, `io::Error` | Every exit code comes from `Status::code()` |

`exit.rs` already maps `Status` exhaustively. The literals `ExitCode::from(5)` at `exit.rs:18` and `main.rs:51-52` go in 0426. A ratchet refuses an integer literal passed to `ExitCode::from`. 0426 removes the grammar's `Result<_, ()>` helpers and `From<()>`, and a text ratchet refuses both in product code. `clippy::result_unit_err` does not fire on the binary's crate-private functions, so it is not the check. `map_err_ignore` stays allowed, because 678 sites drop inner errors on purpose under architecture 8.5.

Internal enums: 0426 removes `#[non_exhaustive]` from every workspace enum, `Source`, `SecretName`, `TrialStatus`, `TrialPhase` and `StudyType` included. The biodata Rust API is unpublished, and under the 0437 ruling Rust readers read replies of their own version, so exhaustive matching costs nothing outside and removes every forced fallback inside. This settles the choice 0452 defers.

Ruling 6 governs removing never-built `ErrorMessage` values in 0434.

---

## (g) Shared status rules and typed next steps

### Current shape

Status choices sit in 101 query files outside `status.rs`. Disease and SEER mark any problem partial, phenotype ignores problems, pathway marks any problem or failed source partial, and disease and phenotype ignore failed secondary sources. Copies: 24 `empty`, 8 `finish`, 3 `page_info` and 7 `run`. Eleven files build steps with `json!` and `from_value(..).ok()`: `pgx/next.rs`, `gwas/next.rs`, `gwas/get/next.rs`, `disease/seer.rs`, `disease/ols.rs`, `adverse_event.rs`, `diagnostic/next.rs`, `pma.rs`, `clearance.rs`, `companion.rs` and `study/page.rs`. `store/pool.rs` uses `json!` and `from_value` on purpose to store values.

### Target shape: status (0419)

`Outcome` moves into `status.rs` with a private `status` field. Entities pass parts, such as section states, source outcomes and the source page's `dropped` flag, to `status::item`, `status::get` or `status::search`, and only those functions set the status. The store restores a stored status through `status::restored`. The compiler enforces the placement, so a text ratchet that cannot tell construction from comparison is not needed.

Rules, from architecture 6.11 and ruling 1:

- Item: complete when resolved, ambiguous or withdrawn with nothing requested failed. Partial when a requested section is unavailable or stale, a secondary source failed, or a record was dropped. Absent when the identity source said not found. Failed when the identity source failed.
- Get: absent when every item is absent, failed when every item failed, partial when any item is partial or failed, complete otherwise.
- Search: complete when every asked source answered, partial when some failed or served stale or a record was dropped, failed when every asked source failed.
- A problem in an optional field changes no status.
- Body hiding: a failed outcome carries no body unless it holds a condition resolution.

### Target shape: next steps (0458)

Each query type used as a step gains a checked constructor. A constructor that can fail returns `Result`, and its error becomes a problem, so no step drops silently. `next::finish(steps, Navigation)` applies the order and caps from `steps.rs`. Each step carries its target source from the entity's `next.rs`, and `finish` drops steps whose source is disabled, for every entity, with no request-to-source table. `Action::NetworkHelp(Setting)` holds the `help network` steps, so `finish` places them as `steps.rs` does today. The filter removes steps only and adds no entry to `sources`. One `run(request)` helper replaces the seven copies. `crates/biomcp-query/clippy.toml` disallows `serde_json::json` and `serde_json::from_value`. `store/pool.rs` carries one reasoned `#[expect]`.

### User-visible changes

No schema change. Statuses change for named cases: disease and SEER replies whose only problems omit optional fields, phenotype replies with a dropped record, disease and phenotype replies with a failed secondary source, and pathway replies whose only problems omit optional fields. Disabled-source follow-ups disappear for every entity. The reply diff lists each case, including the cases 0418 adds before 0419 lands.

---

## (h) Test strategy

### Current shape

- `crates/biomcp/tests/cases.rs` runs 654 YAML cases plus Rust trials, and dispatches parity, live-check, measure, native-proof, installed-proof and generator modes by argv flag. Its default case directory is fixed at compile time from `CARGO_MANIFEST_DIR`. Passing `--cases DIR` skips `proof::trials` and the protocol trials.
- 28 test files read `CARGO_MANIFEST_DIR`, and 26 of them bypass `launch::fixtures()`. 51 runner files call library crates in the harness process, and 25 test files use `#[path]` includes.
- `scripts/coverage.sh` counts harness profiles plus contract targets against `coverage.floor` 90.41.
- `release/smoke.py` runs one copied case and three installed proof modes.
- Feature mapping lives in YAML fields and in `sdlc/features.md` globs. 13 JSON budget files duplicate YAML limits.

### Target shape

1. **Outside-in.** Every behavior test runs the binary as a child process. The runner reads cases and fixtures through `launch::fixtures()` only and registers every trial whatever case directory it reads. The rehearsal runs the whole `cases` target against the installed artifact, with `tests/` copied outside the checkout, on every release target (0429). The installed proof modes become trials.
2. **Coverage counted.** Only profiles written by the `biomcp` binary while `cases` runs count. The launcher gives children `LLVM_PROFILE_FILE=<dir>/bin-%m-%p.profraw`. `coverage.sh` merges only `bin-` profiles with `llvm-profdata merge` and reports with `llvm-cov report --object <biomcp binary>`, because `cargo llvm-cov report` 0.6.17 cannot select one object. Library functions the binary never links fall outside the denominator. 0434's dead-item ratchet finds such code instead. Generated protobuf code is excluded by path. biodata's contract tests do not count (ruling 3). The contract-target collection goes.
3. **Floor.** The PM sets `coverage.floor` from the first binary-only measurement and alone raises it (ruling 4). The gate fails more than 0.2 points below it.
4. **Feature mapping.** The YAML `feature` field names a case's feature. A Rust trial's name starts with `<feature>/`. `sdlc/features.md` keeps one `name: sentence` line per feature. The runner fails at discovery on an unlisted feature or a feature with no test.
5. **Tooling.** Parity, live check, measures, native proofs and generators move to `crates/biomcp/tests/tools.rs` with `test = false`. `release/smoke.py` calls the moved modes from there. `make docs-generate` never edits a case.
6. **Budgets.** Size budgets live only in YAML `max_bytes` and `max_tokens`. The routine gate holds no timing assertion. Wall-time limits come from a measured run plus 15 percent.
7. **Mutation testing.** `cargo-mutants` runs once per release candidate under a 90-minute total cap, and survivors do not block (ruling 5). 0457 adds the tool, measures the cost of one mutant, and sets the per-candidate scope so the run fits the cap. Scope is the files changed since the last candidate, then the shared rule modules. The release owner turns at most ten survivors per candidate into cases, chosen from the rule modules first, and lists the rest.
8. **No behavior change per refactor (0454).** With `BIOMCP_TRANSCRIPT_DIR` set, the runner writes one transcript per step: argument list, surface, exit code and exact stdout.
   - A case step's key is its case path and step index.
   - A trial step's key is the trial name, a hash of the argument list, and a count of earlier identical argument lists in that trial, so parallel spawns keep stable keys.
   - `scripts/reply_diff.py BASE HEAD` prints the first differing pointer per key and exits nonzero on any difference.
   - The mask list starts with the build identity fields: the commit line of `version --verbose`, the executable path printed by `mcp-config` and in help entries, and the version string.
   - Calibration runs two builds at different commits from different checkout paths with no source change between them, and each other differing field joins the mask with its reason.
   - Each refactor ticket requires an empty diff or exactly the cases it names. The diff runs as a `pm job`, not in the routine gate. Ruling 8 leaves the tool's future to 0432.

### Invariants and checks

| Invariant | Check |
| --- | --- |
| The shipped artifact passes the suite | The rehearsal fails on a planted installed binary that prints one changed schema ID |
| Fixtures come from one place | A ratchet allows `CARGO_MANIFEST_DIR` in test code only in `runner/launch.rs` |
| Coverage is outside-in | A planted harness-process query call adds no counted line |
| Every test names a feature | The runner's discovery check, with planted fixtures in `crates/biomcp/tests/ratchets/proof.rs` |
| Refactors keep behavior | The reply diff |

---

## Collisions with other open work

| Open ticket | Shared files | Rule |
| --- | --- | --- |
| 0407, 0408, 0413 | `response/error.rs`, `schemas/error.json`; 0408 also `steps.rs`, `command.rs`, `grammar.rs`, `entity.rs`, `queries.rs`, `spell.rs`, `QueryBody`, `request.rs`, `build.rs`, `tests/schemas.rs` | 0434 waits for all three; 0421 and 0458 follow through the dependency chain |
| 0409 to 0416 | NCI and openFDA adapters, trial, diagnostic, drug and adverse event query modules | 0418 excludes those folders; 0419 waits for 0415 and 0416; 0455 waits for both |
| 0426 (with BioData ask 20) | biodata errors, `grammar.rs`, `config.rs`, fetch | Kept as written on main, with the exit literals, unit-error ratchet and enum choice above added |
| 0436 variant ID spelling | `variant_input.rs`, `response/variant.rs`, variant adapters | 0418's variant-family lane and 0424 rebase on it |
| 0437 open reply schemas | reply derives, schema generator, compatibility gate | 0421 rebases on it; 0418 and 0455 bumps follow its required-set rule |
| 0438 stable `$defs` names | generic response types | 0428 merges `page!` macros only under those names |
| 0439 reply read-back registry | `envelope.rs`, a schema-to-body registry | 0421 takes reply rows of `contracts!` from it and adds no second registry |
| 0444 BioMCP schema namespace | `grammar/queries.rs`, schema IDs | 0421 depends on 0444 |
| 0452 Rust API | biodata enums | 0426 removes `non_exhaustive` from `TrialStatus`, `TrialPhase` and `StudyType` |

## Order

| Order | Ticket | Effort | Depends on |
| --- | --- | --- | --- |
| 1 | 0417 | medium | none |
| 2 | 0454 reply diff | small | none |
| 3 | 0425 | small | none |
| 4 | 0429 | medium | 0454 |
| 5 | 0423 | medium | 0429 |
| 6 | 0434 | medium | 0425, 0407, 0408, 0413 |
| 7 | 0430 | medium | 0429 |
| 8 | 0431 | medium | 0430 |
| 9 | 0456 case content | medium | 0431 |
| 10 | 0418 | large | 0417, 0454 |
| 11 | 0426 | medium | 0434 |
| 12 | 0457 mutation job | small | 0429 |
| 13 | 0419 status rules | medium | 0454, 0415, 0416 |
| 14 | 0458 typed next steps | medium | 0419 |
| 15 | 0420 | large | 0458 |
| 16 | 0421 | large | 0420, 0426, 0444 |
| 17 | 0427 | small | 0421 |
| 18 | 0422 | large | 0421 |
| 19 | 0455 openFDA and NCI value rule | medium | 0418, 0415, 0416 |
| 20 | 0424 | medium | 0418, 0455, 0420, 0423 |
| 21 | 0428 | medium | 0424, 0422 |
| 22 | 0432 | medium | 0428, 0427, 0456, 0457 |

## Dependency and decision changes

1. Architecture 2.1 and the dependency decision: the binary drops `biomcp-adapters`. `biomcp-adapters` adds `prost` with its derive feature (ruled 2026-10-28).
2. Architecture 10.3: only the PM raises the floor (ruling 4), and only binary profiles count.
3. Architecture 6.11: ruling 1.
4. `cargo-mutants` becomes a pinned release tool (ruling 5, ticket 0457).
5. Clippy settings: the query crate disallows fetch request types, `json!` and `from_value`. Module-level `wildcard_enum_match_arm` denials apply where named.

## Every schema and user-visible change

| Change | Ticket | Schema effect |
| --- | --- | --- |
| `TrialStatus` gains `TEMPORARILY_NOT_AVAILABLE`; ctgov keeps rows with bad values | 0417 | Additive value; fields made optional take majors |
| Adapters keep replies with one bad value | 0418, 0455 | Each property leaving a required set takes a major, listed per lane |
| Status rules | 0419 | None; statuses change for named cases |
| Disabled-source follow-ups dropped for every entity | 0458 | None |
| Label bound on the section's own bytes | 0420 | None; changed label pages go to the PM under ruling 2 |
| Invalid UTF-8 key refused | 0426 | None |
| Never-built error messages removed | 0434 | `biomcp.error` major if the gate calls it breaking (ruling 6) |
| Variant sections run in parallel | 0424 | None; section order kept |
| Everything else | all | None |

## PM rulings, 2026-10-28

1. Problem rule: a problem in an optional field leaves the status unchanged; a dropped record makes the reply partial.
2. Label bounding: the query layer bounds the label section by its own serialized bytes, with the constant set so existing label cases keep their fragment counts. The PM accepts or rejects the reply-diff list of any label page that changes.
3. biodata's own contract tests do not count toward the coverage floor.
4. Only the PM raises the coverage floor. Architecture 10.3 changes to match.
5. `cargo-mutants` is adopted as a pinned release tool, run once per release candidate under a 90-minute total wall-time cap. Survivors do not block the candidate.
6. Removing never-built `ErrorMessage` values takes a `biomcp.error` major bump if the compatibility check calls it breaking.
7. The generic HGNC re-key cost of one decode pass on a one-time upgrade is accepted.
8. 0432 decides whether the reply-diff tool stays after the restructuring.
