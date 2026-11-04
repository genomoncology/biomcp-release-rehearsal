# Cell line slice

## Question bank

These questions are reformulated from published records and papers. They are retrieval questions about human health research. They do not ask BioMCP to select an experimental model, predict a treatment response or run an analysis. The accepted answers below identify the facts to preserve; the source capture will establish current values and availability.

| Case | Question | Published source | Accepted answer boundary | Answer status and proof owner |
| --- | --- | --- | --- | --- |
| C01 | Which accession identifies MOLM-13 in the AML paper? | PMID 9305600; [Cellosaurus CVCL_2119](https://www.cellosaurus.org/CVCL_2119) | CVCL_2119, recommended name and source synonyms; no punctuation-based identity invention. | Answered; `cell_line/native.rs` and `cell_line/sections.rs`. |
| C02 | Which exact line does MV4;11 denote? | PMID 12529668; [CVCL_0064](https://www.cellosaurus.org/CVCL_0064) | Native search candidates, followed by exact CVCL_0064 card. | Answered; `cell_line/native.rs` and `cell_line/sections.rs`. |
| C03 | Does Cellosaurus list a FLT3 internal tandem duplication for MOLM-13? | PMID 12529668; CVCL_2119 | Native mutation description, HGNC identifier, zygosity and supporting references; no allele coordinate inferred. | Answered; `cell_line/sections.rs`. |
| C04 | What FLT3 variation does the source list for MV4-11? | PMID 12529668; CVCL_0064 | Independent source mutation record; preserve source zygosity and publication context. | Answered; `cell_line/sections.rs`. |
| C05 | What fusion does Cellosaurus list for MOLM-13? | PMID 9305600; CVCL_2119 | Native KMT2A-MLLT3 record and both gene identifiers, without collapsing a fusion into a single-gene allele. | Answered; `cell_line/sections.rs`. |
| C06 | Which disease identifiers accompany MOLM-13? | PMID 9305600; CVCL_2119 | Native NCIt and ORDO identifiers with their independent labels. | Answered; `cell_line/sections.rs`. |
| C07 | Which curated variations are recorded for OCI-AML3? | [CVCL_1844](https://www.cellosaurus.org/CVCL_1844); PMID 29805321 | Every returned variation with its original source context, including DNMT3A, NPM1 and NRAS when present. | Answered; `cell_line/sections.rs`. |
| C08 | What exact accession and source name does K-562 have? | [CVCL_0004](https://www.cellosaurus.org/CVCL_0004); PMID 29805321 | CVCL_0004 and K-562; a ChEMBL spelling difference does not change identity. | Answered; `cell_line/native.rs` and `cell_line/chembl.rs`. |
| C09 | Which line does the accession CVCL_0005 identify? | [CVCL_0005](https://www.cellosaurus.org/CVCL_0005); PMID 29805321 | NB4 card, without selecting another line that carries NB4 as a synonym. | Answered; `cell_line/native.rs`. |
| C10 | Why can a search for NB4 return another candidate? | [CVCL_8821](https://www.cellosaurus.org/CVCL_8821); PMID 29805321 | Preserve source names and synonyms for SJNB-4 independently; do not choose by name similarity. | Answered; `cell_line/sections.rs`. |
| C11 | Are HL-60 and HL-60(TB) the same accession? | [CVCL_0002](https://www.cellosaurus.org/CVCL_0002), [CVCL_A794](https://www.cellosaurus.org/CVCL_A794); PMID 29805321 | Two separate exact records; no merge from their similar names. | Answered; `cell_line/native.rs` and `cell_line/sections.rs`. |
| C12 | Which source identities accompany MOLM-13? | CVCL_2119; PMID 29805321 | Exact Cellosaurus cross-references for DepMap, ChEMBL, PharmacoDB and other returned resources. | Answered; `cell_line/sections.rs`. |
| C13 | Which Cellosaurus accession is linked to ACH-000362? | CVCL_2119; PMID 29805321 | Source cross-reference search candidates; a selected record must contain the exact requested cross-reference. | Answered; `cell_line/native.rs`. |
| C14 | Which accession is linked to CHEMBL3706573? | CVCL_2119; PMID 29805321 | Exact cross-reference join to CVCL_2119, not a name join. | Answered; `cell_line/native.rs`. |
| C15 | Does the MOLM-13 ChEMBL cell record carry EFO and CLO identifiers? | [ChEMBL cell CHEMBL3706573](https://www.ebi.ac.uk/chembl/explore/cell_line/CHEMBL3706573); PMID 25883136, DOI 10.1093/nar/gkv352 | Preserve optional identifiers as absent when the native record omits them. | Answered; `cell_line/chembl.rs`. |
| C16 | How many ChEMBL assays name the MOLM-13 cell record? | CHEMBL3706573; ChEMBL API documentation | Native assay page total for exact cell_chembl_id, with release metadata; no inferred activity count. | Answered; `cell_line/chembl.rs`. |
| C17 | Can ChEMBL find K-562 by its Cellosaurus accession? | [ChEMBL cell CHEMBL3308378](https://www.ebi.ac.uk/chembl/explore/cell_line/CHEMBL3308378) | Every record returned by exact cellosaurus_id CVCL_0004; verify each returned identifier. | Answered; `cell_line/chembl.rs`. |
| C18 | Which PharmacoDB cell identity claims MOLM-13's accession? | PMID 34850112; Cellosaurus PharmacoDB cross-reference MOLM13_950_2019 | Exact UID request, native integer ID and accession_id must match CVCL_2119. | Unanswered; deferred with PharmacoDB until an affirmative database reuse grant exists. |
| C19 | Which published datasets contain MOLM-13 experiments? | PMID 34850112 | Source experiment IDs and dataset names, with count scope explicit; no sensitivity classification. | Unanswered; deferred with PharmacoDB until an affirmative database reuse grant exists. |
| C20 | Which published metrics exist for PharmacoDB cell 1248 and compound 53572? | PMID 34850112 | Native experiment identities, dataset, compound and cell IDs, AAC, IC50, EC50, Einf, HS and DSS1 when present. No units or threshold invented. | Unanswered; deferred with PharmacoDB until an affirmative database reuse grant exists. |
| C21 | How does PharmacoDB represent repeated experiments on the same pair? | PMID 34850112 | Preserve distinct experiment IDs and dataset context; never average or deduplicate by compound and line names. | Unanswered; deferred with PharmacoDB until an affirmative database reuse grant exists. |
| C22 | What should a request return when a source has no cell record? | Cellosaurus and ChEMBL API contracts; technical control | Source-local empty result only after a valid reply; it does not establish that the cell line does not exist. | Answered; `cell_line/native.rs` and `cell_line/chembl.rs` prove native absence; `cell_line/faults.rs` and `cell_line/chembl/faults.rs` distinguish failures. |
| C23 | What should happen when a source fails or returns an HTML verification page? | API contracts; technical control | Failed source outcome with no fabricated empty biology; no retry through verification. | Answered; `cell_line/faults.rs`, `cell_line/chembl/faults.rs` and `cell_line/all.rs`. |
| C24 | What source version supports a cell-line answer? | PMID 29805321; PMID 34850112; provider release endpoints | Source release when supplied; otherwise retrieval time explicitly labelled as such. | Partial; `cell_line/native.rs`, `cell_line/sections.rs` and `cell_line/chembl.rs` prove Cellosaurus and ChEMBL provenance; PharmacoDB remains deferred. |

All proof-owner paths in the question bank are relative to `crates/biomcp/tests/runner/`. These existing outside-in files own the design case IDs C01 through C24. C18 through C21 have no executable data case because source eligibility forbids PharmacoDB calls; `cell_line/all/controls.rs` proves the explicit deferred guidance. Answered means the retained source facts satisfy the retrieval boundary, without a clinical interpretation. C24 remains partial across the full question because PharmacoDB provenance is unavailable.

## Investigation boundary

The investigation captured Cellosaurus and ChEMBL directly, plus one bounded HPA identity-contract reply, before proposing shared types. PharmacoDB data capture and implementation remain deferred under the PM source-eligibility ruling. Legacy execution remains unmeasured until network containment is independently verified. Raw replies and licensed bodies stay outside Git. No product implementation belongs in this change.


## Legacy accounting

The reviewed source tree is the 0.9 implementation in `biomcp`. Runtime output size, latency and warm-cache behavior remain unmeasured because this investigation has not established independent network containment for that executable. Source files and old recordings describe old behavior; they do not prove current direct-provider behavior.

| 0.9 command, option, section or behavior | 1.0 disposition | Reason and replacement |
| --- | --- | --- |
| `search cell-line QUERY`, `--query`/`-q`, `--limit`/`-l`, `--offset` | `search cell-line QUERY`, `--limit`/`-l`, `--offset` | Preserve the literal source query and native candidate order. Default five headers; checked positive bounds. |
| Name normalization, removal of punctuation, a second hyphenless search | Retire by PM ruling | Text rewriting can join different lines. One unchanged native field query supplies candidates. |
| Human-first and identifier/synonym match reranking | Retire by PM ruling | Publish species and source names without introducing an ordering or declaring exact identity from normalized text. |
| `get cell-line CVCL_…` | `get cell-line CVCL_…` | Resolve the exact primary accession and verify the returned primary accession. Secondary accession resolution remains deferred until a retained native card proves it. |
| Reverse get from DepMap, Cell Model Passports, ChEMBL and PharmacoDB IDs | `get cell-line SOURCE:ID` | Verify the exact database and accession in the native cross-reference before returning a selected identity. Multiple records remain ambiguous. |
| `variants` | `get cell-line ID variants` | Preserve every gene member, reference, description and source zygosity; a free-text description is not a normalized allele. |
| `xrefs` | `get cell-line ID xrefs` | Do not replace unobserved links with synthetic empty arrays for a fixed seven-source list. |
| `chembl` | `get cell-line ID chembl`; separate `cell-line assay-count CHEMBL_ID` | Cell line metadata and assay count require separate requests and disclosure actions. A single section must not hide the count request. |
| `drug_response` | Defer under the PM source-eligibility ruling | No PharmacoDB data call or implementation until affirmative database reuse terms are found. A future exact UID/accession join cannot use a name fallback. |
| `all` | `get cell-line ID all` | Return every named section, one ChEMBL metadata page and no assay request. |
| `cell-line drug-response ID --dataset D --limit N --offset N` | Defer with PharmacoDB source eligibility | Old dataset filtering occurs after downloading every experiment for the line. A small rendered page does not bound source transfer. |
| `drug cell-lines NAME --cell-line ID/--dataset D --limit/--offset` and drug `cell_lines` section | Dropped under the prior drug-slice ruling | The drug slice settled retirement. A PharmacoDB-local compound ID cannot become a resolved drug identity through name matching. Any replacement must take exact provider identifiers and preserve this distinction. |
| `gene cell-lines SYMBOL --group GROUP --limit/--offset` | Defer pending exact cell identity proof | The gene slice moved this capability here. Its old HPA-to-Cellosaurus name join is forbidden. One bounded source contract capture is authorized; no whole expression download or local analysis. |
| Global JSON/Markdown, fresh/offline/record/replay, source disable and terms controls | Shared grammar | Do not add separate entity-specific transport or formatting options. |

The closed public-issue inventory reads every closed issue returned by the repository's full issue listing, then checks title and body for cell-line, Cellosaurus, PharmacoDB and ChEMBL references. No closed issue requests or reports a cell-line behavior. Issue [220](https://github.com/genomoncology/biomcp/issues/220) mentions an unrelated ChEMBL drug link in a missing drug-interactions report; that issue is not applicable to cell identity or screening. Internal legacy tickets 1202, 1205, 1213 and 1214 explain implementation history and are not public user issues.

<!-- parity-accounting -->
| 0.9 leaf command | Command outcome | Ported or renamed options | Dropped options | Deferred options | Undecided options | Ruling |
| --- | --- | --- | --- | --- | --- | --- |
| `search cell-line` | Ported or renamed | `--limit`, `--offset` | `--query` | — | — | Use literal positional text instead of -q/--query; keep native paging. Drop normalization and reranking because text rewriting can join different lines and invented order can imply identity. |
| `get cell-line` | Ported or renamed | — | — | — | — | Keep exact primary accession and verified source-ID candidate resolution. Keep variants, xrefs, chembl and bounded all. The drug_response section defers PharmacoDB until affirmative database reuse terms exist. New names and identity sections disclose bounded source facts; assay-count remains a separate explicit request. |
| `cell-line drug-response` | Deferred by ruling | — | — | `--dataset`, `--limit`, `--offset` | — | Deferred with PharmacoDB until an affirmative database reuse grant exists. No name fallback, experiment download or local response inference runs. |
<!-- /parity-accounting -->

The detailed rows above account for every legacy behavior, option and section, including global controls and the prior drug retirement. Leaf counts exclude positionals, aliases, sections and shared globals under the existing inventory. The gene slice owns the `gene cell-lines` leaf row; its HPA route remains deferred until an exact cell identity join exists. Shared JSON output uses `--format json`, Markdown uses `--format md`, and freshness, offline/cache, source-policy and record/replay controls keep the existing shared grammar. No second entity transport or renderer exists. Every 0.9 runtime size and timing remains unmeasured because independent network containment for that baseline has not been established. Milestone 5 owns the contained comparison.


## Identity and scientific boundaries

Cellosaurus owns the cell-line identity. Other sources retain independent identifiers and assertions. ChEMBL joins by its native `cellosaurus_id`; PharmacoDB must return the requested UID and the selected CVCL accession. A missing cross-reference does not authorize a name lookup fallback. Disease rows preserve NCIt and ORDO identifiers independently. A gene row must carry a source HGNC identifier before a gene pivot. A free-text mutation or fusion cannot become a normalized variant identifier. A PharmacoDB compound remains a provider-local compound unless its reply provides a separately verified exact drug identifier.

Clinical review accepted the question boundaries after the ChEMBL citation correction. Cellosaurus currently gives MV4-11's FLT3 internal tandem duplication an unspecified zygosity; PMID 12529668 reports expression of the mutated allele only. Preserve both source contexts and do not infer homozygosity. Cellosaurus describes MOLM-13 as heterozygous. Published screen metrics never establish a patient's treatment response, and their presence supplies no threshold, unit or sensitive/resistant classification.


## Direct source findings

The source capture ran on the M5 on 2026-10-26 UTC. Job `j14318` used the accepted `make source-capture` helper at commit `d11e218f93ce674564efbe73924684b2222982f3`: 18 Cellosaurus requests, seven ChEMBL requests and one HPA request. Every response was HTTP 200, no redirect or retry occurred, and every reply stayed within the declared 25-second deadline and 8 MiB cap. The plans paced calls by half a second. These are cold direct-provider observations under shared machine load, not a performance benchmark or warm-product measurement.

| Source request | Native observation | Requests, body size and elapsed time |
| --- | --- | --- |
| Cellosaurus exact cards | All nine returned the requested primary CVCL accession. MOLM-13 and MV4-11 each carry two variation rows; OCI-AML-3 has three; U-937 has five. Xrefs are unbounded in practice: K-562 carries 1,398, versus 13 for SJNB-4. | One request per card; 6,141–325,557 bytes; 0.366–0.876 seconds. |
| Literal MOLM13 and MV4;11 searches | Each returned the expected accession as its sole native candidate. These literal source queries suffice without normalization or an alternate spelling call. | One request each; 2,121/2,216 bytes; 0.364/0.360 seconds. |
| Literal NB4 search, `rows=5` | Source order begins CVCL_8811, CVCL_U085, CVCL_U087, CVCL_U088, CVCL_U081. These are derived lines. Exact NB4 CVCL_0005 is not in this first window. A candidate is never chosen or backfilled by a text heuristic. | One request; 7,927 bytes; 0.367 seconds. |
| Literal KG1 search | Three candidates in native order: CVCL_E3VV, CVCL_0374, CVCL_UD72. The first and third carry mouse taxonomy; CVCL_0374 is the human KG-1 record. Species is therefore necessary in every header. | One request; 3,981 bytes; 0.369 seconds. |
| Exact ACH-000362 cross-reference search | Returned CVCL_2119. The search projection does not include cross-references; a reverse get still needs the exact card's cross-reference check before it selects identity. | One request; 2,121 bytes; 0.366 seconds. |
| Unknown literal Cellosaurus search | A valid empty `cell-line-list`, distinct from a failed source. | One request; 51 bytes; 0.361 seconds. |
| Cellosaurus release | Version 57.0, updated 2026-10-22, 169,026 cell lines and 29,992 publications. Dataset counts are not search totals. | One request; 205 bytes; 0.359 seconds. |
| ChEMBL CVCL_2119 | One exact accession row: CHEMBL3706573; EFO and CLO absent; native organism taxon 9606. | One request; 423 bytes; 0.929 seconds. |
| ChEMBL CVCL_0004 | One exact accession row: CHEMBL3308378; name K562; EFO_0002067 and CLO_0007059. A source spelling difference needs no name join. | One request; 428 bytes; 0.585 seconds. |
| ChEMBL CVCL_9999 | Valid empty `cell_lines` and native total zero. This proves source-local absence only. | One request; 108 bytes; 0.727 seconds. |
| ChEMBL assays for CHEMBL3706573, `limit=1` | Native total 1,408. First assay CHEMBL939219 repeats the requested cell ID and describes apoptosis induction. This is an assay count, not an activity, experiment or drug-sensitivity count. | One request; 1,073 bytes; 1.084 seconds. |
| ChEMBL status | ChEMBL_37, release date 2026-05-01. The release metadata request is separate from cell metadata and assay-count requests. | One request; 212 bytes; 2.509 seconds. |
| HPA exact FLT3 Ensembl query, leukemia column group | One gene row has ENSG00000122025 and 93 source-named nTPM columns. No CVCL accession, cell-source ID or per-column identity cross-reference appears. Do not attach those values to cell identities by parsing names. | One request; 3,664 bytes; 0.793 seconds. |

U-937's Cellosaurus record carries a qualified stock-contamination warning. It states that some initial ATCC stocks were contaminated and that the problem was corrected in subsequent stocks. The complete source warning belongs on the default card. BioMCP must not turn that historical, stock-qualified warning into a present assertion about every U-937 stock.

The variation rows preserve `variation-type`, optional `mutation-type`, `mutation-description`, optional `variation-note`, optional `zygosity-type`, `xref-list` and `source-list`. A source can be a PubMed reference, an institution or a database cross-reference. OCI-AML-3's NRAS row has database provenance without a PubMed reference. A type requiring a PMID would lose valid source facts. U-937's TP53 row has a splice-donor note. Its WT1 description carries several native coordinate spellings without a reference transcript. None becomes a normalized allele, reference sequence or assembly through inference. The ClinVar accessions in these rows are explicit cross-references and remain independently named.

### Source paging

A new question arose when the five-row NB4 window omitted NB4 itself. The authorized first-party OpenAPI read in `j14323` proves native `start` and `rows` parameters. The default order is provider relevance when `sort` is absent. JSON supplies no total or continuation field. A page uses native `start=offset`, `rows=limit`, preserves source order and offers the next offset only when the window fills. A full window means more rows may exist, not a known total. A short window establishes the end of that observed request scope. No command fetches an extra page to backfill an exact-looking name. The separate authorized `start=5,rows=5` NB4 probe in `j14325` returned CVCL_F5R0, CVCL_U080, CVCL_U084, CVCL_U082 and CVCL_VV77, with no overlap with the first five. Both windows fill, so the total remains unknown and the exact NB4 card is still obtained by its accession, not by inventing a preferred result.

### Terms, source eligibility and bundles

| Source | Use and group | First-party terms and decision | Bundle finding |
| --- | --- | --- | --- |
| Cellosaurus | `open`, no restriction group | [Publisher description](https://www.cellosaurus.org/description.html) explicitly assigns CC BY 4.0. Attribute Cellosaurus and its cited publication; keep source dates. | Captured [directory](https://ftp.expasy.org/databases/cellosaurus/) lists text 117M, OBO 113M and XML 642M, dated 2026-10-22. These are rounded listing sizes, not measured downloads. Keep API access; no first slice question needs a whole database bundle. |
| ChEMBL | `open`, no restriction group | [Publisher](https://www.ebi.ac.uk/chembl/) assigns CC BY-SA 3.0. Preserve attribution and ShareAlike terms. | Captured [directory](https://ftp.ebi.ac.uk/pub/databases/chembl/ChEMBLdb/latest/) lists SQLite archive 5.4G, PostgreSQL 1.9G and MySQL 2.0G. Rounded listing sizes only; no download. Whole compound databases are disproportionate to exact cell metadata and one count. Keep API access. |
| HPA | Existing `open` source, third-party caveat retained | [Publisher licence](https://www.proteinatlas.org/about/licence) assigns CC BY 4.0 to copyrightable database parts and retains third-party constraints. The one authorized contract read does not establish an exact cell identity join. | No bundle downloaded or proposed. The moved gene expression capability remains deferred until exact identities can be proved. |
| PharmacoDB | Unknown data terms; no implemented source or output group yet | The PM deferred captures and implementation until an affirmative database reuse grant exists. [Public about page](https://pharmacodb.ca/about) supplies no readable policy text. The [first-party repository](https://github.com/bhklab/PharmacoDB) assigns GPL-3.0 to the project, not an explicit database grant. The [site repository](https://github.com/bhklab/PharmacoDB-JS) documents setup without data terms. The describing paper's NC licence covers its article. No data call was made. | Unknown size and cadence, deliberately unmeasured while source eligibility is unresolved. No bundle proposed. |

The retained Cellosaurus API release date and FTP listing agree. Its cadence is observed only as successive releases, not a promised schedule. ChEMBL's status gives its current release and date; the directory has separate file modification dates. Neither provider's capture establishes a future update interval. The design therefore revalidates version metadata through ordinary cache rules and never invents a fixed data freshness claim. No new source needs a key.

## Delight questions

1. What does the agent ask first? “Which line is MOLM13?” One source-native `search cell-line MOLM13` returns selectable CVCL headers. A known accession goes directly to `get`.
2. What must a header contain? Exact accession, recommended name, category, capped species and disease references. KG1 proves why species cannot be hidden. Rank never establishes identity.
3. What can the agent trust without another call? An exact card's identity, source dates, category, age, sex, native disease and species references and qualified warnings. Source-local omissions remain absent.
4. What comes next? Named `names`, `xrefs` and `variants` pages; exact ChEMBL metadata; then a separate assay-count command using the returned ChEMBL cell ID. No eager count or screen query.
5. Which joins are safe? Exact CVCL accession, database-qualified xref identity, returned ChEMBL `cellosaurus_id`, HGNC gene references and explicit PubMed references. No PharmacoDB or HPA name join is available.
6. How are disagreement and missing fields shown? Separate source records, optional source fields and source-specific outcomes. Preserve the MV4-11 zygosity discrepancy rather than choose a consensus.
7. How are empty and failed distinguished? A valid empty native list completes as empty. A timeout, bad body, unexpected identity, HTTP failure or verification page fails its source and gives a fresh action; it never states absent biology.
8. How can the agent stop early? The card answers identity and disease; `variants` answers the curated variation question without ChEMBL; the assay count answers coverage without an assay listing or activity download.
9. What should not be copied from 0.9? Name rewriting, human-first reranking, name-based joins, unbounded `all`, implicit fan-out and dataset filters that download every screen row first.
10. What is not measured? Warm product latency, actual product envelopes, real tokenizer counts and 0.9 runtime comparisons. Native cold timings and offline projected payload byte measurements are the available evidence.

## Proposed model and schemas

The capture precedes these proposed types. `biodata` holds a validated Cellosaurus accession, source-qualified cross-reference, optional Cellosaurus entry dates/version, cell identity, species/disease references and the original variation records. It performs no I/O. Source categories and variation labels remain native strings unless the source contract proves a closed vocabulary; the sample's labels alone do not establish one. A variation's member genes and provenance retain multiplicity. Required identity fields are validated before construction. Optional absence never means not fetched.

`biomcp-types` owns separate `CellLineHeader`, `CellLineCard`, names, identity-detail, xrefs and variation pages, ChEMBL metadata and an exact ChEMBL assay-count reply. Each has a generated schema and a bounded response projection. Header lists use `Capped`; card collections and sections retain totals. A missing requested section is never a stand-in for a source that omitted a field. An unrequested section appears in `more`. Each requested section carries its state. A warning is complete text with source scope, not a classifier invented from its words.

| Response layer | Fields and boundary |
| --- | --- |
| Header | Primary CVCL accession, recommended name, category, species references and disease references capped at three each. No variation or cross-reference fetch. |
| Default card | Header facts, source-created/updated/version values, age and sex when supplied, complete qualified warning entries capped at three, and `more`. Synonyms and all cross-references stay behind named pages. |
| Identity detail | The same species, disease and qualified warning arrays when a card collection exceeds three; independent native totals and section offset retain every record. This is a metadata page, not another entity join. |
| Names | Five native name entries by default, preserving identifier/synonym type and source order; exact native total and section offset. |
| Xrefs | Five complete source cross-reference entries by default, preserving database, accession, optional label/URL/category/IRI and native order; exact native total. K-562's 1,398 entries are never default-card data. |
| Variants | Three complete native variation rows by default, with every gene member, optional note and native provenance row intact; total counts variation rows, not alleles or genes. No row is shortened to fit a budget. |
| ChEMBL metadata | Exact matching cell records with native organism/taxon, tissue, description, optional CLO/EFO and ChEMBL IDs. Null identifiers stay absent. A record with the wrong CVCL accession fails the join. |
| ChEMBL assay count | Exact ChEMBL cell ID and native `page_meta.total_count` from one `limit=1` assay request. Validate any returned row's cell ID. No activity, metric or treatment interpretation. |

The first implementation uses API requests and the existing checked reply cache. It adds no bundle, pool, analysis engine or new shared transport. Local section paging slices the one complete checked identity-source record already returned; it does not make the source transfer small. Source and response byte limits remain separate. An oversized or broken body fails instead of yielding silently truncated collections.

## Proposed command tree and next steps

| Command | Default answer, options and source work | Next actions |
| --- | --- | --- |
| `search cell-line QUERY` | Five native headers, positional text, `--limit` 1–25 and `--offset`; one literal escaped `idsy` source query with native `start`/`rows`, no sort or rewrite. A canonical CVCL input uses exact accession query syntax. | One disambiguating get per shown CVCL accession; a page action when the window fills; no automatic winner. |
| `get cell-line CVCL` | Identity card from one projected Cellosaurus request; `names`, `identity`, `xrefs`, `variants` and `chembl` are closed named sections. `identity` offers the remaining species, diseases and qualified warnings if a default collection is capped. `--section-limit` accepts 1–25 and `--section-offset` accepts a nonnegative offset; they page requested collections independently with source totals retained. | At most five priority actions under architecture 6.10: names, variants, xrefs, ChEMBL, then identity overflow if present; otherwise a supported exact gene/article pivot when available. |
| `get cell-line SOURCE_ID` | Search the exact qualified external ID, then verify that database and accession on the selected exact card. Multiple candidates stay ambiguous. Neither a sole search result nor a name match bypasses the xref check. | Candidate gets on ambiguity; exact card sections only after identity is verified. |
| `get cell-line CVCL chembl` | One additional exact `cellosaurus_id` metadata request. The response's ChEMBL IDs are source-local cell identities, not compounds or targets. | `cell-line assay-count CHEMBL_CELL_ID` for each exact returned record within the architecture bound. |
| `cell-line assay-count CHEMBL_CELL_ID` | One assay request with exact `cell_chembl_id`, `limit=1`, `offset=0`; publish the count alone and source attribution. | Exact source cell URL; no activity or assay-analysis helper. |

Cellosaurus release metadata is an independent cacheable provenance read, never disguised as part of the one-request card or section claim. A failed release read leaves retrieval time explicitly labelled and a provenance problem rather than a fabricated release. ChEMBL status metadata follows the same rule. Transport trace counts every request, including cold release metadata. Source dates and retrieval dates retain separate meanings.

A card cannot embed another complete entity. Variation gene identifiers may give `get gene HGNC:…` pivots, and exact PubMed references may give `get article PMID` pivots. Their count follows architecture 6.10; all original identifiers remain in the section row even when the next-step bound omits a pivot. Disease references remain source-qualified links unless the existing disease grammar supports that exact identifier. No compound name, free-text HGVS or HPA column becomes a pivot by inference.

## Acceptance case map

| Surface | Harvested positive and boundary cases |
| --- | --- |
| Search positional input | C01/C02 literal spellings; assert direct native candidate IDs and order. No second altered request. |
| Search `--limit`, `--offset` | NB4 native pages, source order and retained query; zero total claim on a full window, no repeated source rows introduced by the client. KG1 retains species. |
| Exact accession get | C01, C06, C08/C09; requested and returned primary or explicitly declared secondary accession agree. Different accession is a failed identity check. |
| Reverse source-ID get | C13/C14; exact database/accession verification, ambiguous candidates, valid unmatched identity and wrong-xref controls. |
| Names and section paging | C01/C02/C10/C11; preserve punctuation and source name kinds, valid bounds, total and continuation arguments. No name-derived identity. |
| Xrefs and section paging | C12/C13/C14, K-562 large list and SJNB-4 small list; source total and complete selected rows, no invented empty source keys. |
| Variants and section paging | C03–C05/C07; fusion multiplicity, unspecified zygosity, mutation note, institution/xref-only provenance, absent optional fields and oversized row failure. |
| ChEMBL metadata | C15/C17; MOLM-13 absent EFO/CLO, K562 spelling and present IDs, exact accession positive, wrong identity, valid zero and disabled source. |
| Assay count | C16; exact cell ID, count 1,408 on the capture, first-row cell identity, zero page, wrong-cell row and source failure. Never relabel total as activities. |
| Qualified warning | U-937 source text preserves “some”, historical stock context and correction; no blanket contamination assertion. |
| Terms and versions | C24; Cellosaurus 57.0 and ChEMBL_37 independent dates, failed release read, skipped/disabled source and use/terms metadata retained. |
| Error controls | C22/C23; recorded 429, 500, timeout, malformed JSON, identity mismatch and HTTP 200 HTML each produce source failure, not empty results. No live outage is manufactured. |
| Retired or deferred surfaces | PharmacoDB screening/compound name routes and HPA name joins fail before provider I/O with the PM-approved guidance. |
| CLI/MCP/Markdown | Shared grammar and envelope parity for every kept command, canonical URLs and exact typed next arguments; no shell command strings. |

## Ian's questions

None. The PM owns source eligibility, defaults, budgets and the proposed legacy retirements. PharmacoDB remains deferred pending affirmative database reuse terms. HPA's moved expression capability remains deferred pending an exact cell identity contract. This design authorizes no tool execution, external screening pipeline, paid source, service publication or release.


## Measured disclosure budget

Complete emitted envelopes replace the investigation's offline payload projections. CLI bytes include the newline; MCP bytes count actual tool text; Markdown counts actual UTF-8 stdout. Separate full RPC entries do not become content budgets. The canonical YAML steps own their `max_bytes` and `max_tokens`; the existing exact Rust phases own their measurements in `tests/cell-line-command-budgets.yaml`. Each ceiling adds fifteen percent to that same case and surface. Source policy fields count in the complete envelope.

| Measured envelope | Owning phase | CLI bytes | MCP text bytes | Markdown bytes |
| --- | --- | ---: | ---: | ---: |
| One native search header | `native/search/MOLM13` | 1589 | 1588 | 1620 |
| Exact default card with qualified warning | `native/card/CVCL_0007` | 2077 | 2076 | 2199 |
| Identity overflow page | `sections/bounds/identity/1/1` | 894 | 893 | 982 |
| Native names page | `sections/native/CVCL_0064/names` | 767 | 766 | 760 |
| Complete xref page | `sections/native/CVCL_0002/xrefs` | 1592 | 1591 | 1517 |
| Complete variation rows | `sections/native/CVCL_0002/variants` | 2951 | 2950 | 3120 |
| Exact ChEMBL metadata | `chembl/native/metadata/CVCL_2119` | 719 | 718 | 807 |
| Exact ChEMBL assay total | `chembl/native/assays` | 376 | 375 | 388 |
| Every bounded first section page | `all/native/CVCL_2119` | 6903 | 6902 | 7000 |

These rows reuse the accepted native measurements from 0258 through 0261. The detailed sections below retain every measured phase and ceiling. The K-562 canonical walk retains its own measured largest emitted page plus fifteen percent and proves all 1,398 exact identities in native order. It uses no other case's maximum. The strict `full_output_bytes` step option counts original stdout and actual validated MCP content text without subtracting source terms. Other entities keep their established basis.

Bytes divided by four remain an estimate rather than tokenizer output. Source transfer sizes and source latency remain separate from product envelopes. Product cold/warm timing and all 0.9 sizes and timings remain unmeasured; milestone 5 owns the independently contained same-question comparison. No provider call or bioinformatics tool run was added for this accounting.


## Reproducible evidence

Yellow holds `data/biomcp-redux/0196-cell-line-investigation/cell-line-investigation-evidence.tar.gz`, SHA-256 `fdfdf0923a9d6cc0e54f306e1dad5b8443010a0a77499d2fe50e6b2ed2a0135b`, 270,106 bytes. Its 69 members include 68 manifest-hashed files, the fixed capture plans, source receipts, original and portable native archives, the closed-issue inventory, projection recipe and all 46 projected payloads with measurements. Source jobs `j14318`, `j14323` and `j14325` delivered 28 total requests. No PharmacoDB data request, product command, 0.9 command or external analysis tool ran.

The initial macOS archives contain AppleDouble metadata companions. Job `j14332` repacked the unchanged source directories with `COPYFILE_DISABLE=1`; it made no provider request. The final archive retains both forms and the original witness. Its portable source archives have 57, 24, 6, 6 and 6 exact members for Cellosaurus, ChEMBL, HPA, OpenAPI and paging respectively. Independent Yellow verification checked the complete outer member set and all 68 hashes, the portable member sets and 94 payload hashes, and the identical 94 original payload hashes. Neither metadata repacking nor projection changed any provider bytes. Job `j14336` assembled the archive. Final clinical review accepted the source assertion boundaries without corrections after reading the retained replies.


## Exact request witnesses

The bounded capture job `j15034` ran the fixed seven-row `scripts/cell_line_capture_plan.json` through the existing `make source-capture` helper at pushed commit `24729b68488723af8acf866a1fc6403aba629eb7`. Each row ran once in manifest order with a 25-second deadline, an 8 MiB body bound and half-second spacing. There was no retry, redirect, query change, fallback host or extra page. The helper stopped after the final negative card reply. It classified that HTTP 404 as `provider_failure` and returned 2; the recipe retained that result and completed evidence verification. A successful evidence job does not turn that negative reply into a positive card.

Yellow retains `/home/ian/workspace/data/biomcp-redux/0256-cell-line-captures/2290-cell-line-captures.tar.gz`, SHA-256 `73974a3c186138a1a812bcd5f446ae3be132375846867a02d8d0c675c0097ce8`. Each row below cites this archive and hash. Its `native.tar.gz` contains the plan, every raw body, headers, receipts, state and per-file hash manifest. The outer archive contains the recipe, input plan, OWNER, limits and rights, capture log, runner exit, enriched ledger and another hash manifest. The recipe verified the native archive on M5, checked the archive hash across machines, and independently hashed every outer and native member on Yellow before deleting its own M5 experiment and packed staging. The 0196 source archives remain unchanged.

| Witness | Result in the retained archive | Capability unlocked or deferred |
| --- | --- | --- |
| ChEMBL cross-reference search, `dr:"CHEMBL3706573"`, `start=0&rows=5` | Proven: HTTP 200, one candidate CVCL_2119, 2,121 bytes, 0.927 seconds. No native total or continuation. The retained 0196 CVCL_2119 card supplies the exact `ChEMBL-Cells` database/accession check. | Reverse get from this ChEMBL cell ID can search and then verify the exact qualified xref on its card. The source search uses the accession in `dr`, as the earlier DepMap capture did; it does not bypass card verification. |
| Shared cross-reference search, `dr:"E-MTAB-2770"`, `start=0&rows=5` | Proven ambiguous: HTTP 200, CVCL_L085, CVCL_9980, CVCL_5005, CVCL_5069 and CVCL_5096 in native order; 7,365 bytes, 0.391 seconds. Seven retained 0196 cards share this exact ArrayExpress accession. The full window supplies no total. | Multiple reverse-search candidates stay ambiguous and offer exact candidate gets. This control does not add ArrayExpress to the supported public source-ID grammar. |
| Default projected CVCL_2119 card | Proven: HTTP 200, requested primary accession, 2,009 bytes, 0.365 seconds. | Default get can use the exact documented field projection in the manifest. Names, xrefs and variants still require their own complete source fields for their sections. |
| Default projected CVCL_0004 card | Proven: HTTP 200, requested primary accession, 1,686 bytes, 0.399 seconds. Both projected cards contain accession, age, category, dates/version, disease, identifier name, sex and species. Neither contains xrefs or variation rows. | The large K-562 xref list is absent from the default source transfer. These native body sizes are not product envelope or token measurements. The projection asks for `problematic` and `caution`; these two cards carry neither warning, so this capture does not prove warning projection on a warning-bearing card. |
| Exact accession search, `ac:"CVCL_0005"`, `start=0&rows=5` | Proven: HTTP 200, one candidate CVCL_0005, 1,690 bytes, 0.383 seconds; no native total. | Canonical CVCL input to search can use this exact accession syntax and keep native candidates. It need not send an NB4 name query. |
| ChEMBL second accession, `cellosaurus_id=CVCL_0064&limit=5&offset=0` | Proven: HTTP 200, one row CHEMBL3308063 with exact CVCL_0064, native total 1 and null next/previous; 434 bytes, 1.443 seconds. | Exact accession-linked metadata has a second positive control. No retained cell metadata states a zero assay total, so the approved second-accession alternative was used. Zero-assay behavior remains synthetic; no additional assay call ran. |
| Absent card, CVCL_ZZZZ | Proven source-local absence: HTTP 404 with JSON `code:404` and message `Item not found, ac: CVCL_ZZZZ`; 64 bytes, 0.386 seconds. The generic capture helper records `provider_failure`. | A product adapter may recognize this exact valid native not-found contract as empty. Transport errors, other HTTP failures and malformed bodies remain failed. This is no claim about cell-line existence outside this source. |
| Secondary-accession card | Not called: all nine retained 0196 cards list primary accessions only. No source-supplied secondary target exists among these inputs. | Secondary resolution stays deferred. Get accepts primary accessions only until a retained secondary card proves that route. No guessed secondary ID or discovery request was added. |

The projection uses the retained OpenAPI's comma-separated `fields` form and documented `ac,id,ox,di,ca,sx,ag,problematic,caution,dt` tags. Source requests and response ceilings remain separate. The approved source rights remain Cellosaurus CC BY 4.0 and ChEMBL CC BY-SA 3.0, with attribution and publisher terms preserved. PharmacoDB, DepMap downloads and HPA were not called. The PM's cell-line ruling above the proposal remains authoritative, including the bounded `all` section and its single ChEMBL metadata request.

## Identity request contracts

The default card asks for the documented `ac,id,ox,di,ca,sx,ag,problematic,caution,dt` projection. A reverse get uses the separately retained `ac,id,sy,ox,di,ca,sx,ag,dr,var,prob,caution,dt` request because its native cross-references must verify the exact requested database and accession before cache admission. These different field projections retain different transport and cache identities. Omitted cross-references and variations in a default card never establish their source absence.

A literal native search and DepMap reverse search omit `start` at the provider's default zero and send it for a positive offset. The distinct qualified ChEMBL reverse command uses the retained explicit `start=0` request. It always sends the checked `rows` and never sends a sort selector. Qualified reverse search uses the unchanged accession in the source's `dr` expression. Only DepMap, Cell Model Passports, ChEMBL cell and PharmacoDB identity qualifiers map to reviewed native database spellings; this mapping grants no PharmacoDB data-source access. ArrayExpress reverse input remains unsupported despite its useful retained ambiguity witness. Secondary accessions remain deferred.

The additional bounded projected-card capture ran as `j15093` at `bb461bfb88d0bf6f59b8d8ced41b782374af0825`. Its fixed plan is `scripts/cell_line_identity_plan.json`. It requested CVCL_0005, CVCL_0002, CVCL_A794, CVCL_0007 and CVCL_ZZZZ once each, in that order. Every positive reply had to carry the requested primary accession; the U-937 reply had to preserve the complete stock-qualified warning before the final request was admitted. The four positive replies were HTTP 200; the last reply was the expected native HTTP 404 JSON absence. The helper preserved that negative result rather than calling it a successful card. Each request had a 25-second deadline and 8 MiB limit with half-second spacing, no credentials, retries, redirects, fallback or additional query. The original source captures remain unchanged.

Yellow retains `data/biomcp-redux/0258-cell-line-identity/projected-cards.tar.gz`, SHA-256 `8b2c4cbc1ff921f9043817c016b32315decab94448b76a7a28e5f6e8d8ea138b`, with the OWNER and exact job recipe beside it. All 17 archive members and the archive hash were verified on Yellow and against the M5 copy before the owned M5 staging was deleted. The unchanged open-source bodies used by command tests live in the native fixtures; they contain no curation-team answers.

`CellLinePage` and `CellLineCard` carry independent optional release version/update data and an explicit original `fetched` timestamp. The latter labels retrieval time and never supplies a source release. An independent cached `/release-info?format=json` read supplies the publisher release. Its failure retains the useful record, marks a partial command and source failure, and adds a `release` problem. Search totals remain unknown even when release metadata carries dataset counts.

The shared request schema advances to version 22. The changed page and card envelopes advance to version 2. Typed next steps retain the existing argument-array wire format, so the step structure has no wire change. Named section steps parse through the shared grammar; the command admission boundary explicitly refuses them until their section implementations exist. It never substitutes the default card for a requested section.

A parsed `CellosaurusAccession` search uses the exact native `ac` query with explicit `start=0`, as retained in the 0256 witness. Every other search term keeps its literal `idsy` query unchanged. Exact card retrieval uses `get`.

## Measured command output budgets

These are complete compact product envelopes. CLI includes its newline; MCP measures the actual structured result separately. Markdown measures the actual UTF-8 document. Each listed fixture, phase and surface owns its observed size and a ceiling of its own measurement plus 15 percent. Token ceilings are byte ceilings divided by four and rounded up, not tokenizer counts. Ordinary unlisted Rust phases fail; executable YAML steps own their canonical `max_bytes` and `max_tokens`. Retained native search measurements come from `j15131`; the remaining card, reverse, fault, help and research measurements come from `j15136`. `j15127` measured the four executable examples independently and produced their unchanged replay files.

| Owning phase | Surface | Observed bytes | Ceiling bytes | Estimated token ceiling |
| --- | --- | ---: | ---: | ---: |
| `help/get` | cli | 1718 | 1976 | 494 |
| `help/get` | markdown | 1553 | 1786 | 447 |
| `help/get` | mcp | 1717 | 1975 | 494 |
| `help/search` | cli | 880 | 1012 | 253 |
| `help/search` | markdown | 855 | 984 | 246 |
| `help/search` | mcp | 879 | 1011 | 253 |
| `native/card/CVCL_0002` | cli | 1873 | 2154 | 539 |
| `native/card/CVCL_0002` | mcp | 1872 | 2153 | 539 |
| `native/card/CVCL_0004` | cli | 1863 | 2143 | 536 |
| `native/card/CVCL_0004` | mcp | 1862 | 2142 | 536 |
| `native/card/CVCL_0005` | cli | 1848 | 2126 | 532 |
| `native/card/CVCL_0005` | mcp | 1847 | 2125 | 532 |
| `native/card/CVCL_0007` | cli | 2077 | 2389 | 598 |
| `native/card/CVCL_0007` | markdown | 2199 | 2529 | 633 |
| `native/card/CVCL_0007` | mcp | 2076 | 2388 | 597 |
| `native/card/CVCL_2119` | cli | 2080 | 2392 | 598 |
| `native/card/CVCL_2119` | mcp | 2079 | 2391 | 598 |
| `native/card/CVCL_A794` | cli | 1877 | 2159 | 540 |
| `native/card/CVCL_A794` | mcp | 1876 | 2158 | 540 |
| `native/card/CVCL_ZZZZ` | cli | 438 | 504 | 126 |
| `native/card/CVCL_ZZZZ` | mcp | 437 | 503 | 126 |
| `native/reverse/ChEMBL:CHEMBL3706573` | cli | 2102 | 2418 | 605 |
| `native/reverse/ChEMBL:CHEMBL3706573` | mcp | 2101 | 2417 | 605 |
| `native/reverse/DepMap:ACH-000362` | cli | 2096 | 2411 | 603 |
| `native/reverse/DepMap:ACH-000362` | mcp | 2095 | 2410 | 603 |
| `native/search/KG1` | cli | 2724 | 3133 | 784 |
| `native/search/KG1` | mcp | 2723 | 3132 | 783 |
| `native/search/MOLM13` | cli | 1589 | 1828 | 457 |
| `native/search/MOLM13` | markdown | 1620 | 1863 | 466 |
| `native/search/MOLM13` | mcp | 1588 | 1827 | 457 |
| `native/search/MV4-semicolon` | cli | 1358 | 1562 | 391 |
| `native/search/MV4-semicolon` | mcp | 1357 | 1561 | 391 |
| `native/search/NB4-first` | cli | 5327 | 6127 | 1532 |
| `native/search/NB4-first` | mcp | 5326 | 6125 | 1532 |
| `native/search/NB4-second` | cli | 5336 | 6137 | 1535 |
| `native/search/NB4-second` | mcp | 5335 | 6136 | 1534 |
| `native/search/empty` | cli | 382 | 440 | 110 |
| `native/search/empty` | mcp | 381 | 439 | 110 |
| `research/fetch/CVCL_2119` | mcp | 4461 | 5131 | 1283 |
| `research/search/MOLM13` | mcp | 1828 | 2103 | 526 |
| `synthetic/card/429` | cli | 468 | 539 | 135 |
| `synthetic/card/429` | mcp | 467 | 538 | 135 |
| `synthetic/card/500` | cli | 467 | 538 | 135 |
| `synthetic/card/500` | mcp | 466 | 536 | 134 |
| `synthetic/card/broken` | cli | 465 | 535 | 134 |
| `synthetic/card/broken` | mcp | 464 | 534 | 134 |
| `synthetic/card/html` | cli | 465 | 535 | 134 |
| `synthetic/card/html` | mcp | 464 | 534 | 134 |
| `synthetic/card/release500` | cli | 2199 | 2529 | 633 |
| `synthetic/card/release500` | mcp | 2198 | 2528 | 632 |
| `synthetic/card/timeout` | cli | 473 | 544 | 136 |
| `synthetic/card/timeout` | mcp | 472 | 543 | 136 |
| `synthetic/disabled/prime-card` | cli | 2080 | 2392 | 598 |
| `synthetic/disabled/prime-card` | mcp | 2079 | 2391 | 598 |
| `synthetic/disabled/warm-card` | cli | 390 | 449 | 113 |
| `synthetic/disabled/warm-card` | mcp | 389 | 448 | 112 |
| `synthetic/no-admission/accession/failed` | cli | 465 | 535 | 134 |
| `synthetic/no-admission/accession/failed` | mcp | 464 | 534 | 134 |
| `synthetic/no-admission/accession/healthy` | cli | 2080 | 2392 | 598 |
| `synthetic/no-admission/accession/healthy` | mcp | 2079 | 2391 | 598 |
| `synthetic/no-admission/bad-version/failed` | cli | 465 | 535 | 134 |
| `synthetic/no-admission/bad-version/failed` | mcp | 464 | 534 | 134 |
| `synthetic/no-admission/bad-version/healthy` | cli | 2080 | 2392 | 598 |
| `synthetic/no-admission/bad-version/healthy` | mcp | 2079 | 2391 | 598 |
| `synthetic/no-admission/xref/failed` | cli | 489 | 563 | 141 |
| `synthetic/no-admission/xref/failed` | mcp | 488 | 562 | 141 |
| `synthetic/no-admission/xref/healthy` | cli | 2096 | 2411 | 603 |
| `synthetic/no-admission/xref/healthy` | mcp | 2095 | 2410 | 603 |
| `synthetic/no-admission/zero-version/failed` | cli | 465 | 535 | 134 |
| `synthetic/no-admission/zero-version/failed` | mcp | 464 | 534 | 134 |
| `synthetic/no-admission/zero-version/healthy` | cli | 2080 | 2392 | 598 |
| `synthetic/no-admission/zero-version/healthy` | mcp | 2079 | 2391 | 598 |
| `synthetic/reverse/ambiguous-DepMap` | cli | 4732 | 5442 | 1361 |
| `synthetic/reverse/ambiguous-DepMap` | mcp | 4731 | 5441 | 1361 |
| `synthetic/search/429` | cli | 381 | 439 | 110 |
| `synthetic/search/429` | mcp | 380 | 437 | 110 |
| `synthetic/search/500` | cli | 380 | 437 | 110 |
| `synthetic/search/500` | mcp | 379 | 436 | 109 |
| `synthetic/search/broken` | cli | 378 | 435 | 109 |
| `synthetic/search/broken` | mcp | 377 | 434 | 109 |
| `synthetic/search/html` | cli | 378 | 435 | 109 |
| `synthetic/search/html` | mcp | 377 | 434 | 109 |
| `synthetic/search/timeout` | cli | 386 | 444 | 111 |
| `synthetic/search/timeout` | mcp | 385 | 443 | 111 |

| Executable YAML step | CLI bytes | MCP bytes | Canonical byte ceiling | Estimated token ceiling |
| --- | ---: | ---: | ---: | ---: |
| `cell-line/examples.yaml` step 1 | 1589 | 1588 | 1828 | 457 |
| `cell-line/examples.yaml` step 2 | 1358 | 1357 | 1562 | 391 |
| `cell-line/examples.yaml` step 3 | 2080 | 2079 | 2392 | 598 |
| `cell-line/examples.yaml` step 4 | 1863 | 1862 | 2143 | 536 |

The focused discovery check in `j15137` measured whole-tree CLI help at 84,770 bytes including its newline. The compact MCP tools/list RPC reply was 7,713 bytes and its main-tool description was 3,129 bytes; all remain within existing shared limits. The source enum array was 673 compact bytes and the printed schema was 3,670 bytes, within their existing limits. Entity search help was 760 CLI bytes, 759 MCP bytes and 672 bytes for the bare Help result; get help was 1,015, 1,014 and 930 respectively. Help has no source-term subtraction. The owning table above uses the complete envelopes, with separate actual Markdown measurements. The preserved-recording example replay passed with the explicit `--query` alias; no recording was repeated.

The restored checked canonical-accession branch was measured independently in `j15149`: `native/search/accession-CVCL_0005` returned 1,358 CLI bytes including newline and 1,357 MCP bytes. Its separate ceilings are 1,562 and 1,561 bytes (391 estimated tokens each), with the same 15 percent headroom. It uses the unchanged exact 0256 `ac` witness; no provider call or completed literal-search measurement was repeated.

## ChEMBL request and provenance contracts

An explicit `chembl` disclosure returns the standalone metadata envelope. A primary CVCL accession sends only the exact ChEMBL metadata request and its independent status request. A qualified source identity first follows the existing Cellosaurus verification route. The same command deadline covers verification and ChEMBL retrieval. The metadata request sends `cellosaurus_id`, the checked section limit and section offset. Each native row must carry the exact requested CVCL accession and a valid native ChEMBL cell ID before cache admission. Native names, descriptions, integer cell IDs, organism labels, taxon, tissue and underscore-spelled CLO/EFO identifiers survive unchanged. Native nulls remain absent.

Cell-specific native page metadata preserves `next` and `previous` for admission checks without changing drug endpoint behavior. The native limit, offset, total and received row count must agree. A next link must agree with the remaining native window, and an initial page cannot carry a previous link. Contradictions fail before cache admission. A declared remaining page produces a partial metadata outcome and a typed continuation command. It never establishes that the current response contains every matching record.

The assay-count command sends exactly `cell_chembl_id`, `limit=1` and `offset=0`. Every received assay row must carry the requested cell ID before cache admission. Only the native assay total enters the result. A native total of zero remains a successful zero. Metadata and `all` never issue this request. No assay listing, activity download, drug sensitivity estimate or biological interpretation enters the response.

The separate cached `/status.json` read supplies `chembl_db_version` and `chembl_release_date`. Its failure preserves useful metadata or the assay count, retains the original labelled retrieval timestamp, marks the source failed and adds a `release` problem. Disabled ChEMBL reads bypass both cache and network. Publisher policy remains the existing ChEMBL attribution category and CC BY-SA 3.0 terms.

The shared request schema advances to version 23 for the assay-count request. ChEMBL metadata and assay-count envelopes advance to version 2 for independent optional release and retrieval provenance. The existing response record fields retain their native source meaning.

The native ChEMBL fixtures contain unchanged bodies from the verified 0196 portable ChEMBL archive for CVCL_2119, CVCL_0004, CVCL_9999, the exact assay-count request and status. The CVCL_0064 body comes from the verified 0256 native archive. Both outer archive hashes and all outer and native manifest member hashes were checked before copying these six bodies. The protected originals remain unchanged. Paging contradictions, malformed identities, zero assays and source faults are explicitly synthetic controls. No provider request or retry was added.

## Measured ChEMBL output budgets

Each row owns one fixture or synthetic variant, phase and actual output surface. CLI bytes include the emitted newline. MCP bytes measure the emitted tool text that is checked against structured content. The full RPC framing is measured separately. Markdown bytes measure actual UTF-8 stdout. These complete envelopes include provenance, source terms and typed next commands. The earlier native projections are payload design samples and supply no full-envelope ceiling. The established source-term-subtracted comparable basis removes exactly the registry's three policy fields; one ChEMBL outcome contributes 107 bytes. Help has no term subtraction. The executable cell-line YAML examples retain their existing canonical limits and own no ChEMBL phase.

Native and help measurements come from `j15181`; completed metadata source-fault measurements come from `j15185`; the remaining source-fault, cache, provenance and synthetic-zero measurements come from `j15189`. The contradictory paging controls and complete five-row synthetic continuation come from `j15195`. The actual-byte helper verifies the same emitted surfaces in the routine suite. Each exact Rust phase resolves through `tests/cell-line-command-budgets.yaml`. Unlisted phases fail. No source or fault group supplies a shared maximum, and no argv table duplicates these limits.

| Owning phase | Surface | Observed bytes | Ceiling bytes | Estimated token ceiling |
| --- | --- | ---: | ---: | ---: |
| `chembl/admission/assays/bad-page/failed` | cli | 438 | 504 | 126 |
| `chembl/admission/assays/bad-page/healthy` | cli | 376 | 433 | 109 |
| `chembl/admission/assays/malformed/failed` | cli | 438 | 504 | 126 |
| `chembl/admission/assays/malformed/healthy` | cli | 376 | 433 | 109 |
| `chembl/admission/assays/mismatch/failed` | cli | 438 | 504 | 126 |
| `chembl/admission/assays/mismatch/healthy` | cli | 376 | 433 | 109 |
| `chembl/admission/assays/missing/failed` | cli | 438 | 504 | 126 |
| `chembl/admission/assays/missing/healthy` | cli | 376 | 433 | 109 |
| `chembl/admission/assays/next/failed` | cli | 438 | 504 | 126 |
| `chembl/admission/assays/next/healthy` | cli | 376 | 433 | 109 |
| `chembl/admission/metadata/bad-page/failed` | cli | 464 | 534 | 134 |
| `chembl/admission/metadata/bad-page/healthy` | cli | 719 | 827 | 207 |
| `chembl/admission/metadata/malformed/failed` | cli | 464 | 534 | 134 |
| `chembl/admission/metadata/malformed/healthy` | cli | 719 | 827 | 207 |
| `chembl/admission/metadata/mismatch/failed` | cli | 464 | 534 | 134 |
| `chembl/admission/metadata/mismatch/healthy` | cli | 719 | 827 | 207 |
| `chembl/admission/metadata/missing/failed` | cli | 464 | 534 | 134 |
| `chembl/admission/metadata/missing/healthy` | cli | 719 | 827 | 207 |
| `chembl/admission/metadata/next/failed` | cli | 464 | 534 | 134 |
| `chembl/admission/metadata/next/healthy` | cli | 719 | 827 | 207 |
| `chembl/disabled/assays/prime` | cli | 376 | 433 | 109 |
| `chembl/disabled/assays/warm` | cli | 351 | 404 | 101 |
| `chembl/disabled/metadata/prime` | cli | 719 | 827 | 207 |
| `chembl/disabled/metadata/warm` | cli | 380 | 437 | 110 |
| `chembl/fault/assays/429` | cli | 441 | 508 | 127 |
| `chembl/fault/assays/429` | mcp | 440 | 506 | 127 |
| `chembl/fault/assays/500` | cli | 440 | 506 | 127 |
| `chembl/fault/assays/500` | mcp | 439 | 505 | 127 |
| `chembl/fault/assays/broken` | cli | 438 | 504 | 126 |
| `chembl/fault/assays/broken` | mcp | 437 | 503 | 126 |
| `chembl/fault/assays/html` | cli | 438 | 504 | 126 |
| `chembl/fault/assays/html` | mcp | 437 | 503 | 126 |
| `chembl/fault/assays/timeout` | cli | 446 | 513 | 129 |
| `chembl/fault/assays/timeout` | mcp | 445 | 512 | 128 |
| `chembl/fault/metadata/429` | cli | 467 | 538 | 135 |
| `chembl/fault/metadata/429` | mcp | 466 | 536 | 134 |
| `chembl/fault/metadata/500` | cli | 466 | 536 | 134 |
| `chembl/fault/metadata/500` | mcp | 465 | 535 | 134 |
| `chembl/fault/metadata/broken` | cli | 464 | 534 | 134 |
| `chembl/fault/metadata/broken` | mcp | 463 | 533 | 134 |
| `chembl/fault/metadata/html` | cli | 464 | 534 | 134 |
| `chembl/fault/metadata/html` | mcp | 463 | 533 | 134 |
| `chembl/fault/metadata/timeout` | cli | 472 | 543 | 136 |
| `chembl/fault/metadata/timeout` | mcp | 471 | 542 | 136 |
| `chembl/help/assay-count` | cli | 632 | 727 | 182 |
| `chembl/help/assay-count` | markdown | 682 | 785 | 197 |
| `chembl/help/assay-count` | mcp | 631 | 726 | 182 |
| `chembl/native/assays` | cli | 376 | 433 | 109 |
| `chembl/native/assays` | markdown | 388 | 447 | 112 |
| `chembl/native/assays` | mcp | 375 | 432 | 108 |
| `chembl/native/metadata/CVCL_0004` | cli | 746 | 858 | 215 |
| `chembl/native/metadata/CVCL_0004` | markdown | 848 | 976 | 244 |
| `chembl/native/metadata/CVCL_0004` | mcp | 745 | 857 | 215 |
| `chembl/native/metadata/CVCL_0064` | cli | 752 | 865 | 217 |
| `chembl/native/metadata/CVCL_0064` | markdown | 854 | 983 | 246 |
| `chembl/native/metadata/CVCL_0064` | mcp | 751 | 864 | 216 |
| `chembl/native/metadata/CVCL_2119` | cli | 719 | 827 | 207 |
| `chembl/native/metadata/CVCL_2119` | markdown | 807 | 929 | 233 |
| `chembl/native/metadata/CVCL_2119` | mcp | 718 | 826 | 207 |
| `chembl/native/metadata/CVCL_9999` | cli | 429 | 494 | 124 |
| `chembl/native/metadata/CVCL_9999` | markdown | 466 | 536 | 134 |
| `chembl/native/metadata/CVCL_9999` | mcp | 428 | 493 | 124 |
| `chembl/release/assays/500` | cli | 506 | 582 | 146 |
| `chembl/release/assays/500` | mcp | 505 | 581 | 146 |
| `chembl/release/metadata/500` | cli | 837 | 963 | 241 |
| `chembl/release/metadata/500` | mcp | 836 | 962 | 241 |
| `chembl/synthetic/assay-zero` | cli | 373 | 429 | 108 |
| `chembl/synthetic/assay-zero` | mcp | 372 | 428 | 107 |
| `chembl/synthetic/more` | cli | 1855 | 2134 | 534 |
| `chembl/synthetic/more` | mcp | 1854 | 2133 | 534 |

## Complete-card field contracts

Named Cellosaurus sections reuse the retained `ac,id,sy,ox,di,ca,sx,ag,dr,var,prob,caution,dt` request. The first cold section reads one checked complete reply; independently requested pages share that request and cache identity. The default card retains its separate smaller projection. Unrequested collections remain distinct from source omission.

The complete native xref inventory contains `database`, `accession`, `label`, `url`, `category`, `iri` and `discontinued`. The last field is native text, including `true` and `probable`; it is not converted to a boolean. Native names contain `type` and `value`. Variations contain `variation-type`, optional `mutation-type`, optional `mutation-description`, optional `variation-note`, optional `zygosity-type`, `xref-list` and `source-list`. Provenance contains independently optional `reference`, `institution`, `value` and `xref`; publication references contain `resource-internal-ref`. Every retained member maps without clipping. HL-60 deletion rows omit `mutation-description`, so the public description is optional. The card envelope advances to version 3 and standalone variants to version 2 for that omission contract. The four standalone section pages retain independent source release and original complete-card retrieval time.

Complete-card ambiguity retains the card's bounded candidate identities and exact gets. Gene pivots use only source HGNC members, and article pivots use exact `PubMed=` identities. Native disease accession strings receive no invented namespace. Requested offsets apply independently to species, diseases and warnings, and variation totals count whole source rows.

## Measured native section output budgets

The focused measurements in `j15205` and `j15207` retain separate actual CLI, MCP and Markdown surfaces. Each cell shows observed bytes, its byte ceiling with 15 percent headroom, and the estimated token ceiling. Ordinary cache controls own only their executed CLI surface. Unknown ordinary phase keys fail. The existing command budget fixture owns these phases; executable YAML steps own their own `max_bytes` and `max_tokens`.

| Owning phase | CLI observed / ceiling / tokens | MCP observed / ceiling / tokens | Markdown observed / ceiling / tokens |
| --- | ---: | ---: | ---: |
| `sections/admission/provenance/failed` | 412 / 474 / 119 |  |  |
| `sections/admission/provenance/healthy` | 2182 / 2510 / 628 |  |  |
| `sections/admission/xref/failed` | 412 / 474 / 119 |  |  |
| `sections/admission/xref/healthy` | 2182 / 2510 / 628 |  |  |
| `sections/bounds/identity/1/1` | 894 / 1029 / 258 | 893 / 1027 / 257 | 982 / 1130 / 283 |
| `sections/bounds/names/1/1` | 660 / 759 / 190 | 659 / 758 / 190 | 714 / 822 / 206 |
| `sections/bounds/variants/25/100` | 521 / 600 / 150 | 520 / 598 / 150 | 572 / 658 / 165 |
| `sections/bounds/xrefs/1/1` | 886 / 1019 / 255 | 885 / 1018 / 255 | 961 / 1106 / 277 |
| `sections/cache/identity` | 1526 / 1755 / 439 |  |  |
| `sections/cache/names` | 638 / 734 / 184 |  |  |
| `sections/cache/variants` | 2182 / 2510 / 628 |  |  |
| `sections/cache/xrefs` | 1559 / 1793 / 449 |  |  |
| `sections/disabled/prime` | 2182 / 2510 / 628 |  |  |
| `sections/disabled/warm` | 326 / 375 / 94 |  |  |
| `sections/full/CVCL_0002/names` | 697 / 802 / 201 | 696 / 801 / 201 | 703 / 809 / 203 |
| `sections/full/CVCL_0002/variants` | 2970 / 3416 / 854 | 2969 / 3415 / 854 | 3144 / 3616 / 904 |
| `sections/full/CVCL_0007/variants` | 4207 / 4839 / 1210 | 4206 / 4837 / 1210 | 4560 / 5244 / 1311 |
| `sections/full/CVCL_0064/variants` | 1948 / 2241 / 561 | 1947 / 2240 / 560 | 1990 / 2289 / 573 |
| `sections/full/CVCL_1844/variants` | 3202 / 3683 / 921 | 3201 / 3682 / 921 | 3366 / 3871 / 968 |
| `sections/full/CVCL_2119/variants` | 2201 / 2532 / 633 | 2200 / 2530 / 633 | 2156 / 2480 / 620 |
| `sections/full/CVCL_8821/names` | 691 / 795 / 199 | 690 / 794 / 199 | 697 / 802 / 201 |
| `sections/full/CVCL_8821/variants` | 1219 / 1402 / 351 | 1218 / 1401 / 351 | 1356 / 1560 / 390 |
| `sections/full/CVCL_8821/xrefs` | 2508 / 2885 / 722 | 2507 / 2884 / 721 | 2120 / 2438 / 610 |
| `sections/full/CVCL_A794/variants` | 2385 / 2743 / 686 | 2384 / 2742 / 686 | 2654 / 3053 / 764 |
| `sections/native/CVCL_0002/identity` | 1319 / 1517 / 380 | 1318 / 1516 / 379 | 1409 / 1621 / 406 |
| `sections/native/CVCL_0002/names` | 678 / 780 / 195 | 677 / 779 / 195 | 679 / 781 / 196 |
| `sections/native/CVCL_0002/variants` | 2951 / 3394 / 849 | 2950 / 3393 / 849 | 3120 / 3588 / 897 |
| `sections/native/CVCL_0002/xrefs` | 1592 / 1831 / 458 | 1591 / 1830 / 458 | 1517 / 1745 / 437 |
| `sections/native/CVCL_0007/identity` | 1525 / 1754 / 439 | 1524 / 1753 / 439 | 1627 / 1872 / 468 |
| `sections/native/CVCL_0007/names` | 596 / 686 / 172 | 595 / 685 / 172 | 627 / 722 / 181 |
| `sections/native/CVCL_0007/variants` | 2707 / 3114 / 779 | 2706 / 3112 / 778 | 2841 / 3268 / 817 |
| `sections/native/CVCL_0007/xrefs` | 1592 / 1831 / 458 | 1591 / 1830 / 458 | 1517 / 1745 / 437 |
| `sections/native/CVCL_0007/xrefs-qualified` | 1153 / 1326 / 332 | 1152 / 1325 / 332 | 1089 / 1253 / 314 |
| `sections/native/CVCL_0064/identity` | 1296 / 1491 / 373 | 1295 / 1490 / 373 | 1386 / 1594 / 399 |
| `sections/native/CVCL_0064/names` | 767 / 883 / 221 | 766 / 881 / 221 | 760 / 874 / 219 |
| `sections/native/CVCL_0064/variants` | 1929 / 2219 / 555 | 1928 / 2218 / 555 | 1966 / 2261 / 566 |
| `sections/native/CVCL_0064/xrefs` | 1545 / 1777 / 445 | 1544 / 1776 / 444 | 1467 / 1688 / 422 |
| `sections/native/CVCL_1844/identity` | 1290 / 1484 / 371 | 1289 / 1483 / 371 | 1380 / 1587 / 397 |
| `sections/native/CVCL_1844/names` | 776 / 893 / 224 | 775 / 892 / 223 | 769 / 885 / 222 |
| `sections/native/CVCL_1844/variants` | 3183 / 3661 / 916 | 3182 / 3660 / 915 | 3342 / 3844 / 961 |
| `sections/native/CVCL_1844/xrefs` | 1575 / 1812 / 453 | 1574 / 1811 / 453 | 1497 / 1722 / 431 |
| `sections/native/CVCL_2119/identity` | 1526 / 1755 / 439 | 1525 / 1754 / 439 | 1563 / 1798 / 450 |
| `sections/native/CVCL_2119/names` | 638 / 734 / 184 | 637 / 733 / 184 | 654 / 753 / 189 |
| `sections/native/CVCL_2119/variants` | 2182 / 2510 / 628 | 2181 / 2509 / 628 | 2132 / 2452 / 613 |
| `sections/native/CVCL_2119/xrefs` | 1559 / 1793 / 449 | 1558 / 1792 / 448 | 1481 / 1704 / 426 |
| `sections/native/CVCL_8821/identity` | 1252 / 1440 / 360 | 1251 / 1439 / 360 | 1342 / 1544 / 386 |
| `sections/native/CVCL_8821/names` | 672 / 773 / 194 | 671 / 772 / 193 | 673 / 774 / 194 |
| `sections/native/CVCL_8821/variants` | 1200 / 1380 / 345 | 1199 / 1379 / 345 | 1332 / 1532 / 383 |
| `sections/native/CVCL_8821/xrefs` | 1401 / 1612 / 403 | 1400 / 1610 / 403 | 1308 / 1505 / 377 |
| `sections/native/CVCL_A794/identity` | 1319 / 1517 / 380 | 1318 / 1516 / 379 | 1409 / 1621 / 406 |
| `sections/native/CVCL_A794/names` | 684 / 787 / 197 | 683 / 786 / 197 | 685 / 788 / 197 |
| `sections/native/CVCL_A794/variants` | 2366 / 2721 / 681 | 2365 / 2720 / 680 | 2630 / 3025 / 757 |
| `sections/native/CVCL_A794/xrefs` | 1488 / 1712 / 428 | 1487 / 1711 / 428 | 1416 / 1629 / 408 |
| `sections/synthetic/ambiguous-variants` | 4753 / 5466 / 1367 | 4752 / 5465 / 1367 | 5200 / 5980 / 1495 |
| `sections/synthetic/qualified-identity` | 690 / 794 / 199 | 689 / 793 / 199 | 745 / 857 / 215 |
| `sections/synthetic/release-failure` | 2312 / 2659 / 665 | 2311 / 2658 / 665 | 2280 / 2622 / 656 |

The K-562 executable YAML walk owns its whole 1,398-row cross-reference sequence and exact ordered database/accession floors. `j15207` measured every 25-row page independently in CLI and MCP. The largest pages were 5,102 CLI bytes and 5,101 MCP bytes. The canonical walk ceiling is 5,868 bytes and 1,467 estimated tokens. First, middle and terminal pages retain the same complete source rows and native total. Every variation row stays whole; growth past its owning measured ceiling fails the case instead of clipping the row.

## Bounded composite disclosure and retirements

`get cell-line CVCL_2119 all` and `get cell-line CVCL_0004 all` retain every named section's default first page. The complete checked Cellosaurus reply supplies names, identity, cross-references and variants. One exact-accession ChEMBL metadata request supplies the independent ChEMBL section. The original command deadline covers both sources and their separate release reads. No assay request follows implicitly. Source failure and policy skip remain visible in their section states and source outcomes. A ChEMBL failure keeps the Cellosaurus card partial and offers a fresh command.

Composite paging controls return fixed usage guidance before provider access. Each explicit section limit is a composite control because named sections have different defaults. A nonzero section offset is also refused. The named section command owns paging. Deferred PharmacoDB spellings and the HPA expression command return fixed reasons before provider access. The prior drug cell-line retirement remains unchanged. Positional text submits unchanged Cellosaurus queries.

The outside-in composite cases reuse the protected native fixtures and the existing independent source fault matrices. Additional composites label synthetic failures and distinguish successful empty, failed, skipped and unavailable states. The ordinary-cache policy cases reject disabled sources before retained replies or requests. A local HTML verification page proves that neither source follows or retries a verification route. No live provider request, licensed fixture or analysis tool is involved.

The `all/` keys in `tests/cell-line-command-budgets.yaml` own measured output sizes and derive each ceiling independently through the existing measured-size helper. CLI bytes include the emitted newline. MCP text bytes and the complete RPC reply have distinct entries. Markdown has its own measured entry for each native composite. No group maximum or shared budget replaces a fixture and phase measurement.


## Generated cell line help

The generated help examples come from the canonical case run steps. The existing parsed-example check executes both examples for search, get and assay-count through CLI and MCP. The assay examples use the same retained exact source record with an explicit fresh read as the second spelling. They add no provider request or source interpretation. Named get sections expose their independent paging; a qualified source ID still requires exact candidate verification. Each command states that assay counts are not activity or sensitivity counts, no treatment response is inferred and no bioinformatics tool runs.

Job `j15387` measured complete help output after the source enum supplied its existing variant descriptions to generated help. The generated section list contains names, identity, xrefs, variants, chembl and all. The PM approved each changed help cap from its own measured surface plus fifteen percent on 2026-10-26. The checked-in query and response schemas retain their existing wire shape and identities; only description metadata changed.

| Help command | CLI including newline | Bare Help result | MCP text | Markdown | Full RPC including newline |
| --- | ---: | ---: | ---: | ---: | ---: |
| `help search cell-line` | 880 | 792 | 879 | 855 | 2080 |
| `help get cell-line` | 1718 | 1633 | 1717 | 1553 | 3836 |
| `help cell-line assay-count` | 632 | 539 | 631 | 682 | 1546 |

The MCP description measured 3,401 UTF-8 bytes and keeps the existing description ceiling. Whole CLI help measured 86,118 emitted bytes and keeps its existing global ceiling. The complete compact tools/list RPC measured 7,987 bytes without its newline and stays within its approved cap. That cap retains the PM-approved `j15358` measurement plus fifteen percent. Discovery includes the exact-source input, independent paging, named sections and the explicit assay-count route. Source names and their schema retain the shared source registry. The emitted source schema measured 3,670 bytes, and the compact source enum measured 673 bytes; their existing caps remain unchanged.
