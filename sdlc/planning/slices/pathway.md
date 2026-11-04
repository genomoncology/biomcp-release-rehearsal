# Pathway slice investigation and proposed design

This proposal applies steps 1–4 of [the slice process](../slices.md). It starts from `85e6ba864c3f13ec50432a58670ec2af13e12857`. It proposes entity access to human pathways. It does not infer treatment, calculate enrichment or run analysis tools. The PM reviews the proposed command and source choices before implementation tickets exist.

## Source boundaries

Reactome and WikiPathways remain distinct sources. Their identifiers define separate entities. A common title, overlapping members or a literature citation does not establish pathway equivalence. Human records must carry source-stated species identity. Source release and retrieval time accompany observations. A record with an unexpected identity fails admission and cannot populate the requested card.

The [entity boundary ruling](../../decisions/2026-10-25-entity-boundary-rulings.md) drops g:Profiler and `enrich`. KEGG stays dropped under the existing gene ruling and its reuse terms. This investigation makes no request to either service. Reactome database data use CC0; Reactome images and software have different licenses. The proposed adapter requests database JSON only. WikiPathways content uses CC0; publisher terms and exact current response fields must be verified from the retained capture before types are approved. The captured WikiPathways [help page](https://www.wikipathways.org/help.html) declares its CC0 waiver. The attempted `/license.html` returned a genuine 404 and is not the terms URL. Both proposed sources belong to the open group, require no key and obey source disabling before network or bundle access.

## Questions and acceptable answers

The question bank separates a source's membership statement from a clinical conclusion. A complete acceptable answer names the source, exact pathway identifier, human species, source version, requested relation, and any limits or failures. Membership never means that every member is actionable, that a drug acts on the whole pathway or that a patient will respond.

The source capture asks 24 literal pathway queries. These are adapted research prompts, not quotations from patients or publications. The source cards explicitly carry the publication references below. Clinical review validates their meaning and the boundaries of these accepted retrieval answers. The clinical reviewer accepted the corrected 26 research questions and their source-bound interpretation. Source failure and empty-result controls sit outside that count.

<!-- question-bank -->
These 26 paper-derived retrieval questions are paraphrases, not quotations. Each accepted answer is a source identity, source reference or membership observation. Release97 qualifies the Reactome observations. Empty and failure controls follow separately.

| # | Research question | Published anchor | Accepted source-bound answer | Answer status and case |
| --- | --- | --- | --- | --- |
| 1 | Which human MAPK-mutant pathway carries the source citation for the cervical cancer study? | [PMID24390348](https://pubmed.ncbi.nlm.nih.gov/24390348/); [native card](https://reactome.org/content/detail/R-HSA-9652817) | `R-HSA-9652817` — Signaling by MAPK mutants; `Homo sapiens`; `R-HSA-9652817.2`. Native card references PMID24390348. | Answered; `pathway/card-R-HSA-9652817.yaml`. |
| 2 | Which pathway record contains the BRAF fusion context discussed in pilocytic astrocytoma research? | [PMID18974108](https://pubmed.ncbi.nlm.nih.gov/18974108/); [native card](https://reactome.org/content/detail/R-HSA-6802952) | `R-HSA-6802952` — Signaling by BRAF and RAF1 fusions; `Homo sapiens`; `R-HSA-6802952.3`. Native card references PMID18974108. | Answered; `pathway/card-R-HSA-6802952.yaml`. |
| 3 | What is the exact human pathway record for EGFR signaling? | [PMID15142631](https://pubmed.ncbi.nlm.nih.gov/15142631/); [native card](https://reactome.org/content/detail/R-HSA-177929) | `R-HSA-177929` — Signaling by EGFR; `Homo sapiens`; `R-HSA-177929.6`. Native card references PMID15142631. | Answered; `pathway/card-R-HSA-177929.yaml`. |
| 4 | Which human Hippo signaling record can support retrieval of the pathway studied in tumor signaling research? | [PMID20951342](https://pubmed.ncbi.nlm.nih.gov/20951342/); [native card](https://reactome.org/content/detail/R-HSA-2028269) | `R-HSA-2028269` — Signaling by Hippo; `Homo sapiens`; `R-HSA-2028269.3`. Native card references PMID20951342. | Answered; `pathway/card-R-HSA-2028269.yaml`. |
| 5 | Which ALK signaling pathway cites the NPM-ALK lymphoma fusion study? | [PMID8122112](https://pubmed.ncbi.nlm.nih.gov/8122112/); [native card](https://reactome.org/content/detail/R-HSA-201556) | `R-HSA-201556` — Signaling by ALK; `Homo sapiens`; `R-HSA-201556.5`. Native card references PMID8122112. | Answered; `pathway/card-R-HSA-201556.yaml`. |
| 6 | Which NTRK1 signaling record cites the Trk receptor signaling review? | [PMID12676795](https://pubmed.ncbi.nlm.nih.gov/12676795/); [native card](https://reactome.org/content/detail/R-HSA-187037) | `R-HSA-187037` — Signaling by NTRK1 (TRKA); `Homo sapiens`; `R-HSA-187037.4`. Native card references PMID12676795. | Answered; `pathway/card-R-HSA-187037.yaml`. |
| 7 | Which human DNA repair record cites DNA repair dysregulation research? | [PMID23175119](https://pubmed.ncbi.nlm.nih.gov/23175119/); [native card](https://reactome.org/content/detail/R-HSA-73894) | `R-HSA-73894` — DNA Repair; `Homo sapiens`; `R-HSA-73894.5`. Native card references PMID23175119. | Answered; `pathway/card-R-HSA-73894.yaml`. |
| 8 | Which source record describes homology-directed repair through homologous recombination? | [PMID20965415](https://pubmed.ncbi.nlm.nih.gov/20965415/); [native card](https://reactome.org/content/detail/R-HSA-5685942) | `R-HSA-5685942` — HDR through Homologous Recombination (HRR); `Homo sapiens`; `R-HSA-5685942.4`. Native card references PMID20965415. | Answered; `pathway/card-R-HSA-5685942.yaml`. |
| 9 | Which mismatch repair pathway cites the research review on defects in Lynch syndrome? | [PMID23572416](https://pubmed.ncbi.nlm.nih.gov/23572416/); [native card](https://reactome.org/content/detail/R-HSA-5358508) | `R-HSA-5358508` — Mismatch Repair; `Homo sapiens`; `R-HSA-5358508.3`. Native card references PMID23572416. | Answered; `pathway/card-R-HSA-5358508.yaml`. |
| 10 | Which TP53 pathway records transcription of death receptors and ligands? | [PMID10777207](https://pubmed.ncbi.nlm.nih.gov/10777207/); [native card](https://reactome.org/content/detail/R-HSA-6803211) | `R-HSA-6803211` — TP53 Regulates Transcription of Death Receptors and Ligands; `Homo sapiens`; `R-HSA-6803211.2`. Native card references PMID10777207. | Answered; `pathway/card-R-HSA-6803211.yaml`. |
| 11 | Which human apoptosis record cites the apoptosis-and-disease review? | [PMID15218528](https://pubmed.ncbi.nlm.nih.gov/15218528/); [native card](https://reactome.org/content/detail/R-HSA-109581) | `R-HSA-109581` — Apoptosis; `Homo sapiens`; `R-HSA-109581.6`. Native card references PMID15218528. | Answered; `pathway/card-R-HSA-109581.yaml`. |
| 12 | Which exact human Cell Cycle record provides pathway context for the TCGA signaling study? | [PMID29625050](https://pubmed.ncbi.nlm.nih.gov/29625050/); [native card](https://reactome.org/content/detail/R-HSA-1640170) | `R-HSA-1640170` — Cell Cycle; `Homo sapiens`; `R-HSA-1640170.5`. Paper supplies the retrieval topic; the source card supplies identity. | Answered; `pathway/card-R-HSA-1640170.yaml`. |
| 13 | Which WNT pathway carries the source references for beta-catenin signaling and disease? | [PMID19619488](https://pubmed.ncbi.nlm.nih.gov/19619488/); [native card](https://reactome.org/content/detail/R-HSA-195721) | `R-HSA-195721` — Signaling by WNT; `Homo sapiens`; `R-HSA-195721.7`. Native card references PMID19619488. | Answered; `pathway/card-R-HSA-195721.yaml`. |
| 14 | Which exact human NOTCH signaling record provides source context for the TCGA signaling study? | [PMID29625050](https://pubmed.ncbi.nlm.nih.gov/29625050/); [native card](https://reactome.org/content/detail/R-HSA-157118) | `R-HSA-157118` — Signaling by NOTCH; `Homo sapiens`; `R-HSA-157118.7`. Paper supplies the retrieval topic; the source card supplies identity. | Answered; `pathway/card-R-HSA-157118.yaml`. |
| 15 | Which Hedgehog signaling pathway cites the development-and-cancer review? | [PMID19081070](https://pubmed.ncbi.nlm.nih.gov/19081070/); [native card](https://reactome.org/content/detail/R-HSA-5358351) | `R-HSA-5358351` — Signaling by Hedgehog; `Homo sapiens`; `R-HSA-5358351.5`. Native card references PMID19081070. | Answered; `pathway/card-R-HSA-5358351.yaml`. |
| 16 | Which pathway specifically records gene and protein expression after IL-12 stimulation? | [PMID16038020](https://pubmed.ncbi.nlm.nih.gov/16038020/); [native card](https://reactome.org/content/detail/R-HSA-8950505) | `R-HSA-8950505` — Gene and protein expression by JAK-STAT signaling after Interleukin-12 stimulation; `Homo sapiens`; `R-HSA-8950505.5`. Native card references PMID16038020. | Answered; `pathway/card-R-HSA-8950505.yaml`. |
| 17 | Which human autophagy record cites the autophagosome biogenesis review? | [PMID24296784](https://pubmed.ncbi.nlm.nih.gov/24296784/); [native card](https://reactome.org/content/detail/R-HSA-9612973) | `R-HSA-9612973` — Autophagy; `Homo sapiens`; `R-HSA-9612973.4`. Native card references PMID24296784. | Answered; `pathway/card-R-HSA-9612973.yaml`. |
| 18 | Which insulin receptor pathway cites the insulin signaling review? | [PMID11282018](https://pubmed.ncbi.nlm.nih.gov/11282018/); [native card](https://reactome.org/content/detail/R-HSA-74751) | `R-HSA-74751` — Insulin receptor signalling cascade; `Homo sapiens`; `R-HSA-74751.5`. Native card references PMID11282018. | Answered; `pathway/card-R-HSA-74751.yaml`. |
| 19 | Which interferon signaling pathway cites the type-I/type-II signaling review? | [PMID15864272](https://pubmed.ncbi.nlm.nih.gov/15864272/); [native card](https://reactome.org/content/detail/R-HSA-913531) | `R-HSA-913531` — Interferon Signaling; `Homo sapiens`; `R-HSA-913531.6`. Native card references PMID15864272. | Answered; `pathway/card-R-HSA-913531.yaml`. |
| 20 | Which exact Reactome record corresponds to the Reactome2024 paper example for AKT1 E17K? | [Reactome2024](https://pmc.ncbi.nlm.nih.gov/articles/PMC10767911/) | `R-HSA-5674400` — Constitutive Signaling by AKT1 E17K in Cancer; `Homo sapiens`; `R-HSA-5674400.3`. This identifies the published example without inferring a treatment effect. | Answered; `pathway/card-R-HSA-5674400.yaml`. |
| 21 | Which exact Reactome record corresponds to the Reactome2024 paper example for PI3K/AKT signaling in cancer? | [Reactome2024](https://pmc.ncbi.nlm.nih.gov/articles/PMC10767911/) | `R-HSA-2219528` — PI3K/AKT Signaling in Cancer; `Homo sapiens`; `R-HSA-2219528.4`. This identifies the published example without inferring a treatment effect. | Answered; `pathway/card-R-HSA-2219528.yaml`. |
| 22 | Which exact Reactome record corresponds to the Reactome2024 paper example for ERBB2 signaling in cancer? | [Reactome2024](https://pmc.ncbi.nlm.nih.gov/articles/PMC10767911/) | `R-HSA-1227990` — Signaling by ERBB2 in Cancer; `Homo sapiens`; `R-HSA-1227990.6`. This identifies the published example without inferring a treatment effect. | Answered; `pathway/card-R-HSA-1227990.yaml`. |
| 23 | Which exact Reactome record corresponds to the Reactome2024 paper example for ribavirin disposition? | [Reactome2024](https://pmc.ncbi.nlm.nih.gov/articles/PMC10767911/) | `R-HSA-9755088` — Ribavirin ADME; `Homo sapiens`; `R-HSA-9755088.3`. This identifies the published example without inferring a treatment effect. | Answered; `pathway/card-R-HSA-9755088.yaml`. |
| 24 | Which exact BRAF reference appears among RAF/MAPK physical participants? | [PMID12068308](https://pubmed.ncbi.nlm.nih.gov/12068308/); captured RAF/MAPK participants | `uniprot:P15056`, `ReferenceGeneProduct`, native label `UniProt:P15056 BRAF`. This is source-stated membership, not actionability. | Answered; `pathway-sections/native-reference-walk`. |
| 25 | Which KRAS isoforms does the source preserve in RAF/MAPK participants? | [PMID17496910](https://pubmed.ncbi.nlm.nih.gov/17496910/); captured RAF/MAPK participants | `uniprot:P01116-1` and `uniprot:P01116-2`, both `ReferenceIsoform`; preserve both and require an exact approved gene join. | Answered; `pathway-sections/native-reference-walk`. |
| 26 | Which MAP2K1 and MAP2K2 identifiers can support the next gene lookup? | [PMID22177953](https://pubmed.ncbi.nlm.nih.gov/22177953/); captured RAF/MAPK participants | `uniprot:Q02750` and `uniprot:P36507`; retain namespace and resolve through the existing checked HGNC map rather than extracting symbols. | Answered; `pathway-sections/native-reference-walk`. |

Technical controls: the exact sentinel query `zzzzNoPathway0191` returns the retained Reactome structured no-entry 404 and no literal WikiPathways title match. An unknown Reactome ID returns its genuine 404; that is not a zero-member valid pathway. The Reactome2024 paper’s older fibrin-clot example `R-HSA-140877` also returns404 at release 97, so this design preserves that historical-ID failure without silently searching for a replacement.
<!-- /question-bank -->

## Direct source evidence

All live calls run through `pm job` at the pushed starting revision. Each data request is one GET with a deadline, an eight-MiB response admission bound and no automatic retry or redirect. Two separate HEAD requests measure bundle metadata without downloading the bodies. The evidence retains raw bytes, status, headers, elapsed wall time and SHA-256. A truncated body or malformed reply remains a failed observation. The captured search no-match is HTTP 404 with a structured `NOT_FOUND` body and exact query echo. Only that reviewed endpoint-specific shape can mean empty; a generic 404, HTML error or foreign query cannot. No investigator keys enter the request scripts or evidence.

<!-- source-measurements -->
The first job, `j14201`, retained 38 requests; `j14212` retained 31; `j14219` retained 3; `j14228` retained 1 GET and 2 HEAD observations. All ran on the M5 at the verified pushed starting revision, with shared-machine load, cold Python HTTP clients and no application cache. These are source-service timings, not product or dedicated-machine speed baselines. No call retried or followed a redirect. Capture times are in the receipts.

| Observation | Exact result | Cost and limits |
| --- | --- | --- |
| Reactome RAF/MAPK card | R-HSA-5673001.12, human, release 97 | 23,877 raw bytes; 0.185s; one GET |
| Reactome physical participants | 151 physical entities; 1,010 reference occurrences; 279 distinct UniProt-tagged references | 244,106 raw bytes; 0.268s; one GET |
| Reactome contained events | 81 entries: 69 objects and 12 integer references | 30,885 raw bytes; 0.325s; one GET |
| Reactome BRAF accession mappings | 16 native pathway mappings for P15056 | 6,071 raw bytes; 0.816s; one GET |
| Reactome human top-level | 29 source-stated human top-level records | 12,191 raw bytes; 0.542s; one GET |
| WikiPathways search catalog | 2,218 catalog rows; 1,131 human rows | 2,346,682 raw bytes; 0.625s; one GET |
| WikiPathways get catalog | 2,218 catalog rows; 1,131 human rows | 1,224,217 raw bytes; 0.370s; one GET |

The 23 successful research-query searches returned 6–10 native rows each, with reported `numberOfMatches` between 6 and 1,252. Median request time was 0.203s; range 0.145–0.597s. The legacy `pageSize=5` was ignored. Native `rows=5` returned five rows in the contract probe. `start=5` returned the same IDs as `start=0`, so that spelling does not prove source paging. The captured official OpenAPI names `Start row` and `Force filters` with spaces and documents default filter relaxation. These observations disproved the guessed paging contract. The follow-up search admission witnesses below prove the literal documented spellings and disabled-relaxation behavior.

The [capture-limit ruling](../../decisions/2026-10-26-protein-pathway-study-rulings.md#capture-limit-rulings) requires native Reactome search pages through the proven `Start row` parameter, with `rows` equal to the requested `--limit`. Each page applies exact admission, may contain fewer admitted rows than the limit, advances its continuation from the native position and retains the source-reported native total. It never fetches the 1,252-row window to display five headers. Do not claim a complete 917-result MAPK walk from five or ten observed rows. Human species and accepted `Pathway`/`TopLevelPathway` classes remain mandatory per-row admission checks even if the provider relaxes filters. The first result for the literal KRAS query was Hippo; expose native rank rather than promising literal title matching.

| Harvest query | Reactome rows returned / native matches | WikiPathways human literal-title matches |
| --- | --- | --- |
| MAPK signaling | 10 / 917 | 4 |
| Signaling by BRAF | 10 / 900 | 0 |
| Signaling by EGFR | 10 / 913 | 0 |
| Signaling by ERBB2 | 10 / 911 | 0 |
| Signaling by KRAS | 10 / 901 | 0 |
| Signaling by PI3K | 10 / 909 | 0 |
| Signaling by ALK | 10 / 914 | 0 |
| Signaling by NTRK1 | 10 / 899 | 0 |
| DNA Repair | 10 / 455 | 1 |
| Homologous Recombination Repair | 10 / 199 | 0 |
| Mismatch Repair | 10 / 167 | 1 |
| TP53 Regulates Transcription | 10 / 769 | 0 |
| Apoptosis | 10 / 192 | 11 |
| Cell Cycle | 10 / 1252 | 3 |
| Signaling by WNT | 10 / 907 | 0 |
| Signaling by NOTCH | 10 / 905 | 0 |
| Signaling by Hedgehog | 10 / 901 | 0 |
| Signaling by JAK-STAT | 10 / 905 | 0 |
| Autophagy | 10 / 39 | 7 |
| Insulin receptor signalling cascade | 10 / 1051 | 0 |
| Hemostasis | 6 / 6 | 0 |
| Interferon Signaling | 10 / 925 | 3 |
| Metabolism | 10 / 368 | 111 |
| zzzzNoPathway0191 | 0 / structured no-entry 404 | 0 |

The WikiPathways column evaluates the unchanged phrase against source titles in the single directly fetched catalog. It measures that narrow title selector, not absence of a biological pathway from WikiPathways. It does not translate a Reactome name into a WikiPathways name. Its search catalog also carries `datanodes` and annotations as strings; these are not checked gene IDs. The get catalog carries source-truncated descriptions, so a copied ellipsis must not be presented as a whole pathway explanation.

The xref catalog exceeded the chosen 8-MiB bound at 8,388,609 bytes. That truncated body is retained and is not parsed as valid JSON or evidence of a complete member relation. The exact publisher-linked WP382 GPML was captured separately: 191,701 bytes, 0.306 s, HTTP 200; identity `WP382_r140476`, organism `Homo sapiens`, last-modified `20250829230015`. It contains 263 DataNodes: 252 GeneProduct, 6 Metabolite and 5 Pathway. The xref namespaces include 141 Entrez Gene, 110 Ensembl, 1 HGNC, 3 WikiPathways, 3 ChEBI, 3 HMDB and 2 empty values. Node counts are not distinct gene counts. Exact positive examples include TRAF6 → Entrez 7189 and ELK4 → ENSG00000158711. GPML is a separately proposed source component; implementing it needs its own reviewed XML parser and bounded source contract, rather than silently replacing a failed catalog leg.

The publisher monthly archive lists the human GPML bundle dated 2026-10-10. Its direct HEAD reported Content-Length 11,271,102 bytes; no ZIP body was downloaded. Decompressed bytes, complete gene count and build memory remain unmeasured. Reactome documents quarterly release archives; its named mapping-file HEAD returned 403. That failure does not establish that mapping data are missing or permit an alternate URL. The existing online mapping is sufficient for the initial Reactome design.

The independently verified evidence archive is `data/biomcp-redux/0191-pathway-investigation/0191-pathway-investigation-evidence.tar.gz` on Yellow, SHA256 `7d7a46ccaf538be9bf10393d6d5a336072a75e1fdd2cdc2fd271bef2888f26a3`, 3,194,411 bytes. It has 13 exact members, including 12 manifest-verified files and all four original component archives unchanged. Independent verification on Yellow checked every outer file and every nested component member. It also preserves the exact closed-issue inventory, request recipes, projection recipe and scope report.
<!-- /source-measurements -->

### Search admission witnesses

The follow-up capture `j14353` ran once on the M5 from landed revision `6e20aa069bbb981f9c9c6f0da3fd4ba31e556ceb`. It admitted twelve fixed keyless GETs in their declared order, with one second between admissions, the existing Reactome eight-second deadline and an eight-MiB body bound. It made no retry, redirect, alternate-host request, extra page, bundle request or request to KEGG or g:Profiler. The fixed recipe stops the entire source on 403, 429, 5xx, malformed content, transport failure or an oversized reply. Each independent one-row subplan uses `make source-capture`; only the previously reviewed structured no-entry 404 may allow the next predeclared witness. The helper records that 404 as `provider_failure`; the analysis separately records `empty` only after checking its code, reason, endpoint, exact query echo and null targets. This does not widen the product's empty contract.

Every witness below comes from Yellow's `data/biomcp-redux/0206-pathway-search-captures/j14353-pathway-search-captures.tar.gz`, SHA256 `e970b8bdbbf8865ce1a805b59c253e43ed0005b195fc2169fc5669b6d37299aa`, 4,541,203 bytes. The archive has 115 exact members and 114 manifest-verified files. Independent verification after storage also checked every exact member and all 60 manifest-listed files in its twelve inner capture archives. `fixed-plan.json` names every complete URL before admission; `openapi.json` is the unchanged official document retained by 0191, SHA256 `fce0763c2b0a6c88c7d2ca8928dcb22838423913919627a08266dbac75649dd6`. `ledger.json` records each exact URL, status, body hash, bytes and elapsed time. `capture-NN/0000.body`, `.headers` and `.receipt.json` contain the row named below. Counts describe this observation and are not permanent source expectations.

| Witness and archive row | Result | Capability or required fallback |
| --- | --- | --- |
| Paging, `00` and `01` | Literal OpenAPI parameter `Start row`, encoded `Start+row`, with values 0 and 5 returns two different five-row windows for `MAPK signaling`: 5,858 bytes / 0.146 s and 10,389 bytes / 0.109 s. Both report 917 native matches. The concatenated ten IDs have no duplicate and match the ten IDs in 0191's retained native first window, in order. | The spelled parameter supports these consecutive native windows. The ignored `start` alias remains forbidden. This proves the measured transition, not a complete 917-row walk. Product continuations request the next native page with `rows` equal to `--limit`; retained pages preserve admitted native order. |
| Human species, `02` and `03` | An unfiltered five-per-group request returns 43 entries spanning ten species labels and multiple classes. Literal `species=Homo+sapiens` returns 39 entries across groups, all source-labeled `Homo sapiens`: 26,223 bytes / 0.173 s versus 29,158 bytes / 0.172 s. | The species spelling has a positive narrowing witness. Species identity remains mandatory per-row admission. `rows` does not bound the sum across unrelated result groups. |
| Pathway classes, `00`, `04`, `05` and `06` | `types=Pathway` returns five exact Pathway rows. `types=TopLevelPathway` for MAPK returns the exact structured no-entry 404, 162 bytes / 0.165 s. Repeated `types=Pathway&types=TopLevelPathway` returns five Pathway rows for MAPK, 5,858 bytes / 0.196 s. The larger Cell Cycle window contains 1,217 exact Pathway rows, 21 TopLevelPathway rows and 14 CellLineagePath rows. | The source class selector is not an exact-class admission guarantee. The positive standalone TopLevelPathway filter remains unproven; its MAPK witness is empty. Every returned row must pass the exact accepted class check. Reject all 14 CellLineagePath rows from the measured window; preserve their source-outcome evidence and never call the 1,252 native matches a count of admitted pathway entities. Species and class remain per-row checks in every case. |
| Source window, `06`, `07` and `08` | Cell Cycle with `rows=1252`, `1253` and `10000` returns the same 1,252 distinct native IDs in the same order and byte-identical 2,482,173-byte bodies. Elapsed times are 1.765 s, 0.911 s and 0.848 s. Each response reports `rowCount=1252` and `numberOfMatches=1252`. | A 1,252-row native window is measured below the existing deadline and body bound. It does not prove that the source can return 10,000 actual rows or establish its absolute maximum. Exact admission retains 1,238 human Pathway/TopLevelPathway rows for this observation. The product requests only `rows=--limit` for each native page under the capture-limit ruling. It retains the native total without replacing it with an admitted-row count, allows short admitted pages and advances from the native position. The large-window measurement does not authorize product prefetch, another capture or a full walk. |
| Forced filters, `09` and `10` | The fixed impossible-species control `species=zzzzNoSpecies0206`, `types=Pathway`, `Force filters=false` returns relaxed mixed-species, mixed-class results: 43 entries, 12,035 native matches, 26,223 bytes / 0.200 s. Changing only the literal documented `Force filters` value to `true` returns the exact structured no-entry 404: 162 bytes / 0.111 s. | Literal `Force filters`, encoded `Force+filters`, disables the observed relaxation. Search requests use true. The control is a synthetic filter witness, not a claim that the sentinel species exists. Exact human and class admission still applies after successful source filtering. |
| Empty sentinel, `11` | `zzzzNoPathway0191` under the same human species, repeated accepted classes, `rows=1252`, `Start row=0` and `Force filters=true` returns HTTP 404, `code:404`, `reason:NOT_FOUND`, the exact ContentService search endpoint, `messages:["No entries found for query: zzzzNoPathway0191"]` and `targets:null`: 165 bytes / 0.135 s. | This endpoint-specific, exact-query no-entry shape supports empty. A generic 404, different query echo, foreign endpoint, HTML or malformed body remains failure. |

The first paging window is `R-HSA-9652817`, `R-HSA-6802957`, `R-HSA-5683057`, `R-HSA-372708`, `R-HSA-354194`. The next is `R-HSA-5674135`, `R-HSA-2028269`, `R-HSA-449147`, `R-HSA-5654738`, `R-HSA-5654736`. These identifiers bind the retained observation; future live ranks and counts may change. The measurements use fresh curl processes on a shared machine with no product cache. They measure source replies, not product output or a dedicated-machine speed baseline. The existing 0.9 comparison remains unmeasured.

The product contract uses native Reactome paging with literal `Start row`, `rows` equal to `--limit` and `Force filters=true`. Exact human/class admission applies independently to each source page. Rejected rows may shorten the displayed page; the command does not fetch extra rows to fill it. Continuation advances from the native position and preserves the source-reported native total. Each requested page is bounded; this capture does not authorize an automatic whole-source walk. A failed or empty class-filter witness unlocks no weaker admission rule. The evidence archive retains raw replies and the complete recipe outside Git; this change adds no product code or fixtures.

### Exact command search requests

The command implementation reuses the accepted identity cards, human top-level relation, database release and BRAF accession mapping from the investigation archives above. Its additional capture declares 25 exact missing keyless search requests before admission. Each request uses the final repeated accepted class filters, literal `Start row`, `Force filters=true` and `rows=5`. The plan uses the existing eight-second deadline and eight-MiB body bound, one second between requests, no redirect and zero retries. It contains the remaining harvest first pages, the next MAPK and Hemostasis native pages, and the final empty sentinel. It performs no full-window search or new card, mapping, bundle or publication request.

Job `j14564` at `81ea22bf` captured 24 HTTP 200 replies and the final exact structured no-entry HTTP 404. The helper stops on that last provider rejection and exits two; the archive retains every planned request. The reviewed sentinel body echoes only `zzzzNoPathway0191`, names the native search endpoint, states `code:404` and `reason:NOT_FOUND`, and includes `targets:null`. This observation does not admit a generic route 404 or another query's response as empty. Successful replies range from 2,246 to 22,870 bytes and 0.106 to 0.492 seconds on the shared build machine.

The preserved archive is `data/biomcp-redux/0216-reactome-commands/native-search.tar.gz` on Yellow, SHA256 `0bf01b60951965eebf178e99408905446f3fb002a727af19c51e46f7c0c0af47`. Independent verification checked its complete 78-member list and all 77 manifest-listed file hashes. The original source bodies remain unchanged in the archive; the replay fixtures use the same JSON values and the transport's canonical request spelling. The archive's plan, receipts and native bodies bind the observations to their exact URLs rather than a reconstructed source query.

### Product disclosure measurements

Job `j14621` at `cb0a5a3f` executes the retained command cases through CLI, MCP and Markdown. It measures 106 command steps, including invalid selectors, source failures and local continuation pages. Every CLI envelope equals the MCP structured result and parsed MCP text. The measurements use local replay bodies and make no provider request. Comparable JSON removes exactly the registry's `use`, `terms` and `terms_url` fields from each validated source outcome. Tokens use compact UTF-8 bytes divided by four, rounded up. Each case's checked-in `max_bytes` and `max_tokens` is its own observed size plus fifteen percent, rounded up; the table shows the largest member of each command form.

| Command form | Largest comparable CLI bytes | Largest tokens | Largest byte ceiling | Largest token ceiling |
| --- | --- | --- | --- | --- |
| Header pages | 1551 | 388 | 1784 | 447 |
| Default card | 534 | 134 | 615 | 155 |
| Whole summary | 10524 | 2631 | 12103 | 3026 |
| Gene relation page | 1540 | 385 | 1771 | 443 |
| Top-level relation page | 1516 | 379 | 1744 | 436 |

The database's complete source summaries vary in length. Ten observed summaries exceed the design's 900-token allocation while retaining whole native summation objects and source citations. The largest is the ALK pathway `R-HSA-201556`, with 2,631 comparable tokens. These measurements preserve source text and do not replace it with a locally written summary. The source observations include PubMed references on 21 of the 23 question cards; the captured NOTCH and Cell Cycle cards contain no native PMID. Citation cases assert the source's native reference positions, and URL-only references retain `uniformResourceLocator` as their source URL beside optional PubMed identities.

The approved whole-summary budget uses each observed disclosure's measured ceiling. Summation objects and native references remain complete. The following per-record limits are enforced by their caller cases.

| Native summary ID | Measured tokens | Token ceiling | Measured CLI bytes | Byte ceiling |
| --- | --- | --- | --- | --- |
| `R-HSA-109581` | 1010 | 1162 | 4041 | 4648 |
| `R-HSA-1227990` | 1645 | 1892 | 6581 | 7569 |
| `R-HSA-177929` | 1057 | 1216 | 4226 | 4860 |
| `R-HSA-195721` | 1554 | 1788 | 6214 | 7147 |
| `R-HSA-201556` | 2631 | 3026 | 10524 | 12103 |
| `R-HSA-2028269` | 1449 | 1667 | 5796 | 6666 |
| `R-HSA-2219528` | 2185 | 2513 | 8739 | 10050 |
| `R-HSA-5358508` | 1100 | 1265 | 4399 | 5059 |
| `R-HSA-73894` | 1539 | 1770 | 6157 | 7081 |
| `R-HSA-5673001` | 1197 | 1377 | 4787 | 5506 |

The measurement JSON is preserved on Yellow at `data/biomcp-redux/0216-reactome-commands/output-measurements.json`, SHA256 `4b5dd0d6f40fe8ba8d7ac7ccaca904b427ddf15376f8af4d7083ea9fcf810ca8`. Transfer verification checks the complete file hash and every row's CLI/MCP equality result. The text-search zero-total boundary increment runs in `j14641`. HTTP200 with native total zero is a bad reply, while rejection of every row of a positive-total native page preserves that total, its native next offset and all admission problems. The four CLI/MCP/Markdown step measurements are separately preserved on Yellow at `data/biomcp-redux/0216-reactome-commands/boundary-measurements.json`, SHA256 `eeef5fbad5de082dc52d9c8f405f3ad7486946ae5903cbbe857febd12ea2fe70`. The permanent caller cases also exercise a retained pool under a foreign query, disabled-source admission before saved content, exact ordered gene/top-level walks, native search continuation and Markdown command shapes.

#### Retained Reactome relation contract

The 2026-10-26 bounded capture repeats the exact RAF/MAPK participant and contained-event requests without retries or hydration. It observes 151 physical entities, 1,010 reference occurrences and 332 distinct namespaced references. The 279 distinct UniProt references retain their complete accession and isoform spellings. Membership pages count distinct references in `page.total` with `total_kind=distinct_references`; `reference_occurrences` and `physical_entities` report the other observed counts. The retained snapshot keeps every physical entity context in source order. Default membership rows disclose their physical-context count. The explicit `pathway genes ID details` view discloses all contexts for the requested reference page. Details uses the same retained pool and keeps its mode on page and restart steps. Names never supply an identity. Native therapeutic namespaces and URLs remain visible without a drug action.

The same capture observes 81 contained events: 69 native objects and 12 unresolved integer references. The integer values remain explicit database references. Object IDs, classes and versions belong to each event. An object's publisher detail URL uses its own checked native stable identity. Only native Pathway and TopLevelPathway objects receive pathway get actions. The RAF-activation and TP53 death-receptor controls also contain integers. The approved object-only control is therefore synthetic; it proves parser and action behavior without making a native source-shape claim. No further source request replaces those observations.

The three-request archive is `data/biomcp-redux/0219-reactome-sections/native-1792997476083415000.tar.gz` on Yellow, SHA256 `cc647fb9324183868b64ffa93dd16fee3233bfe136a7f30e3759e6456aa01ff2`. The single corrected control archive is `data/biomcp-redux/0219-reactome-sections/native-1792997839035733000.tar.gz`, SHA256 `5d2c3a2092c4c27c9c35207192ab5231dea108e88a614926135651a333715a9b`. Both were verified before the jobs exited, copied to Yellow and checked against the destination archive hash.

The retained HGNC complete-set recording at `data/biomcp-redux/0102-article-short-pool/zero/hgnc/0887a33cc809657a2a564d65e8cbfcddf45adc3810a8bfb0653e03b418cf2953.json` contains 45,277 Approved rows; its decoded body SHA256 is `12363916be36f8f6f91879b0faade4eeaac973722de7136ec61329b77f2c2f91`. Its native mappings include KRAS HGNC:6407/P01116, MAP2K1 HGNC:6840/Q02750 and MAP2K2 HGNC:6842/P36507. The replay slice retains the 265 source rows covering the relation's 265 base accessions. An exact isoform remains a distinct reference while its validated accession joins through that checked HGNC map. Each displayed page uses one existing indexed AnyExact lookup. Previously retained HGNC rows receive an atomic UniProt-key backfill during cache migration, without another download.

Each complete relation has a source-bound immutable pool with the existing fixed lifetime. Continuations keep the relation kind and pathway identity, refuse foreign or expired tokens before source requests and never renew the pool lifetime. The default `all` reads the source summary plus each relation's bounded first page and performs no enrichment or therapeutic fanout.

The recorded CLI and MCP section measurements agree. The existing comparable-budget normalization removes only the validated registry use label and terms fields. The M5 replay proof starts each command with an isolated case cache and reuses it for the MCP call. These cases enforce measured size plus 15 percent.

| Disclosure | Measured bytes | Measured tokens | Byte ceiling | Token ceiling |
|---|---:|---:|---:|---:|
| `get pathway R-HSA-5673001 genes` | 2661 | 666 | 3061 | 766 |
| `get pathway R-HSA-5673001 events` | 2209 | 553 | 2541 | 636 |
| `get pathway R-HSA-5673001 all` | 8539 | 2135 | 9820 | 2456 |
| `pathway genes R-HSA-5673001` | 2220 | 555 | 2553 | 639 |
| `pathway events R-HSA-5673001` | 1768 | 442 | 2034 | 509 |
| `pathway genes R-HSA-5673001 details` | 10569 | 2643 | 12155 | 3040 |

The default Reactome card renders 723 raw Markdown bytes after adding the executable genes and events links. Excluding 106 source-terms bytes leaves 617 comparable bytes. Its measured ceiling is 710 comparable bytes, which adds fifteen percent and rounds up. The command-shape contract requires both links.

The compact genes default fits its original allocation. The `all` output preserves the complete source summary and both bounded compact section pages. Complete physical contexts appear only in the explicit details view. The first five references retain 29, 9, 16, 16 and 2 physical contexts respectively. The details view does not clip their native names or collapse isoform membership.

The default cards now expose the executable summary, genes and events steps. Their measured comparable byte ceilings include all three actions. The updated card cases enforce these per-record measurements plus 15 percent.

| Card | Measured bytes | Byte ceiling | Token ceiling |
|---|---:|---:|---:|
| `R-HSA-109581` | 592 | 681 | 171 |
| `R-HSA-1227990` | 617 | 710 | 179 |
| `R-HSA-157118` | 601 | 692 | 174 |
| `R-HSA-1640170` | 599 | 689 | 173 |
| `R-HSA-177929` | 600 | 690 | 173 |
| `R-HSA-187037` | 608 | 700 | 175 |
| `R-HSA-195721` | 599 | 689 | 173 |
| `R-HSA-201556` | 599 | 689 | 173 |
| `R-HSA-2028269` | 607 | 699 | 175 |
| `R-HSA-2219528` | 617 | 710 | 179 |
| `R-HSA-5358351` | 610 | 702 | 176 |
| `R-HSA-5358508` | 604 | 695 | 174 |
| `R-HSA-5673001` | 612 | 704 | 176 |
| `R-HSA-5674400` | 634 | 730 | 183 |
| `R-HSA-5685942` | 631 | 726 | 182 |
| `R-HSA-6802952` | 623 | 717 | 180 |
| `R-HSA-6803211` | 648 | 746 | 187 |
| `R-HSA-73894` | 587 | 676 | 170 |
| `R-HSA-74751` | 612 | 704 | 176 |
| `R-HSA-8950505` | 671 | 772 | 194 |
| `R-HSA-913531` | 603 | 694 | 174 |
| `R-HSA-9612973` | 598 | 688 | 173 |
| `R-HSA-9652817` | 614 | 707 | 178 |
| `R-HSA-9755088` | 603 | 694 | 174 |

## Public fields and identity joins

Reactome's query endpoint must prove the requested `stId`, `displayName`, `schemaClass`, species, summary and version before their types are accepted. Its participant endpoint describes physical entities, including complexes and modified proteins. A participant display name is not a gene symbol. The implementation must follow the publisher's reference entities and identifier namespaces. It must not extract symbols with regular expressions or treat each physical participant as one gene. Events may contain both objects and integer database references; every such reference needs either a checked hydration or an explicit unresolved record. Event records do not inherit the pathway's stable identifier. Only an event whose captured class is Pathway or TopLevelPathway may generate a `get pathway` action. Reactions and other event kinds retain source references and URLs without pretending to be pathway cards.

The landed gene section already joins HGNC's UniProt accessions to `/data/mapping/UniProt/{accession}/pathways?species=9606`. The reverse pathway section must use the same identifier namespaces. Proposed joins resolve a captured UniProt accession through the existing checked HGNC map. A reference lacking an exact join remains a source reference with a problem and no fabricated gene action. Several proteins mapping to one gene do not erase their source membership or modification context.

WikiPathways search and get are catalog resources in 0.9 and in the current public capture. A requested WP identifier must match a returned catalog row exactly. The legacy xref mapper expects Entrez references, but its current whole catalog is unproved because the body exceeded the capture bound. The separately captured GPML explicitly carries Entrez, Ensembl and HGNC namespaces; those can join through the existing exact HGNC maps only after that component is approved. No label resolves an identifier. Identifier namespaces remain explicit. A GPML `DataNode` might represent a metabolite, gene product, label or group; neither its text label nor its visual node count is a gene count. The GPML sample proves those public reference fields for WP382. Other records may omit them; missing or empty namespaces remain unresolved. The source description catalogs alone do not prove genes.

A pathway-to-drug relation requires an exact gene identity followed by a captured source's explicit target identifier. Reactome participants also expose explicit `ReferenceTherapeutic` references with IUPHAR ligand IDs, including 9977, 8943 and 10387. These are native therapeutic references, not proved identifiers for the current drug resolver. Keep their exact namespace and source URL; do not generate a drug get from their labels. No candidate source establishes a drug effect solely from pathway membership. The initial design exposes exact gene actions. It does not fan out across every gene, deduplicate drugs by name or label a drug as effective against a pathway. Existing drug identity and target sections remain the route from a selected drug to its exact gene context.

### Bundles

Prefer the existing Reactome online endpoint for exact cards and the existing UniProt-to-pathway join. Propose explicit `sync wikipathways` for the two captured metadata catalogs, each retained with its own hash and revision, with human filtering by the exact source species field. They are published snapshots, not proof of a complete universe of every curated or historical pathway. Keep current catalogs for search and get; add exact GPML member fetch only as a separately reviewed component if the PM keeps it. The failed xref catalog is not required for the initial metadata source. A data bundle requires an explicit `sync`, an atomic validated replacement, a revision-bound snapshot, no network on reads, preserved old data after a failed replacement, source disabling before local reads, and absent/stale/syncing outcomes. The implementation cannot silently turn an unsuccessful API query into a different endpoint or a bundle download.

A publisher directory's compressed file length does not measure decompressed bytes, record count or a build's memory use. No full bundle download is authorized merely to fill those blanks. Capture metadata first. A later bounded whole-bundle job measures full size and validates the complete file independently of code landing when its source design needs one.

## Delight questions

1. **Which human pathways does this exact gene belong to?** Start with the landed `get gene HGNC:1097 pathways`, then follow the exact returned Reactome stable ID. A proposed `search pathway --gene HGNC:1097` supplies a revision-bound full walk when the gene card's bounded list is insufficient.
2. **Which pathway did the search find?** A header contains its source, stable identifier, native title and source-stated species. It never adds an actionability label or a guessed synonym.
3. **Which genes can I inspect next?** A paged gene relation holds exact source references, optional resolved HGNC IDs and source-provided labels. Actions name resolved genes only.
4. **What happened inside this pathway?** Reactome's events section supplies source events and their native identifiers. WikiPathways does not advertise an events section without a captured public field.
5. **How do I compare sources?** Request each source explicitly. Two records retain their native identity and provenance even when their titles match. No synthetic merged pathway is created.
6. **How can I stop early?** A short header and optional whole summary answer identity and source description. Members and events remain behind explicit disclosure. Counts describe the specific retrieved relation rather than clinical significance.
7. **What is empty and what failed?** A successfully parsed, identity-checked empty source result is empty. The reviewed Reactome no-entry 404 shape is endpoint-specific empty. Other HTTP, timeout, malformed content and identity errors are failed source outcomes. A partial multi-source result preserves the successful leg and names the failed leg.
8. **What can I trust clinically?** Membership comes from a named source and release. An agent must obtain independent clinical evidence for therapy, response or actionability. The pathway record makes no such claim.
9. **What does a warm call cost?** A previously retained page reads its stored reply. A Reactome continuation requests only its next native page with `rows=--limit`; it does not preload the whole result. A cold call's source latency and bytes come from this capture, not from an invented performance estimate. No whole-source fanout occurs during an ordinary read.

## Proposed types and schema

The existing checked `ReactomeId` and `PathwayRef` live in `biomcp-types::response::gene::pathways`. Do not duplicate or move their source of truth implicitly. A reviewed migration or wrapper is required if the neutral entity needs them in `biodata`. Proposed `PathwayId` is a tagged Reactome stable ID or WikiPathways ID. Neutral records include `Pathway`, `PathwayRef`, `PathwayMember`, `PathwayEventRef` and namespaced source identifiers. Reuse the current `PathwayRef` contract; version any changed wire shape. A Reactome stable ID's optional version is a separate source field. A WikiPathways revision stays separate from its WP identity. No generic unchecked string replaces existing checked identifier types.

The neutral entity contains identity, native name, human species and source description. Consumer types own `PathwayHeader`, card, page and source-specific sections. Optional absent scalar fields are omitted. Requested source-supported empty sections carry the established empty state. Unsupported sections fail usage validation before I/O. An object with an unresolved source identifier is not dropped silently. The schema defines all closed source and section sets and versioned page continuations.

The service retains native search order for each named source. It follows the established multi-source round-robin order without score blending or title reranking. Equal native pathway keys may deduplicate inside one source; cross-source names never merge. A retained page stores its native offset, ordered admitted rows, native total, source outcomes, source revision, request identity and expiry. Reactome continuation requests the next native page through `Start row` with `rows=--limit`; it advances by native position even when exact admission removes rows. A stored page can be read again without reconstructing it through live I/O. WikiPathways catalog pages remain revision-bound local snapshots. A nonzero mixed-source offset requires its retained pool. Missing or mismatched retained state offers a restart; it never silently walks earlier native pages. Single-source offsets retain their native or catalog meaning.

## Proposed commands, headers and disclosure

These spellings are proposals for the PM, not shipped behavior.

| Command | Default answer | Deeper data and next steps |
| --- | --- | --- |
| `search pathway TEXT --source reactome` | At most five admitted source headers from one native five-row request, matched literal query, source outcome and source-reported native total | One get per header, then native-offset page; exact admission may produce a short page |
| `search pathway TEXT --source wikipathways` | Five human catalog headers from the captured public relation | Get per header, then revision-bound page |
| `search pathway --gene HGNC:1097 --source reactome` | Five pathway headers from exact gene-to-UniProt mappings | Get actions then page; several accessions retain source scope |
| `search pathway --top-level --source reactome` | Five source-stated human top-level pathway headers | Get actions then page |
| `get pathway R-HSA-5673001` | Exact identity, title, species, release, available sections and source terms | Section actions for captured source-supported relations |
| `get pathway WP382` | Checked WP identity and native human metadata | Only captured available sections |
| `get pathway ID summary` | Whole source summary with source scope and citations if present | No sentence clipping or extraction of clinical claims |
| `get pathway ID genes` | First five source membership references and totals | Exact gene pivots, then section continuation |
| `get pathway ID events` | First five native Reactome events and totals | Page before at most three other actions |
| `pathway genes ID --limit N --offset N --pool ID` | One bounded retained membership page | Exact gene pivots then continuation; no whole-pathway drug fanout |
| `pathway events ID --limit N --offset N --pool ID` | One bounded retained native event page | Only actual pathway-class event records get pathway actions |
| `sync wikipathways` | Hash, bytes, revision and human/total row counts for the two approved metadata catalogs | No catalog body in stdout; a failed replacement keeps the old snapshot |
| `get pathway ID all` | All supported sections, each with its own bounded initial page | Continuations retain source, revision and section |

`--source` is explicit for source-specific sections and defaults to Reactome for plain search. Repeated sources can use the established union mechanism only after both source legs are proved. `--limit` accepts the existing checked page-size type; `--offset`, `--pool` and `--fresh` follow existing snapshot contracts. The proposed genes/events continuation commands page the selected record's retained relation, following the existing record-parts pattern. Default limit is 5; propose a checked maximum of 50. Reactome text search sets `rows` to `--limit` and `Start row` to the requested native offset. Exact human/class checks apply to each page; rejected rows do not trigger a refill request. The next offset follows the native position and the total remains the native source total. No free-form URL or source endpoint enters the command tree. `--top-level` and text are mutually exclusive. `--gene`, text and top-level are mutually exclusive. No `--type` is needed when the entity is already pathway. Non-human pathway IDs are rejected before network I/O.

Next actions follow architecture 6.10: selectable headers receive get actions in source order, then page, then at most three others. Membership pages emit a pivot per distinct resolved gene, then page. Other responses have at most five actions. Missing bundles offer sync; expired snapshots offer restart; disabled sources do not offer a network action.

### Measured disclosure costs

<!-- disclosure-costs -->
The retained native-fact projections are lower bounds. They exclude envelope metadata and actions. The following measurements replace the proposed allocations with product output. Job j14975 measured the local CLI and MCP cases at `da2e537d`; the native WikiPathways measurements below remain the accepted archive observations without replay. Each case owns its fixed measured baseline and enforces the shared headroom rule. A CLI measurement includes its trailing newline; compact JSON excludes that newline.

| Command or section | Full CLI bytes | Compact JSON bytes | Comparable JSON bytes | Measured case owner |
| --- | ---: | ---: | ---: | --- |
| Reactome search, largest retained five-header page | 1,641 | 1,640 | 1,552 | `pathway/search-1.yaml` |
| Reactome default identity card, largest retained card | 759 | 758 | 670 | `pathway/card-R-HSA-8950505.yaml` |
| WikiPathways default card, WP382 with genes action | 587 | 586 | 495 | Native GPML archive and `wikipathways-budgets.yaml` |
| Whole Reactome summary, largest retained section | 10,756 | 10,755 | 10,667 | `pathway/card-R-HSA-201556.yaml` |
| WP382 whole catalog description | 714 | 713 | 622 | Native catalog archive |
| Reactome first five exact references as a genes section | 2,836 | 2,835 | 2,661 | `pathway-sections/genes.yaml` |
| Reactome first five event references as an events section | 2,298 | 2,297 | 2,209 | `pathway-sections/events.yaml` |
| Reactome all, card and bounded first section pages | 8,714 | 8,713 | 8,539 | `pathway-sections/all.yaml` |
| Reactome direct genes page | 2,395 | 2,394 | 2,220 | `pathway_sections/measurements` CLI and MCP phases |
| Reactome direct events page | 1,857 | 1,856 | 1,768 | `pathway_sections/measurements` CLI and MCP phases |
| Reactome direct genes details, complete physical contexts | 10,744 | 10,743 | 10,569 | `pathway-sections/details.yaml` |
| WikiPathways sync receipt | 692 | 691 | 600 | Native catalog archive |
| Reactome largest observed 50-reference genes page | 15,615 | 15,614 | Not separately attributed | `pathway_sections/walk`, offset 250 |
| Reactome terminal genes page | 11,017 | 11,016 | Not separately attributed | `pathway_sections/walk`, offset 300 |
| Reactome first 50-event page | 9,829 | 9,828 | Not separately attributed | `pathway_sections/walk`, offset 0 |
| Reactome terminal events page | 7,311 | 7,310 | Not separately attributed | `pathway_sections/walk`, offset 50 |

Comparable output removes only source `use`, `terms` and `terms_url`. The prior native projections (including search headers, bare identity fields, exact gene mappings and event references) remain lower-bound observations in the source evidence sections. The existing Reactome gene-mapping and top-level cases enforce their measured page budgets. The retained WikiPathways GPML table below separately measures both captured pathways and their genes sections, all responses and direct relation pages. Its full-envelope baselines live in `wikipathways-gpml-budgets.yaml`.

`tests/pathway-command-budgets.yaml` owns the measured full-envelope baselines for each named Rust command phase and exercised surface. Stable selectors distinguish commands; runtime pool tokens and temporary cache paths do not create separate budgets. Loops retain their largest observed page. YAML cases own their measured comparable `max_bytes`; a CLI newline is included in those ceilings. Transport and concurrent-process cases retain their existing CLI ownership; a compact JSON size alone does not claim an additional MCP execution. No provider call or native archive replay supplies these new attribution measurements.

The 0.9 default, section, helper, JSON and Markdown costs remain **unmeasured** under the containment ruling. Milestone 5 owns the contained comparison. No 0.9 saving or latency comparison is inferred from these 1.0 measurements.
<!-- /disclosure-costs -->

Bytes divided by four are a planning estimate of tokens and are labeled as such. They are not a tokenizer measurement. Whole observed raw source bodies are not proposed default outputs. Before adapters land, actual CLI and MCP output must be retained and measured at the released revision, with full output and the shared source-terms comparable formula reported separately. The product cases enforce measured bytes plus 15 percent for new budgets. An existing entity-help or MCP description cap increase requires a PM ruling. Whole-tree help and source enum allowances follow their standing measured rules.

## Complete 0.9 accounting

The inspected legacy source is `4050fdc7dbdf4d713d94cf02a39a147b863665be`. No legacy command runs against the network in this investigation. Verified network containment has not yet been established for a retained executable, so all 0.9 latency and output comparisons are **unmeasured**, rather than presumed equal to current fixtures. The source inventory below accounts for behavior without executing KEGG, g:Profiler, hidden trial fallback or drug fanout.

| 0.9 surface | Every option or section | Proposed outcome and reason |
| --- | --- | --- |
| `search pathway` | Positional query; `-q/--query` | Keep positional query. Retire query alias with `use: search pathway <text>`. |
| `search pathway` | `--type` | Drop redundant type filtering; exact accepted pathway class is enforced. Return fixed retirement guidance. |
| `search pathway` | `--top-level` | Keep native human Reactome top-level discovery, with explicit query conflict validation. |
| `search pathway` | `-l/--limit`, `--offset` | Keep bounded Reactome native pages with `rows=--limit`, native-offset continuation and exact admission that may shorten a page. WikiPathways uses revision-bound catalog pages. |
| `get pathway` | ID and default card | Keep Reactome and captured human WikiPathways identity. KEGG IDs return fixed retirement without a request. |
| `get pathway` | `genes` | Keep captured exact source identifier relations with `get pathway ID genes` and `pathway genes ID`; `details` exposes whole physical contexts. Remove participant-name regex extraction. |
| `get pathway` | `events` | Reactome only, preserve integer/object reference distinction. Unsupported source requests fail before I/O. |
| `get pathway` | `enrichment` | Drop under entity boundary ruling; fixed message points to external g:Profiler use without calling it. |
| `get pathway` | `all` | All supported kept sections, bounded individually; never invokes enrichment. |
| `pathway drugs` | ID; `-l/--limit`; `--offset` | Retire automatic whole-pathway gene fanout. Guide to inspecting the exact member genes, then selecting a drug identity and requesting its explicit source sections; reverse gene-to-drug search is unsupported. Accepted by the 2026-10-26 pathway ruling. |
| `pathway articles` | ID; `-l/--limit`; `--offset` | Retire implicit title-to-keyword search. Guide to explicit `search article <text>`; never claim article linkage from a matching title. Accepted by the 2026-10-26 pathway ruling. |
| `pathway trials` | ID; `-l/--limit`; `--offset`; `--source ctgov\|nci` | Retire pathway-name condition search and first-successful-gene fallback. Guide to explicit `search trial --gene <gene>`. Accepted by the 2026-10-26 pathway ruling. |
| `gene pathways` | Gene; `--limit`; `--offset` | Keep landed `get gene <input> pathways`; Full membership paging uses `search pathway --gene GENE --limit N --offset N`. |
| `list pathway` | No entity-specific options | Replace with shared `help search pathway` and `help get pathway`; fixed guidance, no network. |
| `search all` pathway section | Parent query and shared global paging/output flags; no pathway-specific flags | Account for the legacy nested pathway section separately from standalone search. Accounted without a new parent search; standalone pathway search retains source-scoped outcomes. Forbid an implicit enrichment or KEGG leg. |
| `enrich` | Comma-separated genes; `-l/--limit` | Drop whole command under Ian's entity boundary ruling. Fixed retirement message; no provider and no local calculation. |
| global output flags | JSON, Markdown, CLI/MCP transport | Preserve shared versioned envelope and identical request interpretation. Do not create a separate pathway MCP tool. |

### Leaf-command parity accounting

The leaf table owns pathway leaves. The article slice transfers `pathway articles` here; the gene slice retains `gene pathways` and maps its old paging options to the explicit pathway search. Section names, positional inputs, short aliases and global output flags stay outside option-slot counts. The complete accounting table above owns each of those surfaces, including `search all` without a new parent command.

<!-- parity-accounting -->
| 0.9 leaf command | Command outcome | Ported or renamed options | Dropped options | Deferred options | Undecided options | Ruling |
| --- | --- | --- | --- | --- | --- | --- |
| `search pathway` | Ported or renamed | `--top-level`, `--limit`, `--offset` | `--query`, `--type` | — | — | Use positional text or exact gene or native top-level discovery. The native Reactome page controls preserve source offsets; WikiPathways pages retain their catalog revision. |
| `get pathway` | Ported or renamed | — | — | — | — | Use exact human Reactome or WikiPathways IDs with summary, genes, events or all as supported by that source. Enrichment retires under the entity boundary ruling. |
| `pathway drugs` | Dropped by ruling | — | `--limit`, `--offset` | — | — | The 2026-10-26 ruling retires whole-pathway fanout. Inspect exact member genes, then select a drug identity and its explicit source sections; reverse gene-to-drug search is unsupported. |
| `pathway articles` | Dropped by ruling | — | `--limit`, `--offset` | — | — | The 2026-10-26 ruling retires implicit title matching; use explicit search article TEXT. |
| `pathway trials` | Dropped by ruling | — | `--limit`, `--offset`, `--source` | — | — | The 2026-10-26 ruling retires condition-name matching and first-gene fallback; use search trial --gene GENE. |
| `enrich` | Dropped by ruling | — | `--limit` | — | — | The entity boundary ruling excludes g:Profiler, enrichment calculation and execution of analysis tools. |
<!-- /parity-accounting -->

KEGG, g:Profiler, `enrich`, `enrichment`, `pathway drugs`, `pathway articles` and `pathway trials` remain dropped. The full GPML archive and bundle-wide gene index remain deferred; exact publisher-linked per-pathway GPML supplies the shipped membership command.

### Closed public issues

The GitHub closed-issue inventory contains 57 issues on the investigation date. A case-insensitive scan of every title and full body for `pathway`, `Reactome`, `WikiPathways`, `g:Profiler` and `enrich` finds [issue 78](https://github.com/genomoncology/biomcp/issues/78). Its enrichment request is dropped by the entity boundary ruling. Its Open Targets associations and other entities remain with their owning slices; its request does not authorize importing analysis tools. This inventory is separate from private development tickets.

The legacy private history also records useful failure classes: 0016 truthful source-supported sections, 0017 default gene walls and ranking, 0163/0169 WikiPathways HTML/404 sanitization, 0346 isolated WikiPathways tests, 0390 routine/live separation, 0422 volatile live top-N, 0443 wrong-entity guidance, 0669 deterministic ranking, 1063 scoped search-all results and 2282 recording fidelity. Keep truthful errors, isolated fixtures, deterministic retained evidence and exact identity guidance. Remove title heuristics and unsupported sections. Never pin live mutable source counts as timeless expectations.

## Outside-in case matrix

| Contract | Required cases | Case owners |
| --- | --- | --- |
| Plain search and source choice | Known-positive exact native row for each source; literal multiword query; each requested source only; retained native order; source conflict validation; empty query; actual captured no-match | `pathway/search-0.yaml` to `search-2.yaml`, `pathway/validation.yaml`, native catalog archive and `wikipathways.rs`. |
| Gene selector | Exact HGNC and UniProt join; several accessions; missing exact map; no symbol extraction; incompatible text/top-level selectors; disabled source before lookup | `pathway/gene.yaml`, `pathway_registry/admission.rs` and `pathway_failures.rs`. |
| Top-level | Native human positive and terminal page; non-human rejection; text conflict | `pathway/top-level.yaml`, `pathway/validation.yaml` and `pathway_failures.rs`. |
| Paging options | Reactome default, one and maximum limit use equal native `rows`; proven `Start row` controls native offset; exact admission may shorten a page without refill; next offset advances over rejected native rows; native total stays unchanged; invalid limits; offset zero and last page; no duplicates/gaps; pool mismatch; expired/missing pool restart; WikiPathways retains its snapshot after new sync | `pathway/native-page.yaml`, `pathway_commands.rs`, `pathway_sections.rs` and `wikipathways.rs`. |
| Get identity | Reactome and WP positive; wrong returned ID; malformed/non-human ID; unknown genuine ID; missing species; wrong species; wrong-entity fixed guidance | `pathway/cards.yaml`, every `pathway/card-R-HSA-*.yaml`, `pathway/failure-mismatch.yaml`, `pathway_failures.rs`, `wikipathways.rs` and native catalog archive. |
| Summary | Whole native text; absent summary distinct from source failure; citations remain native references | `pathway/summary.yaml`, summary steps in `pathway/card-R-HSA-*.yaml`, `pathway-sections/all.yaml` and native WP382 summary. |
| Genes | Exact namespaced references; duplicates and one-to-many protein/gene mappings; unresolved ID preserved without action; mixed physical-entity kinds; actual relation count; source-specific availability | `pathway-sections/genes.yaml`, `pathway-sections/details.yaml`, `pathway_sections.rs`, `wikipathways_genes.rs`, its `joins.rs` and retained native output contract. |
| Events | Native object plus integer references; no guessed stable IDs; absent public field; unsupported WikiPathways request before I/O | `pathway-sections/events.yaml`, `pathway-sections/synthetic-object-only.yaml`, `pathway-sections/validation.yaml` and `pathway_sections.rs`. |
| All and section paging | Supported sections only; each page bounded; source state per requested section; no hidden enrichment leg; genuine continuation arguments round-trip through CLI/MCP | `pathway-sections/all.yaml`, `pathway_sections.rs` and retained native WP382/WP100 all outputs. |
| Faults | Recorded 429, 500, timeout, invalid JSON, HTML success body, oversized body, foreign identity; no failure converted to empty | `pathway/failure-*.yaml`, `pathway-sections/failure-*.yaml`, `pathway_failures.rs`, its `oversized.rs` and `wikipathways_genes.rs`. |
| Terms and policy | Exact use/name/URL; disabled-before-cache/network; open/restricted groups; no credential discovery; source terms comparable helper removes all three metadata fields | `pathway_registry.rs`, its `admission.rs`, `wikipathways.rs`, `pathway_failures.rs` and shared `source_terms.rs`. |
| Retirement | Every legacy spelling/option above; fixed exact message; zero provider requests; no invalid parser fallback | `pathway_retirements.rs` covers every retired command, alias and option with a local zero-request listener. |
| Costs | Default, maximum, terminal and largest observed section; CLI newline and compact JSON bases explicit; full and comparable output; entity help, whole help, MCP description and executable examples | `pathway-command-budgets.yaml`, both native budget files, all pathway YAML ceilings, `pathway/help.yaml`, `gene/help.yaml`, `protocol.rs` and generated help example execution. |
Captured success cases carry their observation date, request and exact expected identifiers. Authored failures are labeled synthetic. A fixture projection preserves every union variant and fields that determine identity, absence, multiplicity and collection scope. Native bodies stay in the evidence archive. Tests use isolated configuration and deny network on replay.

## Structural risks

One source declaration owns terms, quota, deadlines and admission limits. Existing checked gene/pathway identities own joins. Generated schemas and help derive from the request types. A single next-action function owns actions. The inventory spans Rust cases and YAML. Tests preserve required OS environment while isolating user state. Exact source records and snapshots prevent live rereads from changing a page. Native capture precedes types, and every consumed field is tested against actual public-tier evidence. No new generic graph framework, pathway-equivalence engine or analysis backend is proposed.

## Ian's questions

No new decision is requested from Ian. Existing entity and source rulings apply. The accepted source and retirement rulings apply. Reactome text search follows the capture-limit ruling: native pages use the proven `Start row` and `rows=--limit`, exact admission may shorten each page, continuation advances from the native position and the total retains its source meaning. All 26 paper-derived question answers have clinical acceptance; they make no treatment recommendation. Any new cost, paid source, licensed content or external commitment returns through the coordinator and PM.

## WikiPathways product output measurements

The two retained native catalogs each contain 2,218 rows, including 1,131 rows with the exact species bytes `Homo sapiens`. The search catalog has 2,346,682 bytes and SHA-256 `5638b038a7cd19f6872c57931fc98d6a8cef98fd78619ca2151facbc7c8fa0df`. The get catalog has 1,224,217 bytes and SHA-256 `6c5fc71bd7f036261a47ccaa4a71f450e54e5b3fc0537e6b85a662a74e43040f`. These are observations of retained inputs. Neither counts nor revisions are timeless provider guarantees. Raw species bytes determine human admission before display text normalization; padded labels do not count as human. Display titles retain the shared `Text` outer-whitespace canonicalization.

The earlier catalog-only replay used twenty-seven native command cases covering sync, the 24 investigation search phrases including the no-match sentinel, and the WP382 default card and summary. Each command has identical CLI and MCP result content and passes the embedded response schema. Full native bodies, case inputs, output JSON and measurement receipts remain outside Git. The complete evidence archive has SHA-256 `7d960a51e5f12450f76283af99c61e7168d4201cc22ac1ca9892b9f65632ee99` and retains the earlier accepted case groups.

The catalog-only card observation below predates the genes action. Its retained baseline is now 587 CLI bytes and 586 compact JSON bytes, as the separate GPML replay below records. The earlier measurements remain archive observations.

| Native command | Full CLI bytes | Full MCP bytes | Comparable CLI bytes | Comparable MCP bytes | Estimated comparable MCP tokens |
| --- | --- | --- | --- | --- | --- |
| Sync receipt | 692 | 691 | 601 | 600 | 150 |
| Search, 24 phrases | 394–1,639 | 393–1,638 | 303–1,548 | 302–1,547 | 76–387 |
| WP382 default card | 519 | 518 | 428 | 427 | 107 |
| WP382 summary | 714 | 713 | 623 | 622 | 156 |

CLI bytes include the trailing newline; MCP bytes count compact JSON. Comparable bytes remove exactly source `use`, `terms` and `terms_url` with the shared helper. Estimated tokens are comparable MCP bytes divided by four and rounded up. These are output measurements, not tokenizer measurements or latency comparisons. The native forms retain their measured ceilings in the case budget files. The source catalog description remains whole and explicitly carries its source-truncation label.

`tests/wikipathways-budgets.yaml` records each case's full CLI bytes, comparable CLI bytes and estimated comparable MCP tokens in that order. The opt-in native runner enforces each measured value plus fifteen percent, rounded up. Full catalog evidence remains outside the ordinary routine suite; ordinary command cases exercise catalog replacement, retained paging, exact species admission and failure boundaries with isolated synthetic inputs. The 0.9 comparison remains unmeasured under the containment rule.

### Exact per-pathway GPML genes

The publisher's retained help page explicitly documents `https://www.wikipathways.org/wikipathways-assets/pathways/{WPID}/{WPID}.gpml`; the retained WP382 page links its exact GPML download. This documented locator admits only the requested checked WP identity. It does not permit alternate hosts, xref-catalog fallback or monthly archives. GPML uses one keyless GET with no retry or redirect. The adapter's `gpml_raw::MAX_BYTES`, `MAX_NODES` and `MAX_DEPTH` bound the whole XML document, including ignored content. The reviewed quick-xml decoder rejects DTD declarations and foreign namespaces on admitted DataNode/Xref elements. Parsing, identity checks and member validity precede cache admission, and malformed or truncated documents never become partial member lists.

The two native files are retained on Yellow in `data/biomcp-redux/0224-wikipathways-genes/native-1793004917879261000.tar.gz`, SHA-256 `eeb794004a09a28cd022a81dfe3c10a255dbbed367d9f3dd8b2be744a2e0ca4c`. WP382 is `WP382_r140476`, with 252 GeneProduct, six Metabolite and five Pathway DataNodes. Its two empty-namespace/empty-ID nodes are Pathway nodes, so they are excluded from gene membership and retained as source problems. WP100 is `WP100_r141191`, with 43 GeneProduct, 21 Metabolite and five Pathway DataNodes. These counts describe the retained source observations.

Only direct GeneProduct DataNodes become members. Their native GraphId, label, Xref Database and Xref ID remain distinct. Repeated xrefs remain separate data-node rows. An omitted namespace differs from a present empty namespace; neither resolves a gene or creates a pivot. The total kind `data_nodes` counts the retained GeneProduct nodes, while `physical_entities` counts all native DataNodes. Neither count claims distinct genes. Complete source-ordered membership is retained before selecting a page, and every page command retains the same snapshot and GPML revision.

GPML's complete Version has the exact requested WP identifier, `_r`, and a positive decimal revision. Its source-stated organism must equal `Homo sapiens` before text normalization. The relation header retains that complete native Version as its revision. Catalog revision dates remain separate: the native WP100 catalog date is 2025-11-07 while its GPML Last-Modified is 20251106160209, so date equality would reject the publisher's own retained record. A refresh creates a separate retained relation; an existing pool never changes or renews its source revision.

Entrez Gene and Ensembl Xrefs use the existing exact HGNC xref indexes and verify the returned identifier. Native HGNC numeric accessions use the exact HGNC ID index. The captured HGNC Xref `CACNG3` is a source-stated HGNC symbol and uses only the approved-symbol index with exact returned-symbol equality. No label, alias or inferred version stripping participates. Every unresolved member stays visible with its native reference and a problem. Navigation offers one pivot per distinct resolved gene in source order, then the retained page step. There is no drug, article or trial fanout.

The captured GPML response uses `application/octet-stream`. Its endpoint explicitly selects that exact media spelling through `application_octet_stream()`, followed by ordinary bounded XML decoding, request-bound identity checks and cache admission. Existing `octet_stream()` endpoints retain their separate `binary/octet-stream` permission and unchanged JSON checks.

Native CLI and MCP walks preserve every captured GraphId in source order and retain `WP382_r140476` and `WP100_r141191` independently of the catalog dates. The exact joins are Entrez 7189 → HGNC:12036, ENSG00000158711 → HGNC:3326 and the native HGNC symbol CACNG3 → HGNC:1407. The complete native output archive is `data/biomcp-redux/0224-wikipathways-genes/replay-1793008875251411000.tar.gz` on Yellow, SHA-256 `f9fa5eb0eb9b80910da3bfc26473b193811e35951fdad5cde8ddb7e950be4ef0`.

| Native command | Full CLI bytes | Full MCP bytes | Comparable JSON bytes | Estimated tokens |
| --- | ---: | ---: | ---: | ---: |
| WP382 card | 587 | 586 | 495 | 124 |
| WP382 genes section | 2501 | 2500 | 2323 | 581 |
| WP382 all | 2683 | 2682 | 2505 | 627 |
| WP382 direct genes page | 2160 | 2159 | 1982 | 496 |
| WP100 card | 595 | 594 | 503 | 126 |
| WP100 genes section | 2814 | 2813 | 2636 | 659 |
| WP100 all | 2996 | 2995 | 2818 | 705 |
| WP100 direct genes page | 2465 | 2464 | 2287 | 572 |

CLI and MCP have equal JSON content sizes above; CLI adds its trailing newline. Both genes sections retain their measured ceilings in the case budget file. `tests/wikipathways-gpml-budgets.yaml` owns the measured full-body, comparable-body and token baselines. Every command's checked ceiling is its matching baseline multiplied by 115 percent and rounded up. The retained-output contract checks all sixteen original surface results against these fixed ceilings without repeating source replay. The existing WikiPathways card budget also includes the newly available genes action.
