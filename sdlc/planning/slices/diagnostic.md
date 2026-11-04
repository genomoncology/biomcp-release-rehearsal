# Diagnostic slice design and implemented accounting

This investigation starts from `d18bfbba511e3852c99228f1a5166a14633e5cee`; the PM review revision merges current main `8f2a1a8329d7f9877b5df7aad0ec987440e67d1f` and applies the 2026-10-25 ticket rulings. It follows steps 1 to 4 of [the slice method](../slices.md). It proposes source-attributed diagnostic inventory and regulatory facts for cancer care. The original investigation changed no product code; the implementation and accounting sections below record the later landed surface. Provider observations, public document findings and clinical rulings remain separate. No licensed curation answers enter this document.

## Evidence boundary

The public documents below establish source scope and candidate fields. Native capture and output measurements are recorded below separately from the proposed product surface. The later type and adapter work reviewed absence, multiplicity and scope against those bodies. No API requests, bundle downloads, released executable runs or product measurements have been performed by this design author. Historical repository experiments supply leads only; their text-based gene matching and regulatory name overlays do not prove joins and will not be ported.

## Bounded native capture recipe

The coordinator assigns this recipe to a separate build delegate through `pm job` from an exact pushed revision. Raw bodies, response headers, status, original URL and query, retrieval time, elapsed time, bytes, SHA-256, decoder results and all failed attempts remain outside Git on Yellow. Start a fresh experiment and real recording directory. Record one row per physical request; never synthesize request identities or hashes. Download untrusted bodies as bytes before parsing. No provider retry, redirect following, altered query, fallback, automatic refresh or additional pages are authorized by this proposal. A failed request consumes its slot. A redirect records Location and stops that leg. Do not print keys. No key discovery or account creation.

| Ledger rows | Exact request | Purpose and limit |
| --- | --- | --- |
| G01 | `GET https://ftp.ncbi.nlm.nih.gov/pub/GTR/data/_README.html` | Published file descriptions and update dates; 2 MiB body bound. |
| G02 | `GET https://ftp.ncbi.nlm.nih.gov/pub/GTR/data/test_version.gz` | Full unchanged test-version bundle; 100 MiB compressed, 512 MiB expanded, 1,000,000 row bounds. These are stop bounds inherited from inspected legacy source, not measured sizes. |
| G03 | `GET https://ftp.ncbi.nlm.nih.gov/pub/GTR/data/test_condition_gene.txt` | Full unchanged native gene/condition relation bundle; 50 MiB and 1,000,000 row bounds. |
| W01 | `GET https://extranet.who.int/prequal/vitro-diagnostics/prequalified/in-vitro-diagnostics/export?page&_format=csv` | Whole native CSV; 2 MiB stop bound. Preserve all rows and original product-code cells, including multiline values if present. Keep the complete WHO bundle unfiltered. HPV questions demonstrate the cancer-care use without deleting unrelated native products. |
| F01 | `GET https://www.fda.gov/medical-devices/in-vitro-diagnostics/list-fda-authorized-companion-diagnostic-devices-in-vitro-and-imaging-tools` | Candidate explicit companion relation table; 10 MiB bound. Preserve table row and rowspan scope, full indication/sample, drug NDA/BLA and exact device/supplement IDs. Do not parse biological meaning from text. |
| F02–F13 | `GET https://api.fda.gov/device/pma.json?search=pma_number:%22P170019%22&limit=100&skip=0`, with the same fixed query for `P190014`, `P190032`, `P200010`, `P170041`, `P170005`, `P160040`, `P160045`, `P150013`, `P150025`, `P090015`, `P210011` | Twelve independent base-application pages; 10 MiB each. Preserve all supplements returned, native totals/order, absent versus empty supplement IDs. Stop with partial coverage if total exceeds 100; no extra page in this budget. |
| F14 | `GET https://api.fda.gov/device/510k.json?search=k_number:%22K080653%22&limit=1&skip=0` | Exact 510(k) identity and distinction from PMA. This is a native identity probe; a zero is allowed and cannot be relabeled positive. 2 MiB bound. |
| F15 | `GET https://api.fda.gov/device/pma.json?search=pma_number:%22P000000%22&limit=1&skip=0` | Explicit source-local absence control; retain actual HTTP/body classification. 2 MiB bound. |

Maximum: 19 physical GETs, of which 14 contact the openFDA API. Make keyless requests at the published public quota. If the current terms prohibit keyless use despite its documented no-key allowance, stop for the coordinator's ruling; never borrow a key. Public document review can continue independently. No GTR API, XML bulk file, FDA whole download, WHO individual report or drug API is included. A further source is a separate recipe.

Offline analysis reuses these exact bodies without network. Report full bundle sizes, row counts and current/historical counts; do not confuse report-row counts with unique tests or test-gene pairs with unique genes. Inventory every native column and its optionality, arrays, delimiters, namespace and scope. Compare the same GTR accession/version across both bundles. Identify named test candidates only as candidates; never assert a GTR–FDA or WHO–FDA identity from names. Extract exact NCBI Gene/MedGen/OMIM identifiers if native fields carry them. If they do not, mark that join unproved. Preserve WHO product ID separately from each product code and manufacturer. Group repeated CSV rows only by the native WHO Product ID, preserving each full code row; a repeated product can have several codes without a multiline cell. Inspect whether any structured cross-source identity appears before proposing a next command.

For each harvested question, evaluate all three candidate sources against the retained data, recording supported fields, partial answer, unsupported scope or source-local zero. Sources lacking the relevant field are unsupported, not a successful empty result. Record one cold local parse and one repeated stored query per selection with request count, elapsed time, full compact JSON byte count and comparable byte count after validating the exact terms fields removed. These are investigation projections, never product CLI/MCP outputs or an approved product budget. No fabricated timing, tokens or default card result. Build types only after this evidence resolves absence, multiplicity and scope.

## Source document findings

[NCBI access documentation](https://www.ncbi.nlm.nih.gov/gtr/docs/maintenance_use/) describes weekly comprehensive bulk reports and daily website updates. Its Entrez summary truncates condition display, so it cannot establish the complete condition set. The retained file README gives a more specific daily cadence for the relation report; preserve that source-specific cadence rather than silently applying the overview’s weekly schedule to every file. [GTR overview](https://www.ncbi.nlm.nih.gov/gtr/docs/overview/) describes submitter-provided data and says FDA review information is not independently verified by NIH or FDA. A laboratory listing, test status and CLIA number do not establish device authorization. The inspected legacy parser's two reports are a limited tier, not the complete XML dataset.

[openFDA PMA](https://open.fda.gov/apis/device/pma/) and [510(k)](https://open.fda.gov/apis/device/510k/) document monthly updates. Their native identity and regulatory routes remain distinct. The PMA [field reference](https://open.fda.gov/apis/device/pma/searchable-fields/) offers application/supplement, decision dates and approval-order statement; it does not document structured drug, gene, specimen or tumor linkage. The official [companion table](https://www.fda.gov/medical-devices/in-vitro-diagnostics/list-fda-authorized-companion-diagnostic-devices-in-vitro-and-imaging-tools) has those source columns and exact application/supplement and NDA/BLA identifiers. The PM approved a separate read-only `fda-cdx` table bundle, which is needed to answer companion questions without a name overlay. Do not infer those columns by mining approval-order prose.

The [WHO list](https://extranet.who.int/prequal/vitro-diagnostics/prequalified/in-vitro-diagnostics) publishes product IDs, multiple product codes, assay formats, manufacturer, marker, regulatory version and prequalification year. It includes HPV assays. A year is not an exact prequalification date; regulatory-version prose does not supply an FDA application ID. The site's list count is a web display observation, not a captured complete CSV count. The list gives no documented fixed update cadence; measure response metadata and disclose an unknown cadence.


## Identity and source ownership

| Fact | Source identity and admissible join | Forbidden substitution |
| --- | --- | --- |
| Laboratory test | Complete GTR accession plus version. The exact same accession/version joins GTR reports. A source-given laboratory ID remains separate from CLIA and test ID. | Picking a latest version silently, treating a CLIA number as approval, or selecting a test from its name. |
| Gene or condition tested | Preserve native relation object type, object ID, namespace, label and test scope if the captured tier carries them. Native NCBI Gene IDs can feed exact gene retrieval; a native MedGen/OMIM ID cannot become MONDO without an existing exact source cross-reference. | Converting a panel membership into variant coverage, clinical sensitivity or an indication. Symbol equality is a source-field filter, not a cross-provider identity proof. |
| WHO product | WHO Product ID identifies the WHO listing; preserve product-code arrays and regulatory version as distinct fields. A code is not globally unique until capture establishes its namespace and multiplicity. | Flattening several codes into one identity or equating WHO prequalification to FDA approval. |
| FDA PMA record | Pair native `pma_number` with native `supplement_number`, retaining absent versus empty and original decision metadata. Base application pages contain distinct record events. | Dropping supplement identity or using one supplement's indication for every record in its base application. |
| FDA 510(k) record | Native `k_number`, decision code and description. | Calling a clearance a PMA approval or merging records using product code. |
| Companion relation | Source table row/snapshot owns device application/supplement, drug application(s), indication/sample and biomarker details together. Link to openFDA only by exact application and supplement, and to Drugs@FDA only by exact native NDA/BLA application. Preserve every therapy in a combination. | Treating the row as a generic gene–drug edge, joining a device name to GTR/WHO, inferring a protein/variant identity from a biomarker cell or equating disease labels to ontology IDs. |
| Variant follow-up | Only a reference-qualified source variant identity with source assembly/accession and sufficient allele context can form an existing typed variant request. No such link is established by documents alone. | A gene symbol, exon class, protein expression category, TMB threshold or “fusion” label becoming a chosen genomic allele. |

A source-specific diagnostic record can be a laboratory test, marketed IVD product or device submission. Model these as a tagged union. Keep each source result separate even when its display name matches. A shared product identity, global approval flag, merged sensitivity figure and inferred drug relevance are absent. `now_current` means the current version, not proof the test remains an active clinical offering; retain `test_currStat`, `test_pubStat` and laboratory status independently. The gene design moved diagnostics to this entity; the disease design dropped its diagnostic fanout. This investigation proposes explicit diagnostic requests and no automatic restoration of those sections. Variant and drug commands keep their established source identities and query ownership.

## Use labels and source selection

| Candidate source or component | Proposed use and disable group | Terms and access boundary |
| --- | --- | --- |
| GTR public reports | `open`; no group alias | [NCBI usage policies](https://www.ncbi.nlm.nih.gov/home/about/policies/) place government work in the public domain but retain third-party rights and disclaim endorsement. Preserve submitter provenance. No OMIM narrative or licensed linked document is downloaded or redistributed by this design. |
| openFDA PMA and 510(k) | `open`; no group alias | [openFDA terms](https://open.fda.gov/terms/) provide CC0 with explicit third-party exceptions. Do not expose unrelated harmonized GMDN licensed content. Source-native decision fields carry their FDA attribution. |
| FDA companion table | `open`; no group alias, approved by the PM | [FDA website policy](https://www.fda.gov/about-fda/about-website/website-policies). Preserve regulatory-table authorship and any marked third-party exception. This is a separate approved table bundle; openFDA does not expose its rows. |
| WHO IVD export | `noncommercial`; existing `noncommercial` group | [WHO website terms](https://www.who.int/about/policies/terms-of-use) reserve commercial/substantial reproduction rights; [permissions](https://www.who.int/about/policies/publishing/permissions) distinguish content-specific licences. Do not label the whole CSV CC BY merely because some WHO publications use that licence. The export supplies no narrower licence; the WHO terms page therefore supports the qualified `noncommercial` category. This does not claim that all WHO publications share a single licence. |

Every implemented source starts enabled under the existing open-by-default decision. Reuse the registry's terms, source disable and component filtering; do not create a second runtime policy table. Disabled sources supply neither requests nor cached/replayed content. A failed or skipped source still carries use/terms. WHO's noncommercial category is visible before its rows. Source access is keyless where the publisher documents public access. Optional openFDA user keys only raise quota; do not create a diagnostic-specific credential path. [Authentication](https://open.fda.gov/apis/authentication/) contains inconsistent introductory wording but explicitly lists no-key and keyed quotas; capture follows the existing approved optional-key ruling. No keyed-only source is proposed.

## What would delight the agent

1. “Which listed companion devices support this drug application?” One explicit FDA companion-table request returns source rows with exact device/submission identity, tumor, specimen and biomarker context. It makes no laboratory recommendation.
2. “Which laboratory tests list this gene?” An explicit GTR request returns native test headers and the reason they matched. It does not promise a variant is assayed or the lab currently accepts a specimen.
3. “What makes this record worth opening?” The smallest header has source, native ID/version, native name, record kind, laboratory/manufacturer when available and source link. A short native marker/type field can appear only when bounded and measured.
4. “What should I do next?” Each row names its exact get. A card advertises only source-supported sections. A complete native list remains reachable through paged sections; no display clipping loses rows. A page action retains source, every selector and bundle revision.
5. “Can I trust the approval link?” FDA application and supplement IDs remain visible beside each dated decision and companion relation. A name similarity never earns an approval link. No shared identity is claimed when the source supplies none.
6. “Is this assay suitable for this exact tumor, sample and alteration?” Preserve the source's full scope behind the companion section. Return the attributed scope; leave suitability and clinical interpretation to the clinician.
7. “Did the source answer or fail?” A successful zero states only the selected dataset/query had no rows. Missing bundle, disabled source, malformed body, timeout, HTTP failure and unsupported tier stay distinct source outcomes. No alternate query hides a zero or failure.
8. “What is fast?” Stored bundle reads make zero provider calls and do not synchronize. The retained source-query timings are measured below; product default sizes are measured in the budget table. No speed promise follows from historical experiments.
9. “What does the old product miss?” Its name-derived FDA overlay cannot prove identity, disease synonym expansion changes the request and the optional full JSON gene/condition arrays can overwhelm a discovery page. Keep source inventory and explicit paging; remove semantic guessing.
10. “When can I stop?” The first page can answer that a specific source lists a device, gene or application. It cannot establish clinical equivalence, universal test coverage or current authorization across jurisdictions. Source revision, retrieval date and original link let the agent disclose that limit.

## Reviewed types, schema and source registry

These records describe inventory and regulatory rows. Companion relations and GTR membership relations are not clinical `Statement` values. Reuse `Concept` and `Curie` for explicit MedGen/OMIM/SNOMED source concepts, and `GeneRef` with `NcbiGeneId` for source-stated genes. A condition's `GTR_identifier` maps to `Concept.id = MedGen:C…`; preserve its source label. A gene's numeric `gene_or_SNOMED_CT_ID` maps to `GeneRef.ncbi_gene_id`, while `gene_symbol` supplies its label. The same column on a condition row is SNOMED CT and never a gene ID. Keep raw native identifiers beside these typed values when they explain the mapping. A label alone does not acquire a Curie or gene ID. No inferred disease ontology conversion, variant normalization, drug equivalence or therapeutic classification is added.

`biodata` owns the no-I/O `GtrTestVersion`, `GtrTestRelation`, `WhoIvdProduct`, `DeviceSubmission` and `CompanionDiagnosticRelation` records and their checked native IDs. `biomcp-types` owns requests, the tagged record-kind views, page limits, source outcomes, status and typed next actions. Generate the diagnostic page/card/relation/method/regulatory/companion schemas once from those types; reuse existing sync and envelope contracts. Optionality follows native empty cells versus missing fields, not a second JSON guess. No new schema or product type lands in this design ticket.

Add each new source once to the canonical `Source` enum and the existing registry, using the same atom in CLI selection, query echo, response, health, replay/cache keys and disable settings: `Gtr => gtr`, `WhoIvd => who-ivd`, `FdaCdx => fda-cdx`, `FdaPma => fda-pma`. The registry declares each once; never invent per-command aliases such as `GTR` or `openfda-device`. `Fda510k => fda-510k` is implemented for one exact K-number clearance; it has no free-text search. Existing drug pivots use the already registered `drugsfda`, not a new FDA drug source. Terms and use labels derive from the registry declarations above.

### Header, card and page field lists

All JSON sketches below are proposed shapes, not measured product output. A string beginning `$` denotes a native value or an existing envelope value to be supplied, never a literal placeholder shipped to users. Common `schema`, exact checked `query`, `status`, source outcomes/terms, `problems`, `more` and typed `next` wrap every view through the existing response contract. Omit unrequested sections and absent optional fields. Preserve native explicit empty values inside a native regulatory record when they distinguish a base application.

| Record kind | Header fields | Card additions and named pages |
| --- | --- | --- |
| GTR test | `source`, `kind`, exact `id` including version, optional `name` from `lab_test_name`, `identity_status`; laboratory name/institution/country when supplied. | Native `last_touch_date`, separate current-version/test-current/test-public/lab-current/lab-public flags, submitted manufacturer name, CLIA, state licences, lab ID/test code and creation/deletion dates where supplied. `genes`, `conditions`, `methods` are separate paged sections. Header never expands the full `genes` pipe cell. |
| GTR conflict | `source`, `kind`, exact conflicting `id`, `identity_status=conflict`, `problem`. | No card. Search shows one conflict row, no selected name or get step. Get reports identity conflict and supplies no card. Sync receipt reports duplicate-identity count. Raw rows remain in the bundle conflict inventory. |
| WHO product | A bounded product header retains the exact ID and the name, manufacturer and marker cells from the first matching row. Repeated code rows group by ID in first-product source order. | Exact `WHO Product ID`, whole product name, every native `Product Code` row, assay format, manufacturer, marker, regulatory-version text and prequalification year. Card only; no genes/conditions/methods/regulatory expansion or inferred FDA link. All 435 rows/125 products stay available unfiltered. |
| FDA PMA | No free-text search command. Supplement-page header: exact `pma_number`, `supplement_number`, native `trade_name`, applicant, decision date/code and source link. | Base get returns the native base row with empty supplement number. Explicit supplement get returns only the exact native supplement row. `regulatory` exposes that row's decision/received/notice dates, committee, product code, native supplement type/reason and `ao_statement`. Base-only `supplements` pages the remaining rows. No device-wide approval inference. |
| FDA companion relation | `source`, `kind`, snapshot/table/physical-row identity, whole diagnostic-name cell and complete indication/sample plus drug/device identity context. | Page rows preserve every whole native cell and original cell links. Each row holds `cells` keyed by its own table's exact headers, `device_ids` and `drug_applications` only when owned cells/links establish them. No narrative row is split into inferred claims or gene–drug edges. The two table kinds remain distinct. |

GTR card and gene relation sketches:

```json
{"kind":"gtr_test","source":"gtr","id":"GTR000006692.3","identity_status":"resolved","name":"$lab_test_name","laboratory":{"id":"$lab_id","name":"$name_of_laboratory","institution":"$name_of_institution","country":"$facility_country"},"native_status":{"now_current":"$now_current","test_currStat":"$test_currStat","test_pubStat":"$test_pubStat","lab_currStat":"$lab_currStat","lab_pubStat":"$lab_pubStat"},"last_touch_date":"$last_touch_date","more":["genes","conditions","methods"]}
```

```json
{"kind":"gtr_genes","id":"GTR000006692.3","rows":[{"test_type":"$test_type","gene":{"ncbi_gene_id":"672","label":"BRCA1"},"native_object":"gene","native_medgen":"$GTR_identifier"}],"page":{"limit":5,"offset":0,"returned":1,"total":1,"unit":"native_relation_rows"}}
```

Condition rows use `{"condition":{"id":"MedGen:C2676676","label":"Breast-ovarian cancer, familial, susceptibility to, 1"},"test_type":"$test_type"}` and optional separately namespaced MIM/SNOMED refs. The actual singleton relation supplies the positive native CUI; no literal dollar value passes checked `Curie`. Method rows preserve native method categories/methods and source index; do not pair parallel pipe arrays by position unless the source documents that relation.

WHO exact product-card sketch:

```json
{"kind":"who_ivd_product","source":"who-ivd","id":"0466-046-00","name":"$Product name","native_rows":[{"product_code":"$Product Code","assay_format":"$Assay Format","manufacturer":"$Manufacturer name","marker":"$Pathogen/Disease/Marker","regulatory_version":"$Regulatory Version","prequalification_year":"$Year prequalification"}],"native_row_total":9}
```

`native_rows` contains all nine original rows for this selected product, in source order; the single displayed row is a shape example, not a complete data instance. No default clinical filtering removes WHO products, codes or marker rows.

PMA base/supplement and companion-page sketches:

```json
{"kind":"pma_submission","source":"fda-pma","id":{"pma_number":"P150013","supplement_number":"S020"},"name":"$trade_name","applicant":"$applicant","more":["regulatory"],"native":{"decision_date":"$decision_date","decision_code":"$decision_code"}}
```

```json
{"kind":"companion_page","source":"fda-cdx","bundle_revision":"$content_revision","rows":[{"row":{"table":0,"index":"$physical_row"},"diagnostic_name":"$whole_diagnostic_cell","indication_sample":"$whole_indication_cell","drug_cell":"$whole_drug_cell","biomarker":"$whole_biomarker_cell","biomarker_details":"$whole_details_cell","device_cell":"$whole_device_cell","device_ids":[{"pma_number":"P150013","supplement_number":"S020"}],"drug_applications":[{"id":"BLA125514","url":"$cell_owned_source_link"}]}],"page":{"limit":5,"offset":0,"returned":"$returned","total":"$exact_checked_rows"}}
```

Group-table rows have their four actual cells, including the complete group indication/therapy text, rather than invented biomarker columns. The base PMA card uses `supplement_number=""` from the captured base row and advertises both `regulatory` and `supplements`; a supplement card never advertises its own supplement as an application-wide status. Response schema fields stay typed even where examples use placeholders.

## Reviewed selectors and command tree

| Command or selector | Final design disposition and native proof | Disclosure |
| --- | --- | --- |
| `search diagnostic --source gtr --gene-id 672` | Proven: native relation `object=gene`, exact `gene_or_SNOMED_CT_ID=672`. Exclusive with `--gene`. | Bounded test headers; preserve source association and test type. |
| `search diagnostic --source gtr --gene BRCA1` | Proven: exact `gene_symbol` value on native gene rows. Do not match `object_name`, pipe substrings or a local synonym list. | Same candidate headers; a symbol is a field filter, not a selected HGNC identity. |
| GTR `--condition-id MedGen:C2676676` | Proven by the retained singleton condition row (`GTR000006692.3`): `object=condition`, exact `GTR_identifier`. Parser accepts only MedGen CUI in this selector. | Native test candidates; no label-to-ontology resolver. The positive case is exact MedGen C2676676, source label “Breast-ovarian cancer, familial, susceptibility to, 1”, native MIM604370; the native SNOMED field is empty. |
| GTR `--type VALUE` | Proven native relation `test_type`: Clinical, Research and the third native microbial category as returned; preserve exact source value. | Conjunctive exact whole field filter. A source category does not prove availability. |
| Search `--limit`, `--offset` | Keep; source-owned order and mechanical local pages. Default five rows. | Preserve every selector and immutable bundle revision on continuation. |
| GTR `--laboratory`; WHO `--type` | Deferred. WHO manufacturer words are implemented by the later stored-search addition. | No advertised aliases or best-effort substring matching. |
| WHO search/marker filters | Implement literal native-column WHO search over the synced export. | Product headers link exact cards; no inference, extra sections or automatic download. |
| `search diagnostic --source fda-cdx --drug-application NDA…` | Select only rows whose own drug-cell link carries the exact checked application, with NDA/BLA type established by that owned cell. No match from a number elsewhere in the row. Rows without an owned typed ID remain reachable through device search. | Five complete source relation rows; every combination/group context remains whole. |
| FDA CDx `--drug-cell` | Dropped; no harvested question needs it. | Whole drug cells remain displayed source facts, not a search option. |
| FDA CDx `--biomarker TEXT` | Proven exact table-0 cell: captured P110027/S012 has `Biomarker(s)` equal to `KRAS`, `Biomarker(s) (Details)` equal to `G12C`, and the drug cell/link names Lumakras NDA214665. The positive case asserts `--biomarker KRAS` includes this exact row with its whole NSCLC-tissue context. Group table has no biomarker column and is explicitly unsupported for that selector. | No substring matching or parsing of genes, alteration, score or cutoff into independent clinical assertions. |
| `search --source fda-pma --query`; FDA 510(k) search | Deferred. Captures prove exact records, not a free-text query contract. | DEN identifiers remain source facts without executable gets. |
| `get diagnostic K173492 --source fda-510k` | Exact native `k_number` with `limit=1`, `skip=0`; equality and complete singleton checked before cache admission. | Native clearance fields, never a PMA approval. An exact `fda-cdx --device-id K173492` association step returns to the companion table. |
| `get diagnostic GTR_ACCESSION.VERSION --source gtr [genes|conditions|methods]` | Exact accession/version; one optional named section and section pagination. | Default card only. Missing identity/conflict prevents selected card and cache admission as a chosen record. |
| `get diagnostic P150013 --source fda-pma` | Exact base row with native empty supplement number, never the first arbitrary API result. | Base card. `regulatory` opens its native decision text; `supplements` opens five native supplement headers with exact selectable supplement IDs. |
| `get diagnostic P150013 --source fda-pma --supplement S020 [regulatory]` | Exact base-plus-supplement equality. No omitted-supplement fallback and no inferred latest supplement. | Exact supplement card; regulatory section belongs only to this row. `supplements` with `--supplement` is a usage error. |
| `search diagnostic --source fda-cdx --device-id P150013[/S020]` | Exact device identity from the device cell or its own link; base form selects that application’s rows, supplement form selects only that exact supplement. No card get or silent row choice. | Association page of complete scoped table rows; table/snapshot/row provenance retained. |
| `sync gtr`, `sync who-ivd`, `sync fda-cdx` | Explicit validated whole replacement; shared sync receipt. | No read-triggered download or refresh. GTR receipt includes duplicate-identity count. |
| `help diagnostic`, diagnostic schemas | Existing generated request/schema conventions. | Parsed real examples for every advertised selector/section; no source or section list maintained separately. |

GTR and PMA named section pages use `--section-limit`/`--section-offset`; FDA CDx association searches, including exact-device search, use `--limit`/`--offset` because the whole response is the page. Those spellings remain explicit in each typed continuation. A GTR query may combine `--type` with one exact relation selector. `--gene-id` and `--gene` are exclusive. The condition selector is not an alternate spelling of a gene selector; invalid namespace or source-inapplicable options fail before I/O. No generic `all` source/section is proposed. Sources and checked request types supply the same CLI/MCP behavior; Markdown renders those responses.

### Exact FDA table contract and owned identifiers

The PM approved `fda-cdx` because fixed-column regulatory tables supply facts absent from the API. This does not extend SEER's no-narrative-HTML rule. Sync requires exactly two tables with these exact ordered header values, including captured internal line breaks. Any missing, additional, reordered or changed header fails the whole replacement and preserves the prior usable bundle. Preserve all original cell text and links; no local biological interpretation, trademark cleanup or narrative scraping.

Table 0 header array:

```json
["Diagnostic Name (Manufacturer)","Indication - Sample Type","Drug Trade Name (Generic) NDA / BLA","Biomarker(s)","Biomarker(s) (Details)","PMA / 510(k) /\n513(f)(2) / HDE (Approval / Clearance / Grant Date)"]
```

Table 1 header array:

```json
["Diagnostic Name\n(Manufacturer)","Indication(s) - Sample Type","PMA (Approval Date)","Device Indication for a Specific Group of Oncology Therapeutic Products and Trade Name (Generic) – NDA/BLA"]
```

These match `offline/fda-physical-tables.json`; table 0 has 232 data rows and table 1 has nine. Source date text remains in its original cell until a mechanical date grammar proves it; the relation never borrows an API date. Retain a source snapshot revision plus table index and physical row index as derived row provenance, not a fabricated publisher record ID.

For example, the table-0 P200010/S026 row's own drug cell says `BRAFTOVI (encorafenib) NDA 210496 in combination with ERBITUX (cetuximab) BLA 125084`, with two owned FDA links carrying application numbers 210496 and 125084. Its device cell says `P200010/S026` and links directly to `pma.cfm?id=P200010S026`. The P150013/S020 row's drug cell says `Keytruda (pembrolizumab) BLA 125514` and its owned link carries 125514. These permit exact checked spellings `NDA210496`, `BLA125084` and `BLA125514` for the existing `get drug ID --source drugsfda` request, preserving six digits and any leading zero. The drug request is a source lookup; the table relation does not guarantee that Drugs@FDA will answer it. Never infer NDA versus BLA from the six digits or a drug name.

Read IDs only from their dedicated cell's standard identifier text and its own published link parameter (`ApplNo`, `varApplNo`, PMA `ID`), with a checked published lexical grammar. Where cell text and link disagree, return an identity problem and preserve the whole cell; do not pick one. Where group text supplies several NDA/BLA references, keep the entire group cell and all owned links; retain exact application pivots only when each type/number is unambiguous. Do not reconstruct which therapies form a regimen by parsing prose. If typed extraction is not possible, retain the whole cell and its source links, omit the drug pivot, and reach the row through its exact `--device-id` search. No whole-drug-cell selector is offered.

The retained companion table also carries K173492 in its own device cell/link and multiple DEN identifiers. The PM authorized one exact-ID request for a captured K number in the later adapter ticket. This design runs no request. The later K173492 receipt supplies that positive for the exact 510(k) implementation below. DEN identifiers remain source facts/links without executable gets. A failed K080653 probe is not a reason to spend another request here.

### Per-command next steps under architecture 6.10

One pure diagnostic `next` function spells existing typed actions. Source values never become shell fragments. Candidate pages emit one `disambiguate` get per nonconflicting selectable row, then the page step, then at most three other actions. Association pages emit one `pivot` per distinct linked record in source order, then page, then at most three extras. Other cards/receipts obey the existing maximum-five rule. Page limits bound candidate and association actions; do not arbitrarily truncate the linked records to five. Deduplicate only identical typed requests.

| Response | Ordered next actions |
| --- | --- |
| GTR search | Exact accession/version gets for resolved candidates; skip conflict rows. Next page retains exact selectors, limit and bundle revision. Up to three extras can name sync/fresh only when applicable. |
| GTR card | Source-supported `genes`, `conditions`, `methods` section steps, followed by applicable sync/fresh within five total. No automatic disease or FDA call. |
| GTR gene association page | `get gene 672` and other exact source-given `NcbiGeneId` pivots, once per distinct ID in source order; page next; at most three extras. Label-only rows have no gene get. |
| GTR condition/method page | No disease pivot without an existing exact supported disease namespace/cross-reference. Native MedGen rows remain `Concept` values. Page first when there are no executable linked records; no name lookup. |
| WHO card | No inferred gene/drug/FDA pivots or sections. Applicable sync/fresh actions only, within five. All native rows are on the exact selected product card. |
| CDx association search page | One distinct exact PMA/base-plus-supplement pivot to `get diagnostic … --source fda-pma`, and each distinct unambiguous native NDA/BLA drug pivot to `get drug … --source drugsfda`, in native row/link order; then page; at most three extras. Treat relation pages as association pages, not candidate-card guesses. Each distinct exact K identity gains its `fda-510k` get pivot. DEN/H and ambiguous cells stay source links only. |
| PMA base card | `regulatory`, `supplements`, and optional exact `search diagnostic --source fda-cdx --device-id P150013` relation step, then applicable sync/fresh within five. |
| PMA supplement card | Exact row's regulatory section and exact `fda-cdx --device-id P150013/S020` association search step, then applicable sync/fresh within five. No step broadens the selected supplement's claim to the base application. |
| PMA supplements page | One exact base-plus-supplement get per header, then page with original base identity; at most three extras. |
| PMA regulatory section | Text continuation retains base/supplement, exact native field and offset. Any extra applicable sync/fresh steps remain within the ordinary five-step bound. |
| Sync/missing/stale bundle | Existing sync and fresh semantics; failed replacement never suggests a successful new revision. Missing GTR/WHO/CDx bundle names its explicit sync. No read silently refreshes. |

## Progressive disclosure and measured product budgets

The implemented defaults are five GTR search headers or native section rows, five FDA companion rows and five PMA supplement headers. Exact get returns one source-owned identity card. WHO returns every row under one known Product ID. PMA regulatory output starts with one bounded mechanical text fragment. Section offsets count rows or fragments, never characters. Five-row pages never establish a complete assay scope or a clinical recommendation.

The product budget table below replaces the investigation's provisional source projections. Complete envelopes include source terms, status and typed next actions. Native measurements retain their original source scope; synthetic fixtures prove mechanics separately. Byte ceilings use the owning runner's measured bytes plus fifteen percent, rounded up. The YAML runner subtracts only registry-validated `use`, `terms` and `terms_url` for comparable limits. Full compact bytes and stdout bytes, which add a final newline, are distinct. Approximate tokens are `ceil(bytes/4)`, not tokenizer counts.

Released diagnostic 0.9 bytes, tokens, cold/warm timings, request counts and same-question factual completeness remain unmeasured. The old executable has not run under a verified diagnostic containment recipe. The earlier drug/disease containment does not supply a diagnostic baseline. Ticket 0098 owns that separate comparison. New FDA source routes also lack a corresponding old source-specific command. No source-only projection or local replay timing is a product or provider-speed measurement.

### Native product envelope measurements

These are retained product measurements, not source projections. Full and comparable columns contain compact UTF-8 bytes. The complete stdout ceiling includes the newline and fifteen percent headroom. GTR maximum-size walks use fifty rows explicitly and do not establish the default five-row size. Maximum means largest observed within the stated deck, never a global bound across the source.

| Product view and measured scope | Full / comparable compact bytes | Full stdout ceiling | Approximate full tokens | 0.9 comparison |
| --- | ---: | ---: | ---: | --- |
| GTR default-five candidate page, largest observed native slice page | 4017 / 3867 | 4621 | 1005 | Not run: 0098; slice has eight versions and 122 relations. |
| GTR default-five genes, retained native slice panel | 2014 / 1864 | 2318 | 504 | Not run: 0098; slice panel has ten selected gene rows. |
| GTR default-five conditions, retained native slice panel | 1594 / 1444 | 1835 | 399 | Not run: 0098; slice panel has ten selected condition rows. |
| GTR default-five methods, retained native slice singleton | 1251 / 1101 | 1440 | 313 | Not run: 0098; five of six method entries. |
| GTR full sync, both complete reports | 330 / 180 | 381 | 83 | Not run: verified diagnostic containment and same-question comparator belong to 0098. |
| GTR Gene 672 candidate page, largest of complete 433 walk, limit 50 | 37115 / 36965 | 42684 | 9279 | Not run: 0098. |
| GTR condition candidate page, largest of complete 127 walk, limit 50 | 35564 / 35414 | 40900 | 8891 | Not run: 0098. |
| GTR card, largest of three selected records | 1684 / 1534 | 1938 | 421 | Not run: 0098. |
| GTR broad-panel genes, largest complete 4668-row walk page, limit 50 | 14540 / 14390 | 16723 | 3635 | Not run: 0098. |
| GTR broad-panel conditions, largest complete 5134-row walk page, limit 50 | 10311 / 10161 | 11859 | 2578 | Not run: 0098. |
| WHO full sync | 350 / 157 | 404 | 88 | Not run: 0098. |
| WHO nine-row 0466-046-00 card, largest of four harvested products | 2915 / 2722 | 3354 | 729 | Not run: 0098. |
| FDA CDx full two-table sync | 310 / 158 | 358 | 78 | No old source-specific FDA table command; executable comparison not run. |
| FDA CDx default first page, five whole rows | 6336 / 6184 | 7288 | 1584 | No old source-specific FDA table command; 0098 comparison deferred. |
| FDA CDx default second page, largest of first two unfiltered pages | 7106 / 6954 | 8174 | 1777 | Not a maximum over all 241 rows; no old source-specific command. |
| FDA CDx BLA125514, largest page of complete 15-row walk | 7049 / 6897 | 8108 | 1763 | No old source-specific command. |
| PMA base card, largest of P150013/P200010 | 999 / 897 | 1150 | 250 | No old exact source-specific PMA command. |
| PMA exact S020/S021 card | 970 / 868 | 1117 | 243 | No old exact source-specific supplement command. |
| PMA largest native five-supplement page | 2518 / 2416 | 2897 | 630 | No old section with this request contract. |
| PMA largest retained one-fragment regulatory page | 3258 / 3156 | 3748 | 815 | Old name overlay is not an exact-submission comparator; 0098 deferred. |
| Native K173492 clearance card | 758 / 643 | 873 | 190 | No old exact source-specific clearance command. |
| Native K080653 source-local absence | 281 / 166 | 325 | 71 | No old exact source-specific clearance command. |

The earlier GTR slice proof j12984 passed 35 CLI/MCP pairs using unchanged native rows selected from the retained reports: eight versions and 122 relations, including only ten panel gene and ten panel condition rows. Its default-five values above prove those slice requests, not full-report default continuation overhead or a full-source maximum. Exact BRCA1 and Gene672 default slice pages measured 3920/3770 and 3921/3771 full/comparable compact bytes; the displayed 4017-byte maximum is the Clinical type page. The method continuation measured 654/504 compact bytes for its final one entry. The archive is `data/biomcp-redux/0172-gtr-native-slice-proof/j12984-native-slice-proof.tar.gz`, SHA-256 `3b28057461bf299cc1a9ab2c731b78eca8b6823288f2ff53ce5226eec1232d7f`. All raw measurements were checked against the archived CLI output without product execution.

GTR j13177 completed every CLI walk; CLI/MCP parity was sampled at first, second and terminal pages. Its largest intermediate pages were not individually MCP calls. WHO j13094 checked nineteen CLI/MCP pairs. FDA j13117 checked twenty native plus nine synthetic pairs before an obsolete help check failed; the help defect was fixed separately. Only two unfiltered FDA pages were measured, so the table does not claim a global five-row maximum. FDA comparable values above were recalculated from retained output using the current runner's validated three-field deduction; the original proof's recorded comparable values retained `use` and were thirteen bytes larger. PMA native comparables retain that proof's declared terms deduction; its complete envelope sizes and ceilings remain unchanged.

The GTR full-read artifact is `data/biomcp-redux/0179-gtr-full-product-proof/2292-0179-gtr-reads-final.tar.gz`, SHA-256 `8ff658e9d10f5ef335c5da5627ce55d7bf3a8d79b41b732a454e6f2350525d6a`. WHO product evidence is `data/biomcp-redux/0173-who-ivd-native-proof/j13094-native-proof.tar.gz`, SHA-256 `0a9f6392b9eb4c292b277296f6bc630734379609dea7e5c5e8deb82f0061c4f3`. FDA table product evidence is `data/biomcp-redux/0174-fda-cdx-product-proof/j13117-product-proof.tar.gz`, SHA-256 `aea5e17019f7d6a3357f9f8e3e32f2e58dac9f0937ae34933710e576872f7ea4`. Their raw outputs and manifests remain outside Git and were independently verified before scratch cleanup.

## Harvested clinical questions and accepted answers

A fresh clinical reviewer checked these public-source questions on 2026-10-25. They paraphrase real regulatory, guideline and registry questions; they are not invented patient cases or verbatim clinician quotations. No earlier diagnostic-specific ruling existed. Findings are confirmed with high confidence except Q17's record contents, which remain open, and Q18's explicitly inferred response rule. Clinical correctness does not imply provider-tier completeness. Dates identify the cited decision or guideline; they do not imply that a historical statement is the complete current label.

| Case | Sourced diagnostic question | Accepted answer and boundary |
| --- | --- | --- |
| Q01 | Which companion assay supports alpelisib selection in breast cancer? [FDA P190004, 2019-05-24](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfPMA/pma.cfm?ID=P190004) | therascreen PIK3CA RGQ PCR Kit, with its specified mutation set and tissue/plasma scope. Gene membership alone does not prove coverage of every alteration. |
| Q02 | Does a negative therascreen plasma result establish absence in tumor? [Same decision](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfPMA/pma.cfm?ID=P190004) | No. Preserve plasma-negative scope and the source's tissue-testing qualification. |
| Q03 | Which ESR1 alterations support the original Guardant360–elacestrant claim? [FDA P200010/S010, 2023-01-27](https://www.fda.gov/medical-devices/recently-approved-devices/guardant360-cdx-p200010s010) | Plasma ESR1 missense alterations at codons 310–547; do not expand to every alteration or specimen. |
| Q04 | Does an ESR1 result alone establish the complete elacestrant indication? [FDA drug approval, 2023-01-27](https://www.fda.gov/drugs/resources-information-approved-drugs/fda-approves-elacestrant-er-positive-her2-negative-esr1-mutated-advanced-or-metastatic-breast-cancer) | No. Preserve the test relationship separately from tumor, population and prior-treatment restrictions. |
| Q05 | Which assay supports encorafenib plus cetuximab in colorectal cancer? [FDA P190026, 2020-04-15](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfpma/pma.cfm?id=P190026) | therascreen BRAF V600E RGQ PCR Kit; keep V600E, colorectal FFPE tissue and combination therapy together. |
| Q06 | Does therascreen KRAS's sotorasib claim include G12D? [FDA P110027/S012, 2021-05-28](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfpma/pma.cfm?id=P110027S012) | This companion claim specifies G12C, not G12D. A gene-level search cannot establish the alteration claim. |
| Q07 | Is there a tissue companion listing for NTRK fusions and larotrectinib? [FDA companion list, P230011, 2024-08-21](https://www.fda.gov/medical-devices/in-vitro-diagnostics/list-fda-authorized-companion-diagnostic-devices-in-vitro-and-imaging-tools) | TruSight Oncology Comprehensive lists that scoped fusion relationship; do not substitute arbitrary NTRK point mutations. |
| Q08 | What ALK assay is listed for crizotinib in NSCLC tissue? [Same list, P140025, 2015-06-12](https://www.fda.gov/medical-devices/in-vitro-diagnostics/list-fda-authorized-companion-diagnostic-devices-in-vitro-and-imaging-tools) | Ventana ALK D5F3 CDx with protein-expression context. That observation is distinct from an exact genomic rearrangement. |
| Q09 | Which 22C3 threshold applies to the listed pembrolizumab TNBC relationship? [Same list, P150013/S020, 2020-11-13](https://www.fda.gov/medical-devices/in-vitro-diagnostics/list-fda-authorized-companion-diagnostic-devices-in-vitro-and-imaging-tools) | CPS ≥10 in the stated tissue/tumor context; do not substitute TPS. |
| Q10 | Is the 22C3 cemiplimab NSCLC threshold also CPS ≥10? [Same list, P150013/S021, 2021-02-22](https://www.fda.gov/medical-devices/in-vitro-diagnostics/list-fda-authorized-companion-diagnostic-devices-in-vitro-and-imaging-tools) | No. The listed threshold is TPS ≥50%. The assay name cannot select the scoring rule. |
| Q11 | Does Ventana MMR RxDx have a dMMR solid-tumor pembrolizumab claim? [FDA P210001/S001, 2022-03-21](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfpma/pma.cfm?id=P210001S001) | Yes. Preserve the panel-status claim; it is not eligibility from any single MMR-gene variant. |
| Q12 | Does panel approval make every reported finding a companion finding? [FDA P230011, 2024-08-21](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfpma/pma.cfm?id=P230011) | No. Tumor profiling and specific companion indications remain distinct. |
| Q13 | Does negative plasma Guardant360 EGFR rule out a tumor alteration? [FDA P200010, 2020-08-07](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfpma/pma.cfm?id=p200010) | No. Preserve specimen scope and the decision's tissue-reflex limitation. |
| Q14 | Can BRCA1 gene search find laboratory test listings? [GTR, living page checked 2026-10-25](https://www.ncbi.nlm.nih.gov/gtr/tests/) | Yes, as submitted test candidates with source identity and laboratory. It does not establish complete variant coverage. |
| Q15 | Does a GTR BRCA1 listing establish FDA approval or NIH endorsement? [Same GTR disclaimer](https://www.ncbi.nlm.nih.gov/gtr/tests/) | No. Require an independent authorization record for the regulatory claim. |
| Q16 | Can every GTR test be presented as a clinical offering? [Same GTR use categories](https://www.ncbi.nlm.nih.gov/gtr/tests/) | No. Preserve clinical versus research category. |
| Q17 | What can be verified about a BRCA1 listing when its exact page cannot be read? [GTR test 21517](https://www.ncbi.nlm.nih.gov/gtr/tests/classic/21517/), browser challenge observed 2026-10-25 | Report unavailable/unverified exact record contents. Indexed snippets are not verified current source facts. This is a web access observation, not an API outage capture. |
| Q18 | Does an empty BRCA1 registry query prove no test exists? [GTR scope/disclaimer](https://www.ncbi.nlm.nih.gov/gtr/tests/) | No. “No matching records returned for this source/query” is the working response rule, likely/high confidence from registry limits. No actual empty BRCA1 query was claimed. |
| Q19 | Can breast HER2 IHC 1+ become an undifferentiated negative? [ASCO/CAP update, 2023-06-07](https://www.cap.org/cap-guidelines/her2-testing-in-breast-cancer-2023-guideline-update/) | Preserve numeric IHC score and the applicable ISH result separately. This dated terminology does not determine current therapeutic eligibility. |
| Q20 | Are NGS MSI and MMR IHC universally interchangeable? [CAP MMR/MSI guideline, 2022-08-03](https://www.cap.org/cap-guidelines/mismatch-repair-and-microsatellite-instability-testing-for-immune-checkpoint-inhibitor-therapy/) | No blanket equivalence. Preserve method and tumor context. Performance in one group does not establish another. |
| Q21 | Does “KRAS tested” represent extended RAS coverage? [2017 joint guideline, DOI 10.1093/ajcp/aqw209](https://pmc.ncbi.nlm.nih.gov/articles/PMC7263311/) | No. Preserve actual KRAS and NRAS regions. This dated coverage distinction is not a current treatment algorithm. |
| Q22 | Does a harmful BRCA1 tumor finding establish inheritance? [NCI fact sheet, reviewed 2024-07-19](https://www.cancer.gov/about-cancer/causes-prevention/genetics/brca-fact-sheet) | No. Germline status stays unresolved without independent evidence. |
| Q23 | Are WHO HPV diagnostic resources relevant to cancer care? [WHO NAAT policy, 2022-03-16](https://www.who.int/publications/i/item/9789240045248), [screening update, 2021-07-06](https://www.who.int/news/item/06-07-2021-new-recommendations-for-screening-and-treatment-to-prevent-cervical-cancer) | Yes, for source-defined cervical screening/prevention. Exact card proof answers Q23 for a known WHO Product ID, such as 0466-046-00. Separate WHO search discovers products by native marker or manufacturer words. This establishes neither a cancer diagnosis nor a drug companion claim. |
| Q24 | Does WHO eligibility make every infectious assay an oncology companion diagnostic? [WHO eligibility scope, checked 2026-10-25](https://extranet.who.int/prequal/vitro-diagnostics/vitro-diagnostics-eligible-who-prequalification) | No. Eligibility, completed prequalification and FDA authorization remain distinct. Preserve all WHO bundle rows without inferring an oncology companion relationship. |

The reviewer rejected gene-only eligibility, name-based device identity, method equivalence and tumor-to-germline inference. Exact source label changes, new specimen scope or a readable previously unavailable record can change a scoped ruling; no historical event becomes the entire current label. Q19–Q22 constrain representation and false claims. They do not authorize a new interpretation engine, guideline adapter or patient-result ingestion.

### Question-to-source proof requirements

| Questions | GTR bundle | WHO bundle | FDA table and openFDA |
| --- | --- | --- | --- |
| Q01–Q13 | May list a candidate test; cannot prove companion identity or the FDA claim by name. Record native partial coverage only. | No proven cross-source device join. Unsupported companion scope, not clinical absence. | Table rows can state the listed scope. Exact PMA pages cover only the approved ledger IDs. Unbudgeted P190004/P190026/P110027/P230011/P140025/P210001 remain table/document-only; no covert follow-up request. Clinical limitation text stays in its cited decision if the captured tier lacks it. |
| Q14–Q16 | Exact BRCA1 field selection, native accession/version, laboratory and type are the positive witness. Retain clinical and research distinctions. | Gene selector unsupported. | A regulatory query is not a laboratory test inventory. No name join. |
| Q17 | Check exact accession if present in bulk; preserve source-date distinction from challenged web page. Absence from this report alone does not resolve its historic record. | Unsupported identity namespace. | Unsupported identity namespace; no fallback. |
| Q18 | A planned synthetic zero control evaluates response semantics. Distinguish it from any actual captured native zero. | The exact WHO product-card route uses the same source-local absence boundary; marker search has its own synthetic no-match control. | The ledger's exact absent application query is a separate native absence probe, not a BRCA1 clinical finding. |
| Q19–Q22 | Preserve any native method/region/specimen fields actually supplied; otherwise explicitly unsupported. | HPV listing does not answer these cancer-assay questions. | Source-specific claims can retain native scoring/specimen context; no local equivalence or clinical categorization. |
| Q23–Q24 | Relevant listing only if exact native relation exists; no broad organism research. | Q23 is answered only for a caller-supplied known WHO Product ID. Exact cards preserve native HPV marker, IDs, codes, prequalification year and regulatory-version fields; the full bundle remains unfiltered. No inferred FDA status. | No WHO-to-FDA name bridge or eligibility-to-authorization inference. |

## Complete legacy surface accounting

Static source inspection is pinned to the existing BioMCP checkout at `4050fdc7dbdf4d713d94cf02a39a147b863665be`. This is repository-source accounting, not proof that this checkout equals a released 0.9 executable. Inspected files are `src/cli/diagnostic/mod.rs`, its dispatch, `src/entities/diagnostic/{mod,search,get}.rs`, `src/cli/system/{mod,dispatch}.rs`, `src/cli/types.rs`, diagnostic guide and source modules. The current code, rather than stale prose, establishes that default JSON already clips gene/condition cells and `--full` exposes complete arrays. No released diagnostic executable was run. The earlier contained 0.9.3 proof did not measure these commands; do not relabel it as a diagnostic baseline. All diagnostic 0.9 cold/warm timing, bytes, tokens, request counts and factual completeness remain explicitly unmeasured until a separate verified containment recipe.

| Legacy command, option or section | Proposed disposition | Reason and required case |
| --- | --- | --- |
| `search diagnostic` | Keep source-specific bounded discovery; require explicit source and supported selector. | Q14 plus explicit-source usage controls; WHO marker/manufacturer discovery uses the stored bundle; no hidden `all` merge. |
| `--source gtr`, `who-ivd`, default `all` | Keep named sources; retire `all`. FDA CDx and PMA are approved additions below. | GTR search and WHO exact-card known-positive cases; invalid source/selector no-I/O controls. |
| No legacy `--source fda-cdx` | Approved separate two-table bundle and exact device/owned drug-application association search. | Q01–Q13 native cells/links, exact headers, scoped rows and typed pivots; no CDx card get. |
| No legacy `--source fda-pma` | Approved exact PMA base/supplement cards, regulatory and supplements sections. Free-text search remains deferred. | Captured native base and supplement identities; empty native base supplement field is preserved. |
| `-g`, `--gene SYMBOL` | Keep exact GTR relation `gene_symbol` selector; retire short alias. | Q14 BRCA1 positive; no object-name substring or WHO/FDA leg. |
| No legacy `--gene-id` | Add proved exact `--gene-id 672`, typed `NcbiGeneId`, exclusive with `--gene`. | Q14 numeric source Gene ID positive, zero/invalid namespace and mutual-exclusion usage case. |
| `-d`, `--disease TEXT` | Retire phrase/synonym search. GTR uses proved exact `--condition-id MedGen:C2676676`; WHO names `--marker` as the literal native-column replacement. | Q14 condition positive, Q23 exact-card recovery; no disease ontology guess. |
| `-t`, `--type TEXT` | Keep GTR exact native relation `test_type`; retire short alias. WHO type selector is deferred. | Q16 Clinical versus Research, conjunction positive/zero; no shared homemade enum. |
| `--manufacturer TEXT` | WHO literal native manufacturer words combine with marker words. GTR laboratory selection remains deferred. | Native WHO search controls; no provider calls or substring overlay. |
| `-l`, `--limit N` | Keep bounded `--limit`; retire short alias. | Minimum, representative, maximum and invalid values; complete walk retains scope/revision. Default size chosen from measured native cases. |
| `--offset N` | Keep mechanical local offset/native supported page only. | Second-page and terminal-page GTR/CDx controls; WHO pages distinct product headers; no hidden first-page truncation. |
| `--full` | Retire discovery-wide expansion. | Complete GTR gene/condition pages remain reachable in named get sections; Q14 broad panel proves no loss. |
| `get diagnostic ACCESSION` | Keep explicit source and exact native ID/version. | Q14 selected accession, Q23 selected WHO Product ID, exact FDA submission and mismatched response. No prefix-based source guessing. |
| positional `genes` | Keep GTR-only paged source relation section. | Q14 complete list, missing field, empty list and WHO no-I/O unsupported controls. |
| positional `conditions` | Keep paged GTR conditions only. WHO marker is on its card and has no section. | Q14 exact MedGen scope; Q23 WHO section rejection before I/O. |
| positional `methods` | Keep only native GTR methods with captured fields. | Q16/Q20 distinct native method/type, missing method optionality; WHO unsupported. No inferred assay accuracy. |
| positional `regulatory` | Replace name-matched 510(k)/PMA overlay with explicit FDA submission retrieval. | Q09/Q10 exact supplement; no match-by-name, trademark stripping, alias expansion or overlap ranking. GTR/WHO requests without native FDA ID receive honest unsupported guidance. |
| positional `all` | Retire; name one source-supported section. | Usage guidance and no network; do not silently omit a requested live regulatory leg. |
| implicit `summary` | Keep default identity card; no extra summary token. | Source-native short card, absent optional fields and exact ID checks. |
| `gtr sync` | Rename `sync gtr` under existing shared contract. | Complete pair validation, atomic publication, failed replacement preserves old revision, subsequent stored read sends zero requests. |
| `who-ivd sync` | Rename `sync who-ivd`. | Full CSV with multiline codes, new/missing columns, failed replacement and no read-triggered refresh. |
| `list diagnostic` | Replace with generated `help diagnostic`. | Help examples parsed from the same request tree; no second source/section list. |
| `get gene SYMBOL diagnostics` | Keep retired gene convenience section. | Existing gene disposition stands; explicit GTR diagnostic selector is the supported route. No hidden source work on gene get. |
| `get disease NAME_OR_ID diagnostics` | Keep retired disease convenience section. | Existing disease ruling stands; no name/synonym fanout or diagnostic recommendation. |
| shared `-j`, `--json` and section-extracted JSON flag | Map to shared `--format json`; retire entity parsing workaround. | CLI/MCP same typed request and envelope; Markdown derives from that envelope. |
| shared `--no-cache` | Use existing explicit transport cache policy. | Fresh diagnostic request obeys shared semantics; stored bundle reads never synchronize. No entity-only cache system. |
| shared `--disable-sources` | Keep existing source/group behavior. | Disabled GTR/WHO/FDA before requests, cache and replay; WHO noncommercial group. |
| shared `--help`, `-h` | Existing generated help behavior. | Source-specific inputs and supported section examples. |
| `BIOMCP_GTR_DIR`, `BIOMCP_WHO_IVD_DIR`, `OPENFDA_API_KEY` | No new environment-only API. Reuse shared storage/key configuration. | No home/environment leakage; missing bundle and optional-key public access have separate outcomes. |
| first-read auto-download and stale auto-refresh | Retire read-triggered sync. | Explicit synchronization; missing/stale stored data remains truthful. |

The legacy diagnostic surface has no positional free-text search, drug selector, section-limit option or separate biomarker/genomic-variant interpretation command. The source file named `variant/search/diagnostics.rs` describes search diagnostic messages, not diagnostic test inventory, and adds no diagnostic leaf. Shared search-all routing is not a new diagnostic command. No table row above claims completed product parity.

## Proposed outside-in case map

| Commands and options | Harvested positives and required controls |
| --- | --- |
| GTR search `--source gtr`, `--gene`, `--gene-id` | Q14 exact BRCA1 `gene_symbol` and Gene ID672 match the same retained native relations. Assert their mutual exclusion; unknown field value gives source-local zero. No text rewrite or WHO leg. |
| GTR `--condition-id`, `--type` | Singleton `GTR000006692.3` condition MedGen:C2676676 and native Clinical type are actual positives. Exact Research category and conjunction positive/zero use retained source rows; wrong namespace/source-inapplicable options fail before I/O. |
| GTR identity conflict | Exact GTR000551465.2 retains both incompatible source rows. One conflict search row, identity-conflict problem, no get step; direct get no card; sync receipt counts one duplicate identity. Current/public filters cannot select one row silently. |
| WHO exact card | Q23 caller-supplied known product IDs with one/two/three/nine native code rows, all rows retained; exact unknown ID, missing bundle and unsupported sections/search selectors. Full bundle 435 rows/125 products remains unfiltered. |
| FDA CDx `--device-id`, `--drug-application`, `--biomarker` | Q01–Q13 exact device and owned drug-link IDs; P200010/S026 combination keeps NDA210496/BLA125084; Q09/Q10 same assay with distinct scoring context. P110027/S012 proves exact `--biomarker KRAS` with details G12C and owned NDA214665. Rows lacking typed drug ID remain reachable by device search. Missing/ambiguous ID and text/link mismatch never create a pivot. Group-table biomarker selector unsupported rather than false empty. |
| CDx table sync | Both exact ordered header arrays and all physical rows retained. Missing/extra/reordered/changed column in either table fails entire publication and keeps prior bundle. Whole cells/links and source row provenance survive; no narrative parser or hidden domain filter. |
| Search/association `--limit`, `--offset` | Default five rows, terminal page, every selector and immutable revision retained; native row order and count unit explicit; warm stored read makes zero requests. Association `next` includes all distinct valid typed pivots, then page and at most three extras. |
| GTR get genes/conditions/methods and section paging | Q14 broad panel complete walk, missing versus empty optional native fields, real differing symbol/Gene-ID counts and row/distinct MedGen counts; no display clipping loses source rows. |
| PMA get, optional `--supplement`, regulatory and supplements sections | Captured P150013 base has empty native supplement number; exact S020/S021 positive rows remain distinct. Omitted supplement selects base only; unknown supplement returns absence with no fallback. Supplements section pages only base application members; invalid section-plus-supplement errors before I/O. Exact record identity checked before admission. |
| PMA/CDx/gene/drug next | Gene 672, exact PMA/supplement and whole-cell-owned NDA/BLA pivots use existing checked request spellings. K identities have exact clearance gets; DEN/H and ambiguous/untyped cells have source links but no get. Parsed CLI/MCP steps execute against recordings; no name bridge. |
| All syncs | GTR pair, full WHO CSV and approved FDA tables publish atomically; wrong/malformed member, failed second file, concurrent reads and failed replacement preserve the prior complete revision. No read-triggered sync. |
| Shared policy/output | CLI/MCP equality, generated schemas/help, bounded Markdown, noncommercial disable before cache/replay, registry source/terms identity and failed/skipped outcomes. |
| Empty/failure | Q17 challenged website stays separate from captured bulk current record; Q18 zero semantics. Actual captured absent application remains a separate native zero. Synthetic 429/500/timeout/broken body/HTTP200 error controls never become clinical absence or claimed live outages. Raw bodies/output survive comparison failure. |
| Deferred legacy options | WHO type, GTR laboratory, PMA text query, `all`, `--full` and retired convenience routes give concrete supported guidance before I/O. No 510(k) positive is fabricated. |

Failure bodies are synthetic transport controls unless observed in the existing ledger. Raw native capture precedes shared type acceptance. Adapter acceptance checks the claimed fields, missingness, multiplicity and scope against these unchanged native bodies. Use the existing `cases` target and explicitly feature-owned contract inventory; no unlisted coverage targets or blanket library counting. This design change requires document consistency checks, not a product coverage run.

## Sources kept and deferred

Keep GTR bundles, complete unfiltered WHO IVD cards, exact native FDA PMA retrieval and the PM-approved separate `fda-cdx` two-table bundle. Drop the normalized-name FDA overlay, disease synonym expansion, generic source merge, automatic refresh and discovery-wide full expansion. WHO marker and manufacturer search is implemented below. WHO type, GTR laboratory search and PMA free-text query remain deferred. The original investigation deferred 510(k) pending its later approved exact K-number capture. The exact clearance implementation below uses that retained positive without another provider request. DEN identifiers remain displayed source facts without device get steps. No GTR live API, full XML publication, clinical interpretation engine, new licensed directory or guideline ingestion is proposed.

## Ian's questions

None.

## Native capture findings

The separate build delegate ran `j12632` from landed `d18bfbba511e3852c99228f1a5166a14633e5cee` on 2026-10-25 under the coordinator-approved exact recipe above. The request budget closed at 19 physical GETs. The two expected absence probes returned actual HTTP 404 with `NOT_FOUND` / `No matches found`; these are source-local zeros, not failures of the whole investigation. No extra page, provider retry, redirect or fallback followed. Product code and legacy executables were not run by this capture.

| Captured source | Actual size/count evidence | Meaning and limits |
| --- | --- | --- |
| GTR version report | 13,075,988 compressed bytes; 154,522,200 expanded bytes; 224,353 rows, 224,352 distinct accession/version values; 63,919 rows with `now_current=1`, 160,434 historical rows. | The source contains one conflicting exact accession/version, `GTR000551465.2`: both original rows differ in `now_current`, `genes`, `condition_identifiers` and `last_touch_date`. Both rows and their distinct raw hashes are retained; no copy was chosen. Current-version and active/public test/laboratory status are different native flags. |
| GTR relation report | 47,325,875 bytes; 514,016 rows; 63,950 distinct accession/version values; 305,621 gene rows and 208,395 condition rows. | Relation-report identities do not have the same scope/count as current-version rows. Preserve unjoinable rows explicitly; do not inner-join away gaps or claim every relation has a current card. |
| GTR BRCA1 selection | 433 exact BRCA1 gene relation rows and 433 current-version rows; each represents 433 distinct accession/version values and 433 accession bases; 32 single-gene selections. Native gene relation explicitly carries Gene ID `672`. | Numeric source ID enables an exact gene link without name matching. It does not establish variant coverage or clinical eligibility. |
| Broad BRCA1 panel | `GTR000511179.13`; 4,673 distinct pipe-delimited native gene-symbol tokens, 4,668 distinct native gene IDs/relations, and 5,134 condition relation rows representing 5,127 distinct MedGen IDs. Final source-only whole-row projection is 2,345,390 compact UTF-8 bytes. The earlier 2,345,445-byte figure included investigation wrapper context and is superseded for this declared projection. | These measures count different source structures. Never call them unique gene counts or truncate them silently. A complete default detail response is too large; page native relation rows. |
| Single-gene BRCA1 test | `GTR000006692.3`; one gene relation and one condition relation; final source-only whole-row projection 1,583 bytes. The earlier 1,637-byte wrapper-inclusive figure is not the declared final projection. | Small identity does not justify designing a global one-gene model. |
| Previously challenged GTR web record | Bulk report has 11 versions of `GTR000021517`; `GTR000021517.11` is current-version, test name BRCA1, Molecular Genetics Laboratory / London Health Sciences Centre, `last_touch_date=2026-02-06`. | The bulk observation answers source listing facts independently of the unavailable web page. The web challenge is not absence from GTR. Last-touch date is not fresh laboratory verification. |
| WHO CSV | 435 rows representing 125 distinct WHO Product IDs. Selected HPV products `0455-180-00`, `0085-028-00`, `0466-046-00`, `0633-078-00` have 2, 1, 9, 3 native rows respectively; actual CSV code cells are not multiline. | Native WHO Product ID groups repeated code rows. Preserve the full rows and distinct codes; neither row count nor code count is a product count. Final declared source-only selected projections are 649, 266, 2,818, 931 bytes respectively. Earlier wrapper-inclusive figures are superseded for this projection. |
| FDA companion HTML | Two tables contain 233 and 10 rows including headers; all requested clinical table identifiers were found; no physical cell has `rowspan > 1`. | Preserve both table headers and physical links/cell scope. General parser robustness must not be described as an observed rowspan condition. Table facts are distinct from the PMA API fields. |
| Exact PMA pages | Native totals in ledger order are 62, 12, 28, 31, 10, 7, 13, 45, 36, 19, 15, 5. Each complete native total fits the single approved 100-row page. | These totals are submission/supplement rows for one base application, not tests or unique marketed devices. No second page was required. |
| Exact absence probes | `K080653` 510(k) and `P000000` PMA both return 404 `NOT_FOUND`. | Preserve native absence. The 510(k) probe is no positive record and cannot justify accepting its shared domain type or claim the named ID is invalid everywhere. |

The retained GTR README defines `now_current=1` as current version and `0` as previous version; `test_currStat=1` as current and other values as not current; `test_pubStat=3` as public; laboratory current/public values are separately defined. It identifies `GTR_identifier` as MedGen CUI and `gene_or_SNOMED_CT_ID` as NCBI Gene ID for gene objects and SNOMED CT for condition objects. The object discriminator, namespace and exact value must travel together. A generic “object ID” field that forgets that discriminator would create false joins. The complete raw column inventory and original source bodies remain in the capture artifact, not replicated as product schema claims.

These are full source-only projection sizes, without the BioMCP envelope, terms or CLI/MCP output. Their estimated token cost is mechanically `ceil(bytes / 4)` if needed; no estimated value establishes a product ceiling. Native source names and fields remain unchanged in these projections. No comparable product budget, released 0.9 parity or warm network-cache improvement follows from them.

### Conflicting native identity and cardinality policy proposal

The PM accepted explicit conflict handling for the source's duplicated `GTR000551465.2`. Check exact accession/version multiplicity across the complete version report before applying current/public filters. A last-row-wins map or filtering away one `now_current` value would hide the conflict. Keep both source rows in the bundle conflict inventory. Search shows exactly one conflict row with an identity-conflict problem and no get step. Direct get supplies no card. Sync counts duplicate identities in its receipt. Never emit an inferred gene/condition union. Unrelated valid records remain usable under one complete bundle revision. This accepted behavior is a type/adapter prerequisite; it does not silently repair the source.

The broad panel also proves that native symbol strings and numeric Gene-ID relations cannot be interchangeable views of one canonical set. Preserve each native collection and its origin; never add five fabricated gene IDs or delete symbols to make counts agree. Likewise, retain all 5,134 condition rows with their source scope even though 5,127 distinct MedGen IDs occur. A page total states whether it counts native rows or distinct IDs. No domain deduplication discards an alternate relation without a documented exact identity rule.

## Retained proof and measured source projections

The durable native artifact is `data/biomcp-redux/0154-diagnostic-native-capture/j12632-native-capture.tar.gz` on Yellow, 85,767,072 bytes, SHA-256 `d4a796d206d4199156a2ab0a4ba584a77da40c116012ab4d9f451a401213f46b`. Its root is `2290-diagnostic-native-capture`. The delegate independently verified all 158 manifest files and no unlisted files. It contains all 19 original responses, original URLs/query ledger, headers, retrieval and elapsed metadata, raw decoder results, original capture recipe and scripts, plus logs for `j12632`, `j12640` and `j12647`. Later jobs analyzed already retained bytes; they did not add provider requests. The archive also preserves both conflicting native GTR rows and their hashes, exact requested current/previous GTR versions, full column optionality inventories, both physical FDA tables and links, source-only row selections and declared projection/measurement definitions. No decoder errors occurred.

Key artifact entries are `capture/inventory.json`, `offline/gtr-findings.json`, `offline/gtr-duplicates.json`, `offline/gtr-selection-scope.json`, `offline/gtr-legacy-current-native.json`, `offline/fda-physical-tables.json`, `offline/fda-resolved-table-rows.json` and `offline/source-only-query-measurements.json`. These are raw evidence pointers, not a second status record. Source-provided update and HTTP metadata retain their own roles; local capture time never becomes publisher release time.

All GTR/WHO columns are present structurally in every row, but many GTR cells are empty; no WHO cells were empty. The relation report has 75,109 empty MIM cells and 139,110 empty Gene-or-SNOMED cells. Therefore a structurally required TSV column does not make its value required in every domain row. PMA base rows carry empty `supplement_number`, `supplement_reason` and `supplement_type`, one per captured base page; many supplement records omit `fed_reg_notice_date`. Preserve native empty versus absent and never manufacture “S000” or a publication date.

The following projections serialize captured source rows or explicitly selected source fields without a product envelope. Full and comparable byte counts are identical because no terms/policy fields were removed. Estimated tokens use the declared conservative byte division, not an executed model tokenizer. None is an implemented command default, product ceiling, CLI/MCP proof or measured 0.9 comparison.

| Source-only projection | Compact UTF-8 bytes | Estimated tokens `ceil(bytes/4)` | Case scope |
| --- | ---: | ---: | --- |
| Broad GTR whole native rows | 2,345,390 | 586,348 | Q14 complete source test/version and all native relations; pagination required in product. |
| Broad GTR declared header | 316 | 79 | Q14 selected header fields only; not a generated card. |
| Broad GTR first five gene rows | 1,168 | 292 | Q14 bounded source relation projection. |
| Broad GTR first five condition rows | 1,153 | 289 | Q14 bounded source relation projection. |
| Single-gene GTR whole native rows | 1,583 | 396 | Q14 exact singleton; not representative of broad panels. |
| Single-gene GTR declared header | 340 | 85 | Q14 identity header projection. |
| Single-gene GTR gene / condition rows | 231 / 251 | 58 / 63 | Q14 native row scopes remain separate. |
| WHO HPV `0455-180-00` / `0085-028-00` | 649 / 266 | 163 / 67 | Q23 complete native product groups with two / one code rows. |
| WHO HPV `0466-046-00` / `0633-078-00` | 2,818 / 931 | 705 / 233 | Q23 complete nine / three code-row groups. |
| FDA table `P190004` | 519 | 130 | Q01/Q02 source table row, not all decision-label limitations. |
| FDA table `P200010/S010` | 440 | 110 | Q03/Q04 source relation scope. |
| FDA table `P190026` / `P110027/S012` | 455 / 429 | 114 / 108 | Q05/Q06 exact source identifiers. |
| FDA table `P230011` four rows | 1,751 | 438 | Q07/Q12 row collection; not four device identities. |
| FDA table `P140025` four rows | 1,800 | 450 | Q08 original source contexts kept separate. |
| FDA table `P150013/S020` / `S021` | 493 / 489 | 124 / 123 | Q09/Q10 different tumor/scoring context. |
| FDA table `P210001/S001` | 462 | 116 | Q11 source-listed relationship. |
| FDA table all `P200010` ten rows | 4,656 | 1,164 | Q13 base application relation collection, not one universal claim. |

The fresh parser process read the retained files in 1.725193291 seconds. Broad-panel native selection plus serialization measured 0.049968/0.049422 seconds in two runs; the declared header measured 0.028357/0.027301, first five gene rows 0.010398/0.010189, and first five condition rows 0.010523/0.011895. Both runs used the same retained files. The filesystem cache was not evicted. These values are local investigation measurements with job load recorded in the artifact; they establish neither cold physical-disk performance nor warm HTTP caching. Native request timings, counts and byte sizes remain in the physical ledger. No timing was inferred from historical experiments or invented for a product command.

The declared 11-field GTR header projection uses only native `test_accession_ver`, `lab_test_name`, `name_of_laboratory`, `name_of_institution`, `facility_country`, `last_touch_date`, `now_current`, `test_currStat`, `test_pubStat`, `lab_currStat` and `lab_pubStat`. It is not an approved product header: an implementation may need fewer status fields or a distinct submitted-status section, while preserving their source meaning and complete accessibility.

Physical request elapsed time was 0.250548167 seconds minimum, 0.454992916 median and 4.446366417 maximum. Its sum was 14.993589998 seconds, excluding explicit pacing and offline analysis. Total original response bodies occupied 61,225,746 bytes. WHO's complete CSV was 80,865 bytes and the complete FDA companion HTML was 178,560 bytes. These are actual captured source sizes, not estimates of an installed SQLite bundle or a compressed redistribution package. The GTR README was 7,424 bytes. Each absence response body was 80 bytes. Original per-request measurements remain in the ledger. FDA companion-table update cadence is not documented as fixed; retain native retrieval/HTTP metadata without inventing a publisher refresh interval. No current XML or whole FDA JSON bundle size was measured because neither was downloaded.

## Standalone diagnostic type boundary

The neutral diagnostic records retain source ownership through their tagged record kind. `DiagnosticRecord` has separate GTR version, GTR membership/method, WHO product, PMA submission and companion-row variants. None implements conversion to `Statement`. Consumer requests remain standalone until diagnostic command registration; existing request, step, sync and envelope schema versions do not change here.

The field review uses the retained capture `j12632-native-capture.tar.gz` (SHA-256 `d4a796d206d4199156a2ab0a4ba584a77da40c116012ab4d9f451a401213f46b`) and its `capture/inventory.json` and `offline/` inventories. Raw source bodies remain outside Git. Every GTR test-version column is present in that capture, but many cells are empty, including test and laboratory names. `GtrTestVersion` therefore preserves every whole cell, including empty strings and independent pipe lists. It neither pairs parallel lists nor interprets native flags. Each complete accession/version row remains representable independently, including both conflicting rows under `GTR000551465.2`. Candidate conflict rows carry no selected name; the card union has no conflict variant.

GTR gene and condition row types remain distinct. A gene reference carries source `gene_symbol` as its label and the numeric GeneID only when supplied. The independent `GTR_identifier` remains a MedGen reference. Condition rows carry a shared `Concept` and retain their separate native MIM and SNOMED cells, including empty values. Independent method entries carry their list kind and source list index. Genes, conditions and methods response schemas each accept only their own row type. The typed discovery request accepts one relation selector, optionally combined with the exact native test type. It never resolves a label into an identifier.

WHO's eight captured columns are nonempty on every row. `WhoIvdProduct` groups by the checked product identity while retaining every complete code row in order; product name, code, format, manufacturer, marker, regulatory version and year remain row-scoped. No shared manufacturer, marker or FDA identity is inferred from the group. Product cards contain every native row and have no section expansion. Separate stored WHO discovery uses literal marker and manufacturer cells.

The PMA capture inventories distinguish missing notice-date fields from explicit empty base supplement cells and empty supplement type/reason cells. `DeviceSubmission` preserves these distinctions. The checked native supplement cell accepts the explicit empty base value and exact supplement values; the request supplement selector accepts only an explicit supplement, with no invented base alias. Consumer cards and supplement headers exclude the whole approval statement. Regulatory fragments retain their selected submission identity and its own dated metadata, while the statement remains only in bounded fragment text. An omitted section limit resolves to one fragment for regulatory and five native rows otherwise; selecting a section keeps an explicitly supplied limit and resolves an omitted one for that section.

Companion individual and group rows have separate tagged variants and separate exact-header cell structures. Every original whole cell retains its own source links. Each row retains its immutable snapshot and physical data-row index; its table kind determines the source table. `device_ids` admits captured PMA, K, DEN and H source facts without promising executable gets. The search selector admits PMA base, PMA/supplement and exact K identities. Typed drug applications retain a published NDA/BLA prefix and exactly six digits. A value type checks grammar; later source mapping must establish cell/link ownership, refuse conflicting text/link identities and never populate an identifier from another cell or a name.

## GTR command API details

GTR search candidates retain the first matching native relation in relation-report order under `matched`. The search count unit is `test_identities`. A relation whose exact test has no current card remains a `missing_card` candidate with its original relation; it never gains a fabricated name. Multiplicity is checked across all version rows before deciding current-card availability. A repeated accession/version returns `identity.conflict` and supplies no selected card or executable get.

Genes and conditions use the count unit `native_relation_rows`. Their `distinct_id_total` counts complete source identifiers across that relation collection, including namespace, while retaining rows without identifiers. Methods use `native_method_entries` and retain independent source list indices. The original version-row pipe cells do not supply any of those relation counts. Sync bytes count the downloaded source files before native gzip decoding.


## Exact PMA implementation measurements

The bounded six-request capture j13187 retrieved the complete P150013 application with 36 rows and P200010 with 31 rows. Their explicit empty base supplement cells occur at native indices 24 and 10. Exact P150013/S020 and S021 each returned their own single unchanged application member. S020 has its native notice date; S021 omits that field. P000000 returned native HTTP 404 `NOT_FOUND` using the approved `limit=1` control. The product application request uses `limit=100`, so this receipt is not a native product replay for P000000. Synthetic exact 404 replay proves product absence handling. Unknown-supplement product absence is also synthetic-only. No capture receipt was relabeled to another query.

The separate retained product job j13229 ran 29 CLI/MCP pairs against four local receipt deliveries with no provider calls. Native base and supplement selection, complete 35/30-header walks, each exact row’s statement reconstruction, emitted cross-source selectors and embedded schemas passed. A native CDx bundle was not mounted in this product job, so its PMA-to-CDx pivots were checked as exact selectors without claiming a native CDx execution. Bodies and raw outputs remain outside Git. K173492 returned one native 510(k) row in the bounded capture; this is a positive source fact for a separate source ticket. The later exact 510(k) implementation below supplies K gets; DEN identifiers remain source-only links.

All sizes below include the product envelope. Comparable compact bytes remove the registry terms contribution only; stdout adds its final newline. Token ceilings use the existing conservative byte division. Each new ceiling is measured stdout plus 15 percent, rounded up.

| Native command | Largest compact full/comparable bytes | Stdout byte ceiling | Token ceiling |
|---|---:|---:|---:|
| Base card, P150013/P200010 | 999 / 897 | 1150 | 288 |
| Exact S020/S021 card | 970 / 868 | 1117 | 280 |
| Five native supplement headers, largest page | 2518 / 2416 | 2897 | 725 |
| One native regulatory fragment, largest retained statement | 3258 / 3156 | 3748 | 937 |

The synthetic outside-in case uses larger statement fragments to exercise splitting. Its eight measured stdout values are 986, 961, 961, 2428, 1241, 7238, 300 and 280 bytes. The corresponding ceilings live in `tests/cases/diagnostic/pma.yaml`. Recorded failure cases cover 429, 500, deadline and broken JSON with source terms; request-bound identity and complete-collection checks run before cache admission. An incomplete application returns `collection.incomplete`, no selected card and no cached cut collection.

Retained product evidence is `data/biomcp-redux/0175-fda-pma-product-proof/j13229-native-product-proof.tar.gz`, SHA-256 `6ee8b423121aee341f6ca4d73367b36d7287d5dcca0fec12aa49e3443d5f5c2e`. Independent verification checked all 380 manifest entries and exact 381-file archive membership before unique scratch cleanup.

The recorded 429, 500, deadline and broken-JSON failures measured 308, 307, 313 and 305 stdout bytes in j13238. Their +15 percent ceilings are enforced in `tests/cases/diagnostic/pma-failures.yaml`. Whole command help passed its existing ceiling before the routine gate; no existing help ceiling changed.


## Exact 510(k) implementation

The K173492 receipt in j13187 supplies the approved positive. Its exact request is `GET https://api.fda.gov/device/510k.json?search=k_number%3A%22K173492%22&limit=1&skip=0`; HTTP 200 returns one native row and native `meta.results` skip 0, limit 1, total 1. The body is 1,979 bytes with SHA-256 `7a6e8e0093d82bdff9980354052d818fe3d33145469edb0e0f33201e2da1183b`. Its retrieval date and native `meta.last_updated` remain separate provenance facts. The adapter never changes the captured literal decision description `Unknown` into an inferred regulatory meaning.

The clearance card keeps `k_number`, `device_name`, `applicant`, `decision_code`, `decision_description`, `decision_date`, `date_received` and `product_code`. Missing optional fields remain missing and explicit empty strings remain empty. The native `openfda` object, harmonized fields and GMDN content have no consumer projection. Records never merge by device name or product code. A foreign returned K number produces a `k_number` identity problem and no card; the existing checked cache-admission predicate prevents that row from entering or returning through the cache.

The unchanged K080653 probe in j12632 returned HTTP 404 `NOT_FOUND` with `No matches found!`. This is source-local absence, with no claim that the identifier is invalid elsewhere. Malformed bodies, rate limits, upstream failures and deadlines remain failed source outcomes. Non-K get identities, section arguments, supplements and free-text 510(k) searches fail before transport. DEN and H identifiers have no executable get.

Companion associations emit one pivot per distinct exact cell-owned K identity, preserving other PMA and drug pivots and any page continuation. The clearance card emits the exact companion device selector and its ordinary fresh action. The source registry owns `fda-510k`, its openFDA terms, open use category and disabled-source behavior; it has no group alias.


The retained product run j13314 at `62fe0c837a0ef4a55dc45dbcb519aff9fcce8fc8` passed eight CLI/MCP pairs with embedded schema validation. It used three contained local deliveries and zero provider requests. The unchanged native companion HTML executed the clearance card's exact device selector and returned the exact K-number get pivot. This proves those source-owned selectors without a name or product-code join. The routine whole-help preflight passed its existing ceiling; no help ceiling changed.

| Output | Complete compact JSON bytes | CLI stdout bytes | Comparable compact bytes | Complete stdout ceiling | Estimated token ceiling |
|---|---:|---:|---:|---:|---:|
| Native K173492 card | 758 | 759 | 643 | 873 | 219 |
| Native K080653 absence | 281 | 282 | 166 | 325 | 82 |
| Synthetic 429 | 309 | 310 | 194 | 357 | 90 |
| Synthetic 500 | 308 | 309 | 193 | 356 | 89 |
| Synthetic deadline | 314 | 315 | 199 | 363 | 91 |
| Synthetic malformed JSON | 306 | 307 | 191 | 354 | 89 |

Ceilings round complete stdout times 1.15 upward. The existing case runner independently validates publisher terms and subtracts only `use`, `terms` and `terms_url` for its historical comparable check. The full native envelopes remain separately measured above. No released 0.9 equivalent was measured, and these contained replay measurements say nothing about provider speed.

## Shipped command and option ledger

This closes the static legacy surface accounting above. The historical denominator includes leaf commands and long command options, excluding aliases, positional IDs, section words and global flags. A ported source selector does not retain the old `all` value. Deferred selectors remain recorded as undecided implementation; they are not silently counted as dropped.

<!-- parity-accounting -->
| 0.9 leaf command | Command outcome | Ported or renamed options | Dropped options | Deferred options | Undecided options | Ruling |
| --- | --- | --- | --- | --- | --- | --- |
| `search diagnostic` | Ported or renamed | `--source`, `--gene`, `--type`, `--manufacturer`, `--limit`, `--offset` | `--disease`, `--full` | — | — | Choose gtr, who-ivd, fda-cdx or fda-device-recall explicitly. FDA device recall search uses recalling-firm phrase, exact product code and initiation dates for every medical device. GTR uses exact native fields; condition-id replaces phrase expansion. WHO uses literal `--marker` and `--manufacturer` words without synonyms. GTR laboratory selection remains deferred. |
| `get diagnostic` | Ported or renamed | — | — | — | — | Choose gtr, who-ivd, fda-pma, fda-510k or fda-device-recall explicitly and an exact native identity. Device recalls use Z-NNNN-YYYY and optionally add an exact enforcement classification. Named sections remain source-scoped; all and the name-derived regulatory overlay retire. |
| `gtr sync` | Ported or renamed | — | — | — | — | Use sync gtr; validate and atomically replace the complete two-file bundle. |
| `who-ivd sync` | Ported or renamed | — | — | — | — | Use sync who-ivd; replace the full unfiltered export. |
<!-- /parity-accounting -->

| Legacy or added surface | Shipped spelling or disposition | Case owner |
| --- | --- | --- |
| Search/default source and `all` | Explicit `--source gtr`, `--source who-ivd` or `--source fda-cdx`; automatic merge retires. | `diagnostic/gtr.yaml`, `diagnostic/fda-cdx.yaml`, `diagnostic_retirements.rs`. |
| `-g`, `--gene` | `--gene SYMBOL` selects exact GTR gene_symbol; short alias retires. | `diagnostic/gtr.yaml`, fixed-guidance and literal-value controls. |
| Added `--gene-id` | Exact positive NCBI Gene ID, exclusive with gene and condition. | GTR full native proof and diagnostic request contracts. |
| `-d`, `--disease` | Retire phrase/synonym expansion; use exact GTR `--condition-id MedGen:CUI`. | GTR native condition pages and fixed guidance. |
| `-t`, `--type` | Keep exact GTR Clinical/Research value; short alias retires. WHO type remains deferred. | `diagnostic/gtr.yaml`, diagnostic request contracts, retirements. |
| `--manufacturer`, GTR laboratory selector | WHO manufacturer words are supported; GTR laboratory selection remains deferred. | WHO search cases plus fixed GTR no-I/O guidance. |
| `-l`, `--limit`, `--offset` | Keep bounded long options and revision-preserving search continuation; short alias retires. | GTR/CDx YAML, CDx pagination and full native walks. |
| `--full`, positional `all` | Retire unbounded expansion; name one section and follow its next page. | Retirement cases and complete GTR/PMA walks. |
| Exact get and implicit summary | Explicit source plus accession/version, WHO Product ID, PMA number or K number; default is the card. No source guessing. | GTR/WHO/PMA YAML and `diagnostic_510k.rs`. |
| `genes`, `conditions`, `methods` | GTR-only native pages with `--section-limit`, `--section-offset`. | GTR YAML, native full walks and request contracts. |
| `regulatory` | PMA-only selected base/supplement statement fragments; no name-derived FDA overlay. | PMA YAML, `diagnostic_pma/walks.rs`, native j13229. |
| Added `supplements`, `--supplement` | Base-only supplement headers; exact supplement card/statement. Never infer latest supplement. | PMA YAML, metadata/walks and native j13229. |
| WHO get | Entire known Product ID group; no sections or inferred FDA identity. Separate WHO search supports marker/manufacturer, while type remains deferred. | WHO YAML and native j13094. |
| Added FDA companion search | Exact `--device-id`, owned `--drug-application`, whole table-0 `--biomarker`; complete individual/group rows stay separate. No CDx card get. | CDx YAML/Rust and native j13117. |
| Added exact K get | `get diagnostic K… --source fda-510k`; no sections or free-text search. DEN/H remain source facts without executable gets. | `diagnostic_510k.rs`, failure YAML and native j13314. |
| Added `--bundle-revision` | Continuations retain immutable stored source data. Missing retained revision offers explicit restart/sync. | GTR/CDx pagination and bundle mechanics. |
| `gtr sync`, `who-ivd sync`; added FDA sync | `sync gtr`, `sync who-ivd`, `sync fda-cdx`. Failed replacement preserves the usable old bundle. | Bundle mechanics and two executable sync examples per source. |
| `list diagnostic`, `--help`, `-h` | Generated diagnostic help; source choices and sections derive from request schemas. | `diagnostic/help.yaml`, executable-example check and MCP discovery. |
| Gene/disease `diagnostics` | Remain retired convenience sections. Use an explicit diagnostic request; no fanout. | Existing gene/disease retirement owners. |
| `-j`, `--json`, extracted-section JSON workaround | Shared `--format json`; Markdown derives from the same envelope. | Fixed guidance, source Markdown cases and CLI/MCP parity. |
| `--no-cache` | Shared `--fresh` for transport cache bypass. Stored bundle reads still never synchronize. | Retirements and stale-bundle cases. |
| `--disable-sources` | Shared source/group union applies before request, cache or replay. | Registry admission, GTR/WHO policy and exact FDA controls. |
| `BIOMCP_GTR_DIR`, `BIOMCP_WHO_IVD_DIR`, `OPENFDA_API_KEY` | Shared storage/key configuration replaces entity-specific directories; optional key does not gate public FDA access. | Isolated launch and registry controls. |
| First-read download, stale auto-refresh | Dropped; explicit sync only, stale data remains marked stale. | Missing/stale and atomic-replacement cases. |

Deferred: free-text PMA and 510(k) search, WHO type and GTR laboratory selection lack approved native selector proof. Dropped: normalized-name FDA overlay, disease synonym expansion, generic source merge, read-triggered refresh and discovery-wide full expansion. Exact FDA identifiers do not restore those dropped behaviors. A listing is not authorization. A companion row is not a treatment recommendation.

## Complete question and capture ownership

Question IDs name reviewed questions, not twenty-four successful clinical answers. The implementations retain source facts without a clinical interpretation engine. The following map accounts for every design question and every original physical ledger row.

| Design IDs | Product or evidence owner | Limit |
| --- | --- | --- |
| Q01, Q02 | F01 native companion table; j13117 complete relation output. | P190004 table context only; the cited decision's full qualification is not supplied by a new PMA call. |
| Q03, Q04 | P200010/S010 native table relation; j13117. | Source row context does not establish a complete therapy indication. |
| Q05, Q06 | P190026 and P110027/S012 native table relations; j13117. | Exact biomarker/cell scope stays unchanged; no alteration inference. |
| Q07, Q08 | P230011 and P140025 native table collections; j13117. | Whole native rows, not new exact-submission captures. |
| Q09, Q10 | P150013/S020 and S021 table rows plus native exact PMA j13229. | Scoring text remains in its own row; PMA statement belongs to that submission. |
| Q11, Q12 | P210001/S001 and P230011 table rows; j13117. | No gene-level eligibility or universal panel companion claim. |
| Q13 | P200010 table and native PMA application j13229. | Neither source becomes a clinical negative-result interpretation engine. |
| Q14 | Full native GTR j13177: BRCA1/Gene 672 selection and complete broad-panel gene/condition walks. | Membership, native row totals and distinct IDs remain different quantities. |
| Q15, Q16 | GTR type/status/source fields; native j13177 and synthetic GTR cases. | Listing, current version and Clinical/Research do not establish authorization or present availability. |
| Q17 | Original G02/G03 native bulk has current GTR000021517.11; no new website request. | The challenged web page remains unverified; bulk facts do not repair that access observation. |
| Q18 | Synthetic GTR unknown selector zero and WHO absent product; native K080653 absence. | No empty BRCA1 result or general absence is claimed. P000000 product zero and unknown PMA supplement are synthetic. |
| Q19, Q20, Q21, Q22 | Reviewed representation constraints above; native method/cell fields retained where supplied. | No guideline ingestion, universal method equivalence, extended-region inference or inheritance interpretation is implemented. |
| Q23, Q24 | W01 full native WHO export and j13094 exact one/two/three/nine-row cards. | Exact card checks use caller-supplied known Product IDs. The later WHO discovery addition searches native marker/manufacturer cells without a WHO-to-FDA join. |
| G01 | Original j12632 README in the investigation archive. | Source field semantics only; not a product command. |
| G02, G03 | Original complete reports, independently retained again by j12938/j12940; slice j12984 and full product j13177. | Full sync and complete walks are postlanding data proof, separate from routine synthetic cases. |
| W01 | Original j12632 export, repeated unchanged capture j13073 and native product j13094. | All 435 rows/125 products preserved; no source-row filtering. |
| F01 | Original HTML, unchanged j13075 capture and j13117 product outputs. | Twenty native and nine synthetic pairs passed; later sync-help failure was repaired and checked separately. |
| F02, F03, F04, F05, F06, F07, F08, F09, F10, F11, F12, F13 | Original twelve complete application responses in j12632. | Investigation inputs only unless explicitly reused by a named product proof. Native j13229 uses its separately approved unchanged P150013/P200010 captures. |
| F14 | Original exact K080653 native 404; native product j13314. | Source-local absence, not proof the ID is invalid elsewhere. |
| F15 | Original P000000 native 404 at limit 1. | Product uses limit 100; product absence is synthetic, not a relabeled native query receipt. |

The later K173492 positive comes from the separately approved j13187 capture and j13314 product replay. Both companion directions execute in j13314. Native PMA j13229 checks emitted companion selectors without mounting the companion bundle; it does not independently prove their native execution. Bodies and raw CLI/MCP evidence remain outside Git.

## Executable case inventory

Every diagnostic YAML run carries its owning `max_bytes` and estimated-token ceiling. Native outside-repo cases retain measured full/comparable ceilings with their archived raw CLI and MCP outputs. These are separate inventories, not an assertion that an archived case runs in the routine suite.

| Case file or target | Behavior owned |
| --- | --- |
| `tests/cases/diagnostic/gtr.yaml` | Synthetic sync, exact selectors, card, each section, zero and fresh replacement. |
| `tests/cases/diagnostic/who.yaml`, `who-help.yaml` | Synthetic whole known-product card, multiplicity, absence, sync and generated help. |
| `tests/cases/diagnostic/fda-cdx.yaml`, `help.yaml` | Synthetic whole cells, source-owned selectors, pagination, two sync examples and full generated diagnostic help. |
| `tests/cases/diagnostic/pma.yaml`, `pma-failures.yaml` | Synthetic exact base/supplement, absent selection, bounded statement, supplements and transport failures. |
| `tests/cases/diagnostic/510k-failures.yaml` | Synthetic 429/500/deadline/malformed reply controls. |
| `runner/diagnostic_gtr.rs`, `diagnostic_gtr/mechanics.rs`, `diagnostic_gtr/native_value.rs` | Missing bundle, atomic replacement, policy, conflicting identities, Markdown and opaque native Gene-ID values without false pivots. |
| `runner/diagnostic_who.rs`, `diagnostic_who/mechanics.rs` | Missing/unsupported/policy, complete replacement, concurrent readers and Markdown. |
| `runner/diagnostic_cdx.rs`, `diagnostic_cdx/pivots.rs` | All-header replacement matrix, concurrency, owned links, conflicting aliases, entity decoding, page/action preservation and Markdown. |
| `runner/diagnostic_pma.rs`, `diagnostic_pma/walks.rs`, `diagnostic_pma/metadata.rs` | Request-bound identity/cache admission, complete collection, complete statement/header walks and absent/empty metadata. |
| `runner/diagnostic_510k.rs` | Exact card, identity rejection/cache, source-local absence, disabled/invalid/error-body and companion pivots. |
| `runner/diagnostic_registry.rs`, `diagnostic_registry/admission.rs` | Shared registry declaration, configuration union, policy and cache admission. |
| `runner/diagnostic_retirements.rs`, `diagnostic_bundles.rs` | Fixed no-I/O migrations, literal boundaries and all three stale stored bundles. |
| `biomcp-types/diagnostic_contracts` and generated schema contracts | No-I/O native IDs, tagged records, request combinations and schema compatibility. |
| `article-help/executable-examples`, `mcp/discovers-artifact-command-tree`, `gene/help.yaml` | Case-derived examples, compact discovery and whole generated help. |

The routine Rust cases test mechanism boundaries and reuse the corresponding source envelope budgets; they do not establish a native source maximum. Fixed usage replies use their authored error-envelope bound. Shared health/configuration contracts retain their shared bounds. No internal-only coverage or investigation projection is counted as successful product parity.

### Final generated help and discovery budgets

The local help preflight j13380 measured search diagnostic at 2026 compact/2027 stdout bytes and get diagnostic at 2223 compact/2224 stdout bytes. The PM approved get-help growth to 2557 compact/2558 stdout because the canonical described enum now makes its five sections visible. Search retains its existing 2200-byte ceiling. Sync help measured 373 stdout bytes for GTR and 389 for WHO and FDA CDx; each exposes exactly two case-derived examples. Their new help-case ceilings use measured stdout plus fifteen percent.

Whole help measured 56826 stdout bytes and retains its existing 63341 ceiling. MCP discovery measured 1920 bytes and retains its 1923 ceiling. It adds schema-derived diagnostic source choices, native selectors, the supplement selector and sections while preserving every PGx paging flag. The compact intro points to generated help and schema discovery instead of repeating utility-command leaves; full generated help still contains every utility command. No public command, source selector or section was removed to fit discovery. The shared executable-example check executes every published example against its owning synthetic recording.

The Rust diagnostic mechanism maxima measured by j13382 live in `runner/diagnostic_retirements.rs::bounded`. The existing isolated invocation path applies them to diagnostic request and sync replies, while registry health/configuration remain under their shared contracts. Each maximum is a complete compact synthetic envelope by schema, result kind and outcome, with fifteen percent headroom. Native page maxima and YAML comparable ceilings remain separate.

Direct process launches retain the same budget checks: concurrent FDA replacement receipts and readers use the measured compact envelope class. The three synthetic Markdown mechanism cases measured complete stdout at 1531 GTR, 1714 WHO and 1965 FDA CDx bytes in j13400. Their direct stdout checks allow fifteen percent headroom, rounded up; they do not subtract publisher terms or claim native source maxima.

### WHO stored product discovery

`search diagnostic --source who-ivd --marker HIV` matches literal whole words in `Pathogen/Disease/Marker`. `--manufacturer` matches `Manufacturer name`; both filters must match the same native row. Matching ignores case, preserves literal phrase order and requires whole-word boundaries. It adds no synonyms, disease resolution or clinical interpretation. A punctuation-only selector fails before storage access. WHO rejects GTR and FDA selectors before cache or I/O. The retired WHO `--disease` spelling names `--marker`; `--type` remains deferred.

The stored complete bundle supplies one header per WHO Product ID. Groups follow the first occurrence of each product in the export, even when matching code rows are interleaved. Header name, manufacturer and marker come from the first matching row. The header does not claim that those cells describe every row in that product. Exact cards retain all rows. The default page contains five product identities; its total counts matching products, not native code rows. Every header has an exact WHO get action before continuation. Get and page actions bind the same stored content revision. Reads, fresh reads and page walks never synchronize or download. A replacement snapshot rejects an old continuation. Missing or stale data uses the existing explicit sync action and source policy.

The retained WHO capture inventory contains 435 complete rows and 125 products, with eight named nonempty columns. Its archive hash was verified before inspecting inventory. Native runtime proof remains separate from synthetic routine cases; no provider request is part of this addition. `diagnostic/who-search.yaml` owns selector results and CLI/MCP parity. The WHO search mechanism cases own the complete no-download page walk, exact actions, replacement rejection and policy admission.

Synthetic CLI measurements from j13465 record full compact bytes and runner-comparable stdout bytes separately: default HIV page 2249/2057, manufacturer page 1422/1230, combined filters 818/626, no match 464/272 and terminal page 1190/998. Publisher terms remain in every full response. Case ceilings apply fifteen percent headroom to the runner's actual measurement, not to a projection. Source guidance measures 143 compact and 144 stdout bytes. Whole help measures 57118 stdout bytes and fits its existing ceiling. These synthetic sizes are not native maxima or source timings.

The contained retained WHO proof j13485 ran 90 CLI/MCP pairs over the unchanged complete export. HIV matched 62 products across 13 pages; manufacturer `InTec Products, Inc.` matched five products and its HIV conjunction matched two. The literal `Mycobacterium tuberculosis` phrase matched five products; reversed phrase order, `HI` as a partial-word probe and a verified absent marker matched zero. Seventy followed exact cards preserved every original native cell and row. Every get/page action retained the stored revision. The real recorder received the unchanged body once through the contained endpoint; replay and stored reads made no provider requests.

Native full/comparable compact bytes were 2527/2334 for default HIV, 2671/2478 for the largest HIV page, 1226/1033 for its terminal page, 2149/1956 for manufacturer, 1168/975 for both filters and 2272/2079 for the literal marker phrase. These are actual retained-product measurements; they do not measure provider speed or a released 0.9 executable.

Final preflight j13494 measures search help at 2306 compact/2307 stdout bytes; its 2654-byte case ceiling stays below the PM-approved maximum. Whole help measures 57106 stdout bytes and keeps its existing ceiling. MCP discovery measures 1917 bytes and keeps its existing ceiling. The reversed manufacturer phrase measures 478 full compact/286 runner stdout bytes; its case ceiling is 329 bytes. The native proof archive is `data/biomcp-redux/0185-who-ivd-product-proof/j13485-native-proof.tar.gz`, SHA-256 `272863da262c3fa9db14f7b7fdcdf06a2e8d4a1855ca62197db9bc4bc1e5c2d5`; independent verification checked all 683 files and exact membership before scratch cleanup.
