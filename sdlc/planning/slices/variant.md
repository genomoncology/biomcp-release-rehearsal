# Variant slice

This investigation defines the variant slice through steps 1 to 4 of [the slice method](../slices.md). The starting revision is `7a7ce2a0f09b5981353753e2f1c94bb39484b06c`. Product code is unchanged. The PM accepted the design with the [variant slice rulings](../../decisions/2026-10-24-variant-slice-rulings.md). Those rulings govern the command forms and ownership below. Design acceptance does not declare capabilities ported.

## Evidence and limits

The investigation uses public sources without credentials. The question bank has 20 paper-derived questions and three investigator controls. The questions paraphrase retrieval needs anchored to published passages; they are not quotations of questions asked by clinicians. Its concentration in EGFR, TP53 and CHEK2 measures the specified input and assertion risks, not overall cancer coverage or source quality. These are historical source assertions, not patient care recommendations.

A fresh clinical reviewer read the primary papers, supplied the accepted answers, and reviewed source discrepancies. No existing variant rulings file was named. The complete review, source passages and raw responses remain outside Git on the evidence store. Paper text and licensed source payloads are not copied into the repository. The tables below cite the primary papers and record short factual findings.

The probe uses one shared request cache per exact URL and body. Repeated gene-level questions reuse the same actual response rather than making duplicate live calls. A matrix cell names the response or the missing join prerequisite. A source that cannot accept the supplied form is not sent a fabricated coordinate. Documentation-only keyed sources have no live coverage claim. No complete source database was built or refreshed.

Initial exploratory NCBI searches incorrectly paced ClinVar and dbSNP separately. Seven HTTP 429 replies therefore do not establish a provider problem. A bounded refinement shared NCBI pacing and retried only those affected reads. A refinement script also stopped on a CAR alternate contig without a chromosome; the correction skipped the unjoinable contig and reused completed responses. Both failed attempts remain in the evidence. A deliberately malformed NCBI RefSNP beta path and an EFetch request without `is_variationid` are exploratory mistakes, excluded from endpoint conclusions.

## 1. Public questions and accepted answers

- **H2015**: [Afatinib in Non-Small Cell Lung Cancer Harboring Uncommon EGFR Mutations Pretreated With Reversible EGFR Inhibitors](https://academic.oup.com/oncolo/article/20/10/1167/6399667); 2015-09-09; Published article. Read: Actual publisher page; Introduction and Results mutation list, Figure 1.
- **L2017**: [Recommended guidelines for validation, quality control and reporting of TP53 variants in clinical practice](https://pmc.ncbi.nlm.nih.gov/articles/PMC7457206/); 2017-03-15; Published article in author manuscript. Read: Actual PMC page, Table 1, Figure 2 and Boxes B/C.
- **C2023**: [Management of individuals with germline pathogenic/likely pathogenic variants in CHEK2: An ACMG clinical practice resource](https://pmc.ncbi.nlm.nih.gov/articles/PMC10623578/); 2023-07-25; 2023 published practice resource. Read: Actual PMC page, Challenges in interpretation, Figure 1 case 1 and Table 1.
- **V2022**: [Standards for the Classification of Pathogenicity of Somatic Variants in Cancer (Oncogenicity): Joint Recommendations of ClinGen, CGC, and VICC](https://pmc.ncbi.nlm.nih.gov/articles/PMC9081216/); 2022-01-29; 2022 published consensus. Read: Actual PMC title, metadata and abstract; later fulltext retrieval timed out.

The clinical reviewer excluded a bench-focused BRAF paper and used the clinical reporting guideline instead. The controls assert retrieval behavior only. A source-local zero is not an absent clinical biomarker. A failed source supplies no classification.

| Case | Input supplied by the paper or control | Question and accepted answer | Source passage |
| --- | --- | --- | --- |
| Q01 | `L858R`; gene `EGFR` | Which published EGFR variant observation matches this protein label? Common exon-21 variant in the paper. | H2015: Introduction first paragraph; Confirmed. |
| Q02 | `T790M`; gene `EGFR` | Which published EGFR variant observation matches this protein label? Thirty reported occurrences; retain compound-mutation context. | H2015: Results, mutation-distribution paragraph and Figure 1; Confirmed. |
| Q03 | `G719A`; gene `EGFR` | Which published EGFR variant observation matches this protein label? Reported separately and with E709A. | H2015: Results, mutation-distribution paragraph and Figure 1; Confirmed. |
| Q04 | `G719C`; gene `EGFR` | Which published EGFR variant observation matches this protein label? Reported separately and with E709A or L861Q. | H2015: Results, mutation-distribution paragraph and Figure 1; Confirmed. |
| Q05 | `G719S`; gene `EGFR` | Which published EGFR variant observation matches this protein label? Two reported cases. | H2015: Results, mutation-distribution paragraph and Figure 1; Confirmed. |
| Q06 | `L861Q`; gene `EGFR` | Which published EGFR variant observation matches this protein label? Reported alone and in compound mutations. | H2015: Results, mutation-distribution paragraph and Figure 1; Confirmed. |
| Q07 | `E709K`; gene `EGFR` | Which published EGFR variant observation matches this protein label? Reported with L858R, not as an isolated observation. | H2015: Results, mutation-distribution paragraph and Figure 1; Confirmed. |
| Q08 | `S768I`; gene `EGFR` | Which published EGFR variant observation matches this protein label? Reported with V769L. | H2015: Results, mutation-distribution paragraph and Figure 1; Confirmed. |
| Q09 | `NM_004333.4:c.1799T>A`; gene `BRAF` | Which BRAF protein change accompanies this transcript-qualified variant in the clinical reporting paper? The Introduction explicitly pairs this change with p.Val600Glu. | L2017: Introduction paragraph beginning For several genes, such as BRAF; Confirmed. |
| Q10 | `LRG_321p1:p.Arg273His`; gene `TP53` | Which reference-qualified protein example does the clinical reporting guideline give? Box B supplies this protein example alongside LRG_321t1:c.818G>A. | L2017: Box B items 4 and 6; Confirmed. |
| Q11 | `NM_001126114.2:c.1015G>C`; gene `TP53` | Which isoform-specific protein consequence accompanies this coding entry? Table 1 pairs it with p.(Glu339Gln). | L2017: Table 1, head and neck squamous cell carcinoma row; transcript-column header; Inconsistency. |
| Q12 | `NC_000017.10:g.7576633A>G`; gene `TP53` | Which transcript-specific coding consequence is listed for this genomic variant? Table 1 lists NM_001126114.2:c.1018A>G and p.(Asn340Asp). | L2017: Table 1, hepatocellular carcinoma row; Inconsistency. |
| Q13 | `NM_001126113.2:c.1006T>G`; gene `TP53` | Which isoform-specific protein label does this cancer observation carry? Table 1 pairs it with p.(Leu336Val). | L2017: Table 1, B-Chronic Lymphocytic Leukemia row; Confirmed. |
| Q14 | `NM_001126113.2:c.1034C>T`; gene `TP53` | Which alternative-isoform protein change accompanies this observation? Table 1 pairs it with p.(Ser345Leu). | L2017: Table 1, melanoma, neuroblastoma and skin BCC rows; Confirmed. |
| Q15 | `rs78378222`; gene `TP53` | What cancer-relevant location does the guideline assign to this rsID? Figure 2 places it in the polyadenylation signal. | L2017: Figure 2 caption; Confirmed. |
| Q16 | `NM_007194.4:c.470T>C`; gene `CHEK2` | Which germline cancer-predisposition assertion does the practice resource attach to this variant? The paper reports discordant germline labels and low penetrance. | C2023: Challenges in interpretation; Figure 1 case 1; Confirmed. |
| Q17 | `NM_007194.4:c.1283C>T`; gene `CHEK2` | Which germline cancer-predisposition assertion does the practice resource attach to this variant? Table 1 identifies a lower-risk missense variant. | C2023: Table 1; Confirmed. |
| Q18 | `Arg117Gly`; gene `CHEK2` | Which germline cancer-predisposition assertion does the practice resource attach to this variant? The paper separates its breast-cancer association from lower-risk missense variants. | C2023: Missense variants; Table 1; Confirmed. |
| Q19 | `NM_007194.4:c.433C>T`; gene `CHEK2` | Which germline cancer-predisposition assertion does the practice resource attach to this variant? Listed as a germline missense example. | C2023: Challenges in interpretation, paragraph on ClinVar; Confirmed. |
| Q20 | `NM_007194.4:c.1100del`; gene `CHEK2` | Which germline cancer-predisposition assertion does the practice resource attach to this variant? Used as a truncating germline comparator. | C2023: CHEK2 variant spectrum; Challenges in interpretation; Confirmed. |
| Q21 | `V600E` | Can a protein-only label without a gene uniquely identify a genomic allele? No unique allele is established by this input; preserve ambiguity. | L2017: Box B item 8; Box C transcript and protein-isoform specification; Likely. |
| Q22 | `NM_004333.4:c.1799T>A`; gene `BRAF` | When an investigator forces a valid lookup to return zero rows, what is established? Only source-local absence under the recorded request; no clinical negative follows. | L2017: Introduction BRAF paragraph supplies the positive identity anchor; Open question. |
| Q23 | `NM_007194.4:c.470T>C`; gene `CHEK2` | When the source fails, can retrieval replace disagreement with one classification? Preserve failure and unresolved evidence; do not fabricate a classification. | C2023: Challenges in interpretation; Figure 1 case 1 provides real disagreement anchor; Open question. |

The TP53 paper's Table 1 footnote c explicitly labels NC_000017.10 coordinates GRCh37 (hg19). CAR rejects two literal genomic alleles from the paper with `IncorrectReferenceAllele`: the paper prints G at 7576636 while CAR reports C, and the paper prints A at 7576633 while CAR reports T. The transcript input NM_001126114.2:c.1015G>C resolves. The clinical review labels this an inconsistency. Preserve the attributed paper spelling and the resolver's rejection separately. Never complement, repair or assert equivalence locally.

The CHEK2 practice resource documents differing germline laboratory labels for Ile157Thr. It does not prove a same-allele germline-versus-somatic classification contradiction. That comparison remains unverified. The clinical review requires framework, condition, source, version and context to remain separate.

## 2. Direct source findings

The evidence matrix records every question against every ROADMAP source, including unsupported input and missing-identifier cells. A successful HTTP response does not by itself establish coverage. GraphQL errors, zero results, partial pages and failed identity checks remain distinct. Search results are candidates until the source provides an exact checked identity. No local clinical ranking or interpretation is proposed.

| Source | Observed public behavior and returned joins | Proposed disposition and disclosure | Terms and tier |
| --- | --- | --- | --- |
| MyVariant metadata timestamps | The [official BioThings metadata service example](https://github.com/biothings/biothings.api/blob/master/biothings/web/services/metadata.py#L20-L26) gives a `build_date` with six fractional digits and no offset. The [current SDK builder](https://github.com/biothings/biothings.api/blob/master/biothings/hub/databuild/builder.py#L579-L585) writes local aware `datetime.fromtimestamp(self.t0).astimezone().isoformat()` text. Neither establishes UTC for older unsuffixed MyVariant timestamps. | Keep native fractional precision and explicit offsets. Preserve missing timezone information when the source supplies none. | These are public source-code facts about timestamp spelling. They establish no additional rights over component data. |
| MyVariant.info | Bare `EGFR L858R` and explicit versioned transcript HGVS can return zero although the historical command finds a record. rs113488022 returns three genomic allele IDs. A source-specific field query must be explicit and echoed. CAR-supplied GRCh37 single-base alleles support direct annotation retrieval. Returned annotation fragments and source licenses differ by record. | Keep as the named discovery and annotation source. Default search uses declared filters, not free-text inference. Retain every rsID candidate; never take the first. Compact annotations sit on the card; complete predictors and historical COSMIC sit behind named sections. | [API query documentation](https://docs.myvariant.info/en/latest/doc/variant_query_service.html), [assembly FAQ](https://www.myvariant.info/faq), [data sources](https://docs.myvariant.info/en/latest/doc/data_source.html). Aggregation grants no blanket license over component fields. COSMIC is only the historical copy MyVariant carries; no direct COSMIC calls. |
| ClinGen Allele Registry | GET `/allele?hgvs=` returns CAid, source-given GRCh37/38 alleles, transcript forms, rsIDs and ClinVar Variation IDs for supported reference-qualified input. It rejects paper/reference mismatches. The LRG protein example produced a provider error; no fabricated fallback. | Keep as explicit reference-qualified resolution. Preserve input and all source-qualified candidates. GET only, no registration or writes. Protein shorthand without a reference remains search input. | [Registry API](https://reg.clinicalgenome.org/redmine/attachments/download/262/AlleleRegistry_0.8.3_api.pdf), [service paper](https://pmc.ncbi.nlm.nih.gov/articles/PMC6519371/). Public read access is established; blanket output redistribution rights remain unverified, so keep raw evidence external. |
| ClinVar EFetch | Broad Entrez text search can return many unrelated records; an ID from exact CAR identity gives a direct `rettype=vcv&is_variationid=true` XML request. VCV, RCV and SCV context must remain separate. | Keep direct evidence section using checked Variation IDs. No text-search first hit becomes identity. Preserve germline, somatic clinical-impact and oncogenicity assertion domains where captured; fields absent in a record stay optional. | [ClinVar use](https://www.ncbi.nlm.nih.gov/clinvar/docs/maintenance_use/), [NCBI policy](https://www.ncbi.nlm.nih.gov/home/about/policies/). Optional key only raises quota. Submission provenance and possible contributor rights remain. |
| dbSNP | Exact RefSNP endpoint is `/variation/v0/refsnp/<number>`, not the exploratory beta path. rsIDs may describe more than one alternate allele. The direct record supplies placements and assembly metadata. | Keep exact rsID and coordinate joins. Preserve allele-specific placements; a matching rsID alone never merges distinct alternate alleles. Do not invent a genome build. | [NCBI variation API](https://api.ncbi.nlm.nih.gov/variation/v0/), [NCBI policy](https://www.ncbi.nlm.nih.gov/home/about/policies/). Keyless public data; original submission rights apply. |
| gnomAD | CAR's explicit GRCh38 allele supports `gnomad_r4` GraphQL lookup. The documented BRAF control returns exome ac 2/an 1460618 and genome null. Empty genome is not zero genomes. Population counts and filters are public. | Keep population section with dataset, assembly, ac/an, ancestry counts and filters. Detailed ancestry table is a deeper page. No population frequency becomes a classification. | [Policies](https://gnomad.broadinstitute.org/policies). Policy page is JavaScript-only in this reader; independent live redistribution license verification remains open. Preserve source attribution and do not redistribute captures. |
| CIViC | Anonymous GraphQL returns accepted evidence and assertion totals, IDs, disease, evidence type/level, significance and molecular-profile names. BRAF V600E name search also returns a compound OR profile. Name matching is not exact allele matching. | Keep explicit civic section. Preserve molecular-profile membership and logical context. Source record IDs and exact variant membership must be joined before attaching evidence to an allele; broad name results remain candidates. | [API](https://docs.civicdb.org/en/latest/api.html), [CC0 licensing](https://civicdb.org/releases/licensing). Anonymous API quota is published; a free optional key changes access rate and curation permissions, not data rights. No curation actions. |
| cBioPortal | Public gene and selected study mutation pages are available. Gene-wide rows are not an exact allele recurrence answer. A fixed hidden study would make an unexplained clinical denominator. | Keep only explicit study/profile/sample-set recurrence. Require user-selected cohort, exact source allele join and study denominator. No implicit default cohort and no sample records in the default card. | [API](https://docs.cbioportal.org/web-api-and-clients/), [FAQ](https://docs.cbioportal.org/user-guide/faq/). ODbL default with attribution; individual studies can impose commercial restrictions. Study rights must be shown. |
| Cancer Hotspots | Gene endpoint returns residue, transcript, position tumor count and alternate-amino-acid counts. Four reused gene replies cover this corpus; positions are not unique genomic alleles. | Keep named hotspots section with source transcript and residue/alternate counts. Do not call residue recurrence clinical actionability or allele-specific frequency. | [Publisher](https://www.cancerhotspots.org/). Public reachability established; redistribution terms not verified from rendered page. External captures only until terms are checked. |
| GWAS Catalog | Current v2 rsID associations endpoint returns a valid zero for rs113488022. V1 is retired. Dataset uses GRCh38 and dbSNP build 156 per current documentation. | Keep named gwas section for exact supplied or resolved rsIDs. No result is absence from this catalog, not benignity. Preserve study, trait, effect allele and effect estimates without interpretation. | [v2 API](https://www.ebi.ac.uk/gwas/docs/programmatic-access/rest-api/), [migration](https://www.ebi.ac.uk/gwas/docs/news/rest-api-v2-migration-guide/), [build FAQ](https://www.ebi.ac.uk/gwas/docs/faq/). Catalog terms and dataset-specific summary-statistics rights differ; no summary-statistics download. |
| ClinGen evidence repository | Exact CAid request returns versioned assertions or provider `No Records Found`. TP53 CA000434 returns PCER000001171, document 1.1.0, TP53 VCEP and condition. A missing assertion is not a missing allele. | Keep evidence section by CAid, with assertion/version drill-down. No spontaneous gene-wide scan for an absent CAid. Preserve criteria met/unmet availability without creating classifications. | [Repository](https://erepo.clinicalgenome.org/), [ClinGen sharing](https://clinicalgenome.org/about/terms-of-use/). Exact service output redistribution terms need verification; raw evidence stays external. |
| UniProt | Reviewed human gene search returns source protein accessions, features and variants. Protein context does not resolve an arbitrary genomic allele; multiple isoforms cannot be silently collapsed. | Defer variant-specific structure command. Existing gene protein section remains reachable. No harvested accepted answer requires PDB or AlphaFold joins. Source accessions remain links. | [UniProt license](https://www.uniprot.org/api-documentation/support-data): CC BY 4.0 for copyrightable database content. |
| InterPro | Exact returned UniProt accession gives paged domain entries. Domain overlap alone supplies no clinical assertion. | Defer variant-specific domains with the structure command. Existing protein context is sufficient for this corpus. No local transcript or residue guessing. | [InterPro](https://www.ebi.ac.uk/interpro/about/). Terms page could not be rendered; no broader reuse claim. |
| Mutalyzer | Explicit source-qualified coding and protein inputs produce normalized descriptions or explicit invalid-input messages. The two genomic paper mismatches reject; no local repair. | Keep explicit normalize service option. Preserve corrected/normalized descriptions, original input, warnings and source-equivalence output. Do not choose a transcript. | [Official documentation](https://mutalyzer.readthedocs.io/en/latest/usage.html). Public API output verified; code/article licenses do not establish a blanket service-data license. |
| VariantValidator | Explicit transcript inputs produce source HGVS mappings, warnings and version metadata. Protein-only LRG gives a validation warning. HTTP 200 can therefore still mean rejected input. | Keep explicit normalize service option. No fallback query alteration, no classification, no automatic canonical transcript selection. The caller supplies assembly. | [Publisher](https://variantvalidator.org/), [source paper](https://doi.org/10.1002/humu.23348). Preserve original warnings and version. Service redistribution terms remain unverified. |
| OncoKB | The initial investigation used documentation for protein and genomic annotation, explicit assembly and tumor context. Later approved demo capture supplies source proof; the product never calls demo. | Keep keyed named section. Without the user's configured key: skipped with setup hint, zero source requests. Do not infer clinical actionability. | [API](https://api.oncokb.org/oncokb-website/api), [technical FAQ](https://faq.oncokb.org/technical). User license and key required by the accepted decision. Public documentation is not executed-tier proof. |
| AlphaGenome | Publisher describes a keyed API for noncommercial use. No live request, key discovery or prediction was attempted. | Defer. None of the accepted retrieval answers needs generated regulatory predictions; this slice does not interpret them. Reconsider only for a named cancer-care question and applicable user license. | [Publisher announcement](https://deepmind.google/blog/alphagenome-ai-for-better-understanding-the-genome/). Keyed and noncommercial; documentation only. |

### Coverage interpretation

Sending literal paper wording to every search engine is a diagnostic, not a common query language. ClinVar/dbSNP Entrez searches can report broad gene matches; MyVariant bare protein text can return zero. These results do not justify local rewriting. The proposed `--gene` and `--protein` options declare their meaning and construct one fixed, echoed source query. Reference-qualified HGVS goes unchanged to the chosen resolver. An incomplete coordinate without reference and alternate alleles remains a regional search, never an exact variant get.

For each question, the matrix records CAR/Mutalyzer/VariantValidator as unsupported when no explicit reference is supplied, gnomAD as unjoinable without an unambiguous source-supplied GRCh38 allele, ERepo as unjoinable without CAid, and GWAS as unjoinable without rsID. Those are identified gaps, not fabricated requests or passed coverage. Further source calls require the caller to choose the missing candidate or a source to return the missing identity.

## 3. What would delight the agent

1. **First question.** “What does this source call the reported variant?” `get variant NM_004333.6:c.1799T>A` sends unchanged reference-qualified notation to CAR by default. `search variant --gene EGFR --protein L858R` returns candidates for protein shorthand. No guess hides between them.
2. **Smallest header.** Echoed input/form/build, resolved or ambiguous state, exact source record IDs, source-given gene/transcript/protein spellings, one selected source-qualified coordinate, and concise source assertion labels with condition, framework and date. The header never substitutes a computed classification.
3. **Next steps.** `next` names candidate selection, ClinVar assertions, population, CIViC evidence, hotspots, cohort recurrence, GWAS, ERepo, keyed OncoKB and normalization. It includes the exact source IDs needed by each command.
4. **Exact identifiers.** Preserve CAid, ClinVar Variation/VCV/RCV/SCV, rsID plus allele, reference accession with version, HGVS source text, gene ID and provider coordinate/build. rs113488022's three alternate alleles demonstrate why a bare rsID cannot be the sole variant key.
5. **Disagreements.** Print attributed claims separately. A paper/reference mismatch remains a checked resolver rejection alongside the paper spelling. Assertions retain framework, condition and review level. No vote or cross-framework collapse is allowed.
6. **Empty and failure.** Separate invalid input, ambiguous candidates, valid source-local absence, unavailable source, disabled source, missing key, unsupported input and missing join identity. A successful HTTP 200 with a validation warning is not a resolved variant.
7. **Cold and warm speed.** Reuse the shared transport and its existing cache. The measurements below identify endpoint calls and source-level latency. “Warm” historical runs are immediately repeated calls after a no-cache cold run; they do not prove a prepopulated HTTP cache. No promise of a warm speedup follows.
8. **Historical excess.** The historical card carries `_meta`, repeated commands, unrequested section outcomes and source predictors beside compact identity. These belong in shared envelope metadata or deeper sections. Historical successes for protein shorthand must be preserved without restoring free-text heuristics.
9. **Stop early.** An exact source-qualified identity and attributed summary let the agent stop. A candidate set or source disagreement tells it exactly why it cannot stop. Bulk evidence is fetched only when selected.
10. **Trust.** Every assertion includes source record link, condition, framework, release or retrieval time, review status and source-stated build. A missing version/build is explicit, never borrowed from the query.

## 4. Types, schema and request boundaries

These are reviewed design types, not implemented contracts. Reuse the existing envelope, checked strings, IDs, `Source`, section states, provenance, paging, source policy and generated schema pattern. `wire_enum!` remains the single source of enum spelling. No generic variant interpretation framework is introduced.

- `VariantInput` is a closed tagged request: source record ID; checked reference-qualified HGVS text with CAR as the default resolver; declared gene plus checked protein text for search; or genomic interval/allele plus explicit assembly. The edge rejects empty/control-containing or oversized text. The HGVS text remains opaque. A source record ID type checks its published identifier grammar, not molecular meaning. Get distinguishes rsIDs, CAids, ClinVar Variation IDs and reference-qualified HGVS by those published grammars. Chromosome HGVS requires `--build`. Gene/protein shorthand is refused with a usage error naming its declared search command.
- `VariantIdentity` contains one source record key, source-returned aliases and source-qualified representations. `VariantRepresentation` pairs original notation with reference accession/version and optional provider-stated assembly. Coordinates use the existing explicit build and counting types. CAR zero-based start/end and HGVS one-based text are never mixed under an unqualified integer.
- `VariantResolution` is resolved, candidates, absent, rejected or unavailable with the responsible source. Candidates have bounded pages, source identity and exact selection actions. No most-likely candidate is selected.
- `VariantHeader` carries input, resolution, exact key, source gene/transcript/protein text, selected coordinate and concise assertions. Fields absent from the source are optional. Reference/build uncertainty is explicit.
- `VariantAssertion` carries source assertion ID/version, framework, condition, original classification text, review/status, submitter or expert panel, evaluation date and links. Preserve source labels. Use closed enums only where the source documents a closed vocabulary; do not invent a common ordered severity scale.
- `VariantPopulation` carries source dataset/release/build, source allele, sequencing subset, ac/an, reported frequency, ancestry groups and filters. A missing genome/exome dataset differs from zero observations. Do not fill missing frequency by silently dividing counts; expose what the provider supplies.
- `VariantRecurrence` carries dataset/study/sample-set, source transcript or genomic allele, matching granularity, numerator, denominator if supplied and query bounds. Residue-level hotspot counts never masquerade as exact allele recurrence.
- `VariantNormalization` carries unchanged input, chosen service, normalized/corrected source text, source mappings, warnings, version and outcome. No full HGVS, VCF, liftover or transcript-selection implementation belongs here.

The existing article reader's small protein-change parser stays private unless a reviewed slice requirement justifies sharing it. This design needs checked source text first. Supporting notation forms that sources cannot resolve is an explicit product gap, not permission to build the other team's notation parser. Any future typed amino-acid parse uses a published code table, one implementation and an explicitly declared option; it never scans free text.

Identity checks compare exact source-qualified records and returned reference/allele identities. They do not compare only a gene symbol, source score or protein label. Joins use provider IDs. The transport owns TLS, retries, deadlines, quotas, source restrictions and cache admission. Adapters own raw schemas and source checks. Query owns source composition and disclosure. No adapter builds a client or local ranking rule.

## 5. Reviewed command tree and disclosure

| Command | Default output | Deeper action |
| --- | --- | --- |
| `search variant --gene G --protein TEXT [--source myvariant] [--limit N --offset N]` | Candidate header and source-native ordered bounded page. Echo exact query and assembly policy. | Select source ID for get. No local protein guess or silent first hit. |
| `search variant --query TEXT --source myvariant` | Explicit advanced source query and native candidates. | Typed source-supported filters, only after known-positive public-tier cases. Never claim source query syntax portable. |
| `search variant --build B --region REGION --source myvariant` | Region search with echoed assembly/counting and bounded results. | Exact selected allele get. Missing build is usage error. |
| `get variant INPUT [--resolver mutalyzer\|variantvalidator] [--build B] [SECTIONS]` | Published-grammar rsID, CAid, ClinVar Variation ID or reference-qualified HGVS; CAR resolves HGVS by default. Every alternate-allele candidate returns with selection steps. Gene/protein shorthand produces a usage error naming search. | `clinvar`, `population`, `civic`, `hotspots`, `cbioportal`, `gwas`, `erepo`, `oncokb`, `predict`, `conservation`, `cosmic`; `all` remains bounded and obeys source policy. |
| `variant normalize --source car\|mutalyzer\|variantvalidator --hgvs TEXT [--build B]` | Source's result and warnings. | Source-returned representations, paged when needed. `--source all` returns independent labeled results; no consensus. |
| `variant assertions ID --source clinvar\|erepo [--assertion ID --version V]` | One page of source assertions, totals and exact identities. | Selected submission/assertion/version and original evidence details. |
| `variant recurrence ID --source cbioportal --study ID --profile ID --sample-set ID` | Exact source match and cohort bounds; source counts. | More rows by bounded paging, without downloading whole cohorts. |
| `variant population ID --detail [--limit N --offset N]` | Deeper ancestry groups, counts and source filters. | Page or source link. |
| `variant cspec` | Deferred until a harvested question needs it. | No new command or document selection is approved by this investigation. |

All routes use the same request types and one MCP tool argument array. Section names and action variants come from generated schemas. CLI validation, MCP and Markdown views consume the same response. A multi-input get may reuse the established bounded batching design; a separate custom variant batch framework is not justified.

The card defaults to identity and MyVariant’s compact, source-attributed facts. It does not fan out to every source. CAR resolves reference-qualified HGVS by default. `--resolver mutalyzer` and `--resolver variantvalidator` select the other services explicitly. Direct source IDs avoid an unnecessary resolver. MyVariant is the compact annotation source where a checked join exists. Named sections fetch only their named sources. The default does not call cohort data, protein structure, GWAS, OncoKB or predictors. Source skips occur before cache and replay. A source's unsupported field never appears as a meaningful empty section.

### Structural risks

One identity representation owns build and counting rules. One request definition owns CLI/MCP spelling and schema. There is no second source registry, hidden normalization cache or hand-maintained alias list. Cases run through the built executable with isolated config/home, record/replay and explicit source controls. Only the chosen source identities are shared across calls. Outside-in cases verify requested identity, provider outcome, output budgets and source request counts. Proof retains original captures and a small generated manifest, not transcribed whole replies or duplicated status files.

## 6. Historical surface accounting

The released baseline comes from [the 0.9.3 inventory](../parity-0.9.3.json). Global flags, aliases, positional section values and options on non-leaf parents are outside that denominator. The authoritative generator is [scripts/parity.py](../../../scripts/parity.py); it remains unchanged. The [accepted rulings](../../decisions/2026-10-24-variant-slice-rulings.md) define the target behavior. Ledger outcomes mark implemented command behavior separately from the live source-proof boundaries below. The 2026-10-26 PM ruling in ticket 0275 defers unproved filter mappings under variant ruling 7. Advanced filters require known-positive source cases. Design approval does not increment shipped parity.

The leaf table below owns the variant commands and the transferred CSpec document leaf. The section table that follows gives canonical routes, retirement guidance and source-proof boundaries. `variant trials` and `search gwas` belong to the trial and GWAS slices. `variant articles` belongs to article accounting. Variant accounting owns `gene cspec document`; its parent flags `--version`, `--capture-id`, `--files`, `--limit` and `--offset` remain outside the leaf denominator. Generic `batch variant` and global `--json`, `--no-cache` and `--disable-sources` belong to shared command accounting. No capability is counted twice.

<!-- parity-accounting -->
| 0.9 leaf command | Command outcome | Ported or renamed options | Dropped options | Deferred options | Undecided options | Ruling |
| --- | --- | --- | --- | --- | --- | --- |
| `search variant` | Ported or renamed | `--gene`, `--hgvsp`, `--review-status`, `--condition`, `--has`, `--limit`, `--offset` | — | `--significance`, `--missing`, `--max-frequency`, `--min-cadd`, `--consequence`, `--population`, `--revel-min`, `--gerp-min`, `--tumor-site`, `--impact`, `--lof`, `--therapy` | — | Use declared gene plus --protein, explicit region/build, or unchanged --query with --source myvariant. --hgvsp names --protein in its retirement message. Only the three captured ClinVar filters are offered; the other mappings are deferred under variant ruling 7 and the 2026-10-26 PM ruling in ticket 0275. Native candidate pages retain selectors and source order. |
| `get variant` | Ported or renamed | `--assembly` | — | — | — | Use published-grammar rsID, CAid, ClinVar Variation ID or reference-qualified HGVS. --assembly names --build; chromosome notation needs it. CAR is the default HGVS resolver. Shorthand names search variant --gene BRAF --protein V600E. Every candidate stays available. |
| `variant structure` | Deferred by ruling | — | — | — | — | Deferred by ruling 6. Its retirement message names get gene <SYMBOL> protein; no structure or clinical equivalence is inferred. |
| `variant oncokb` | Ported or renamed | — | — | — | — | Use get variant <ID> oncokb with explicit --build and --tumor-type. Missing keys skip before source access; synthetic keyed cases cover annotation and credential containment. The accepted source proof uses an approved demo capture; the product requires the caller's own key and never calls demo. |
| `variant erepo` | Ported or renamed | `--input`, `--limit`, `--offset`, `--detail`, `--assertion`, `--version` | `--gene` | — | — | Use variant assertions <CAid> --source erepo; the input becomes positional. --assertion <PCERid> with optional --version selects details. Limit and offset page native assertions. The old leaf names this route. The PM dropped legacy --gene under the CAid-only scope. No gene-wide scan replaces missing allele assertions. |
| `variant normalize` | Ported or renamed | `--input` | — | — | — | Use variant normalize --source car, mutalyzer, variantvalidator or all with --hgvs TEXT. The old positional service and --input name this form. VariantValidator and all require --build. Results and mappings remain independent; there is no local repair or consensus. |
| `gene cspec document` | Deferred by ruling | — | — | — | — | Variant ruling 8 defers variant cspec until a harvested question needs it. The 2026-10-26 PM ruling in ticket 0275 transfers this released document leaf from gene accounting. |
<!-- /parity-accounting -->

The inventory and generated ledger own the leaf and option totals. Trial and GWAS slices own `variant trials` and `search gwas`, including their three and five option slots. The generated ledger supplies counts; do not copy a new total into ROADMAP by hand. `variant articles` remains the article accounting owner's responsibility. Deferred advanced filters retain their native positive-proof requirement before implementation.

### Implemented section answers and executable follow-ups

The default card holds source identity and compact MyVariant facts. Named sections call their named source. Source-local empty results describe only that request. Section rows use `--section-limit` and `--section-offset`; identity candidates use `--limit` and `--offset`. Returned next actions carry argument arrays with the original identity, selectors and policy settings. Run those arrays as CLI arguments or the MCP tool's `args`; never reconstruct commands from publisher prose.

| 0.9 section or helper | 1.0 answer and migration | Proof boundary |
| --- | --- | --- |
| `clinvar` | `get variant clinvar:13961 clinvar`; `variant assertions clinvar:13961 --source clinvar` selects native VCV, RCV or SCV records and versions. | Germline, somatic clinical impact and oncogenicity keep separate source domains and labels. Unmapped claims remain native. |
| `erepo` | `get variant CA000434 erepo`; `variant assertions CA000434 --source erepo --assertion PCER000001171 --version 1.0.0` selects a historical document. | Native framework, source version and assessments survive. A disagreement flag does not silently repair a source label or identifier. |
| `civic` | `get variant ID civic`; returned actions select native profile, collection and cursor. | Local cases enforce membership and preserve compound profile context. A name match remains a candidate. An actual positive joined source capture remains open. |
| `population` | `get variant CA123643 population`; `variant population CA123643 --detail` pages native ancestry rows. | A source-given GRCh38 SNV is required. Missing dataset release keeps counts native; typed frequency needs matching release and counts in the same response. Missing values differ from zero. |
| `population-details` | Retired with `variant population <ID> --detail`. | No inferred frequency or sum across overlapping groups. |
| `gwas` | `get variant rs113488022 gwas` uses GWAS Catalog v2 and native pages. | Valid zero means no associations returned for that request. Search belongs to the GWAS slice. |
| `oncokb` / old `variant oncokb` | `get variant ID oncokb --build GRCh38 --tumor-type TEXT`; the old leaf names this route. | Requires the user's key. Source-native rows keep contexts the common Statement cannot represent. The approved source proof is a demo capture; the product never calls demo. No treatment sensitivity is inferred. |
| `predict` / `predictions` | `get variant ID predict`; `predictions` returns the canonical replacement. | Native scores retain their component terms and versions. Scores are not a BioMCP classification or blended severity. |
| `conservation` | `get variant ID conservation`. | Source scores and context arrays retain their own labels. |
| `cosmic` | `get variant ID cosmic` reads MyVariant's historical snapshot. | Each fragment names its parent record and marks identity unverified. Snapshot age and terms remain visible. No exact allele join is established. |
| `cbioportal` | Retired with `variant recurrence <ID> --source cbioportal --gene SYMBOL --study ID --profile ID --sample-set ID`. | An explicit cohort is mandatory. Distinct matched samples are page-local. Selected sample count is not an assayed denominator. |
| `hotspots` | `get variant ID hotspots` preserves native transcript, residue and alternate counts as candidates. | The clinical review found no established transcript/consequence join. These are not exact allele frequencies or clinical actionability. Reuse terms remain unverified. |
| `cgi` | Retired with an explicit deferral. | No proved current field mapping or speculative section. |
| `all` | Select all implemented bounded sections. | Source restrictions and missing keys still apply. Each source keeps its own outcomes and failures. |
| `variant cspec` | Retired with an explicit deferral. | No additional clinical interpretation or document route is offered. |

The PM dropped a direct dbSNP adapter from 1.0. rsIDs remain available through MyVariant and the ClinGen Allele Registry with that intermediary provenance; they do not establish a direct dbSNP response.

Every source and component starts on with its publisher terms. Commercial users should configure both `noncommercial` and `restricted` groups. A group switches off matching fields inside MyVariant while unrelated fields remain available; explicitly naming `myvariant` switches off the whole service. These rules apply before requests, caches and replay. Named filters obey ClinVar component policy, and gene/protein selectors obey dbNSFP policy. Opaque raw queries are refused when component restrictions apply. No parser rewrites them or switches fields to evade a restriction.

The complete CLI and MCP cases cover executable behavior and output caps. They do not close the remaining actual CIViC positive, same-response gnomAD release or real ClinVar drug-response source proofs. The Cancer Hotspots candidate limitation remains explicit instead of claiming a join that the source does not establish.

### Named search filter contract

Named filters use MyVariant's [documented field, presence and Boolean query syntax](https://docs.myvariant.info/en/latest/doc/variant_query_service.html). Literal ClinVar phrases are escaped as values. BioMCP does not map clinical aliases, combine classifications, infer review stars or diagnose a condition. The two RCV filters match source fields independently; their conjunction does not assert that all values occur on one RCV submission. Native source candidate order, duplicates, total and result window remain authoritative.

| Option | Source clause | Meaning and boundary |
| --- | --- | --- |
| `--review-status TEXT` | `clinvar.rcv.review_status:"TEXT"` | Literal source phrase, without star conversion. |
| `--condition TEXT` | `clinvar.rcv.conditions.name:"TEXT"` | Literal source condition name, without local terminology expansion. |
| `--has clinvar` | `_exists_:clinvar.variant_id` | Source annotation identity exists. No other component selector is offered without proof. |

Each filter requires an existing declared gene/protein or region selector. Named filters cannot accompany raw `--query`, which remains unchanged source text. A region requires its build. MyVariant forbids interval expressions inside parentheses; BioMCP therefore joins declared selectors and filter clauses with `AND` without inventing a raw query parser. Unsupported component selectors and raw-query/filter combinations are refused before source access.

Named filter replies label the independent ClinVar terms from the shared component registry. The same registry controls disabled components before requests, cache and replay. Raw queries remain opaque and cannot enforce a component restriction. With an effective component restriction, they return typed usage guidance instead of sending or rewriting the query. With the default source policy, raw query bytes remain unchanged. Explicitly disabling MyVariant keeps the existing source skip behavior.

The declared gene/protein selector searches dbNSFP fields and carries its own selector terms label. It obeys the dbNSFP component switch even without a named filter. It never switches to another gene or protein field when the caller disables that component. Region selectors remain available independently.

The missing-component mapping remains deferred under the 2026-10-26 PM accounting ruling because its direct source query returned a valid zero, which cannot prove the field mapping. The significance mapping remains deferred: the direct source query for `Pathogenic` also matched a returned record whose captured labels did not contain that literal value. No local post-filter or changed query was used to hide this source matching behavior. The remaining frequency, population, CADD, REVEL, GERP, consequence, impact, loss-of-function and historical tumor-site mappings have no accepted positive filter capture in this implementation. Their legacy field names are not offered speculatively. The therapy mapping also lacks a proved current MyVariant CIViC field. CGI stays deferred. These limitations are proof gaps; Ian's open-by-default source ruling permits restricted components when labeled and enabled, so terms alone do not establish that a filter is implemented.

## 7. Proposed harvested cases

Each Q01–Q20 becomes an outside-in case using the exact paper input and the captured source response. Its assertions distinguish accepted source identity from source coverage. Q21 requires a candidate or usage response and zero guessed allele actions. Q22 edits a captured valid response to a clearly synthetic zero; it asserts source-local empty, not clinical absence. Q23 edits a captured valid response to failure; it asserts retained successful-source facts and an unavailable source, without a replacement classification.

Additional mechanical cases: rs113488022's three alternate alleles; exact candidate selection; explicit build refusal; returned identity mismatch; CAR zero-based start versus source HGVS position; source-stated build absent; transcript version warning; HTTP 200 validation rejection; ERepo no assertion versus no allele; CIViC compound-profile candidate; gnomAD null genome versus zero ac; disabled and missing-key sources before replay/cache; exact study/sample-set recurrence and missing denominator; valid zero on GWAS v2; 429/500/timeout/malformed bodies per adapter; stable native paging, no duplicates/gaps and a warm no-network page; CLI/MCP parity; default and per-section output ceilings.

No ordinary case contacts live providers. Public captures are retained only after their terms permit repository storage; other cases use external `--cases` and recording roots. Failures are response-only edits over actual request identities. Source paths, canonical request IDs and checksums are never reconstructed. The PM owns the implementation tickets filed from the accepted design. This case inventory creates no additional work authorization.

## 8. Reviewed product choices

The PM accepted the five recommendations and settled their command forms in the [canonical variant slice rulings](../../decisions/2026-10-24-variant-slice-rulings.md). The clinical reviewer supplied the source-handling findings above. No clinical decision is delegated to the caller.

| Design question | Accepted choice | Published ruling |
| --- | --- | --- |
| Default and notation resolution | Get accepts published-grammar rsID, CAid, ClinVar Variation ID and reference-qualified HGVS. CAR is the default HGVS resolver; the other two services are one option away. Chromosome HGVS needs `--build`. Gene/protein shorthand is refused with the declared search step. Every candidate returns. | Rulings 1, 2 and 4. |
| Default source fanout | Identity and MyVariant's compact facts form the card. Other sources are named sections and call only their own source. | Ruling 3. |
| Cohort recurrence | Require explicit study, profile and sample set; show denominator and study terms. There is no default cohort. | Ruling 5. |
| Investigate sources | Keep explicit CAR, Mutalyzer and VariantValidator services. Defer variant structure, variant-specific UniProt/InterPro and AlphaGenome. The retired structure command names gene protein. | Ruling 6. |
| Advanced filters and ownership | Port each filter only with a known-positive source case. CGI and variant cspec are deferred. GWAS search, trial search and article search retain their owning slices. Canonical section spellings and retirement messages follow the decision. | Rulings 7, 8 and 9. |

## 9. Investigation measurement appendix

This appendix records the original investigation and its proposed envelopes. Section 6 describes the implemented command contract and its remaining source-proof boundaries. Historical prototype sizes do not replace complete shipped-envelope case caps.

The final evidence manifest, source totals, baseline comparison and proposed byte ceilings follow the same raw captures. Bytes divided by four are estimates, not tokenizer counts. Measured latency includes the M5's concurrent load and network; it is not an isolated speed benchmark. No source ranking or cross-machine comparison follows from these figures.

### Provenance and raw evidence

Live work ran on the configured M5 through `pm job` at the starting revision. Jobs `j10340`, `j10356` and `j10369` hold source discovery, refined joins and declared protein queries. `j10350` is the preserved failed alternate-contig inspection. Jobs `j10356`, `j10365` and `j10382` hold the historical get, search/helper and positive-rsID section measurements. The jobs passed when the bounded script completed and recorded each observed command status; that does not relabel expected source errors or historical command failures as success.

Raw responses, request method/URL/body, timestamps, selected response headers, SHA-256, status, duration, byte counts and truncation markers are retained on Yellow at the evidence store under ticket 0078. `requests.json`, `final-question-matrix.json`, `reviewed-matrix.json`, `declared-protein-queries.json`, `clinical-questions.json`, `baseline.json`, `baseline-extra.json`, `baseline-sections.json`, `proposed-outputs.json`, `comparison.json` and `manifest.json` are the entry points. The manifest includes job logs and scripts. This store is evidence, not a second work-status record.

The first 24 exploratory inputs include redundant forms and an invalid-position diagnostic. The final 23-question corpus replaces the exploratory BRAF rsID absence control with the clinical paper's coding form and retains the rsID experiment separately. Q22 therefore shares Q09's positive captured request, then plans a response-only empty control. Q23 shares Q16's positive request, then plans a response-only failure control. The synthetic controls are proposed outside-in cases, not claimed completed clinical findings. The full final matrix maps these reused responses by question ID.

The official historical binary is BioMCP 0.9.3, darwin-arm64, downloaded from [the tagged release asset](https://github.com/genomoncology/biomcp/releases/download/v0.9.3/biomcp-darwin-arm64.tar.gz). Archive SHA-256 `e637062c18c6692f3ec49c5847e69b29f6feee7f7909345970bd504d72174994` matches its published `.sha256`. Binary SHA-256 is `54f66769d9193c569d1b65756ad766eafd1a2c5e92d08f8c855cc8964e9a7145`. The trial investigation retains the shared release provenance. The installed 0.9.1 executable was inspected only, not used in these comparisons. Static surface accounting also checks historical source revision `4050fdc7dbdf4d713d94cf02a39a147b863665be` (0.9.4); differences from the released executable are labeled below rather than silently substituted.

Each historical command receives an empty owned HOME/config/cache and a keyless environment. The cold get uses `--no-cache`; the immediate second call enables cache. Since the cold call bypasses managed state, the second is not proof of a primed HTTP cache. No hardware speed claim follows from concurrent loaded M5 runs.

### Source request cost

These totals include exploratory mistakes, metadata and bounded retries, not just successful final questions. Per-question coverage is in the matrix. Reused identical requests count once. The initial unprojected MyVariant response adds 508,997 bytes outside this table's later `requests.json` inventory.

| Source | Actual HTTP requests | Status counts | Median seconds | Returned bytes |
| --- | --- | --- | --- | --- |
| car | 13 | 200: 9, 400: 2, 500: 2 | 0.095 | 34,164 |
| cbioportal | 8 | 200: 8 | 0.135 | 8,438 |
| civic | 32 | 200: 32 | 0.278 | 10,523 |
| clinvar | 44 | 200: 39, 429: 5 | 0.162 | 2,008,234 |
| dbsnp | 43 | 200: 40, 404: 1, 429: 2 | 0.184 | 1,135,625 |
| erepo | 9 | 200: 1, 404: 8 | 0.074 | 10,549 |
| gnomad | 8 | 200: 8 | 0.325 | 20,248 |
| gwas | 2 | 200: 2 | 2.473 | 3,417 |
| hotspots | 4 | 200: 4 | 0.127 | 79,129 |
| interpro | 4 | 200: 4 | 1.819 | 10,032 |
| mutalyzer | 13 | 200: 10, 422: 3 | 0.986 | 33,455 |
| myvariant | 49 | 200: 48, 400: 1 | 0.268 | 1,517,828 |
| uniprot | 4 | 200: 4 | 0.593 | 503,044 |
| variantvalidator | 13 | 200: 11, 404: 2 | 0.527 | 27,388 |

Total: 246 HTTP requests and 5,402,074 returned bytes. No response exceeded the probe's 1 MiB bound without a `truncated` marker. A truncated body is not complete-field or assertion proof. Only the documented bounded NCBI retry was used; no retry loop or changed-query retry was used.

Declared MyVariant gene/protein queries returned one candidate for every Q01–Q08 EGFR example and CHEK2 Arg117Gly. The fixed query for Q01 is `dbnsfp.genename:EGFR AND dbnsfp.hgvsp:"p.L858R"`. This is source-documented field construction from explicitly declared options; the source returns genomic ID `chr7:g.55259515T>G`. No local alternate spelling or mutation normalization was tried. Bare `EGFR L858R` returned zero. Q09's unchanged transcript form resolved through CAR to CA123643 and ClinVar Variation 13961, although MyVariant bare text and the historical get failed. CAR also returned both GRCh37 and GRCh38 representations, with separate source coordinate counting. These are retrieval observations, not a clinical equivalence judgment.

The current MyVariant metadata reports build version `20250624`, build date `2025-06-24T16:27:25.889841-07:00`, and COSMIC version 68 with an explicit pre-license-change snapshot note. Dataset ages remain in provenance. They are never presented as a current direct COSMIC release.

### Default get comparison on the question inputs

The proposed outputs are offline projections of retained raw replies, not a working 1.0 command or schema. They measure source identity/candidate and rejected-input headers. They omit historical bulk predictors, repeated commands, `_meta`, unrequested section outcomes and embedded CIViC evidence. Those facts are deferred to named sections, not declared false or absent. They also do not yet measure the complete shipped envelope, every provider's assertion summary or all additional provenance. Before implementation review, the actual generated envelope must be measured and its case ceiling set from the complete output; the projection is a design cost estimate, not a green budget test.

| Case | 0.9.3 cold exit | Cold seconds | Historical bytes | Proposed header bytes | Header estimate tokens | Proposed fixture ceiling bytes |
| --- | --- | --- | --- | --- | --- | --- |
| Q01 | 0 | 0.422 | 9668 | 438 | 110 | 504 |
| Q02 | 0 | 0.628 | 9501 | 438 | 110 | 504 |
| Q03 | 0 | 0.450 | 5996 | 438 | 110 | 504 |
| Q04 | 0 | 0.650 | 3034 | 437 | 110 | 503 |
| Q05 | 0 | 0.355 | 2582 | 437 | 110 | 503 |
| Q06 | 0 | 0.282 | 5419 | 438 | 110 | 504 |
| Q07 | 0 | 0.296 | 3574 | 438 | 110 | 504 |
| Q08 | 0 | 0.389 | 6553 | 438 | 110 | 504 |
| Q09 | 1 | 0.906 | 338 | 818 | 205 | 941 |
| Q10 | 1 | 0.011 | 493 | 271 | 68 | 312 |
| Q11 | 1 | 0.946 | 344 | 802 | 201 | 923 |
| Q12 | 1 | 0.291 | 221 | 340 | 85 | 391 |
| Q13 | 1 | 0.976 | 344 | 756 | 189 | 870 |
| Q14 | 1 | 0.971 | 344 | 797 | 200 | 917 |
| Q15 | 0 | 0.282 | 2220 | 377 | 95 | 434 |
| Q16 | 0 | 0.972 | 2347 | 810 | 203 | 932 |
| Q17 | 0 | 0.992 | 2328 | 813 | 204 | 935 |
| Q18 | 0 | 0.279 | 2325 | 546 | 137 | 628 |
| Q19 | 0 | 0.922 | 2327 | 811 | 203 | 933 |
| Q20 | 0 | 0.957 | 1976 | 820 | 205 | 943 |
| Q21 | 1 | 0.009 | 445 | 146 | 37 | 168 |
| Q22 | 1 | 0.906 | 338 | 818 | 205 | 941 |
| Q23 | 0 | 0.921 | 2347 | 810 | 203 | 932 |

Of 23 get invocations, 16 succeeded and 7 returned an error. These include the ambiguity control and the exploratory rsID positive; repeated failure-control positives do not add independent coverage. Median cold duration was 0.525 seconds; immediate repeat 0.526. Historical JSON ranges from 221 to 9,668 bytes; projected headers range from 146 to 820 bytes.

A larger projected Q09/Q11/Q13/Q14 header replaces a historical error with a source-resolved identity, so its extra bytes buy a previously missing fact. That is not an output-size regression for the same successful answer. It needs an explicit source-resolution case. The prototype's other rejected/empty outputs cannot stand in for source assertions. No tokenizer was run; all token figures use compact UTF-8 bytes divided by four, rounded up.

### Section and helper budget comparison

The source-derived section prototypes use the actual captured first page. ClinVar projects three source submissions and total, CIViC keeps the actual three-item evidence/assertion page, ancestry detail keeps the source groups, and recurrence keeps the exact selected cohort request. They retain record IDs and source fields; original prose remains a deeper selection/link. The conservation projection keeps only source `gerp++`, `phastcons`, `phylop` and `siphy_29way` values and the component license. It does not call the whole dbNSFP record conservation.

Historical section figures below are positive `get variant rs113488022 SECTION` calls. They include the historical whole card, whereas the proposed figures measure the named section payload only. They illustrate disclosure savings; they are not fact-for-fact full-envelope benchmarks. The proposed section ceiling is measured payload plus 15%; the implementation adds its required envelope/provenance before binding the final case ceiling. It must stay within a comparable successful 0.9 output unless the new facts justify an explicit exception.

| Section | Measured payload bytes | Estimated tokens | Proposed payload ceiling bytes | Historical full reply bytes |
| --- | --- | --- | --- | --- |
| clinvar | 1477 | 370 | 1699 | 108197 |
| civic | 1242 | 311 | 1429 | 16102 |
| population | 146 | 37 | 168 | 15473 |
| population-details | 1077 | 270 | 1239 | 15473 |
| hotspots | 1126 | 282 | 1295 | No same-command comparator |
| gwas | 176 | 44 | 203 | 1644 |
| erepo | 258 | 65 | 297 | No same-command comparator |
| cbioportal | 2234 | 559 | 2570 | 10110 |
| predict | 1416 | 354 | 1629 | 9322 |
| conservation | 601 | 151 | 692 | 9414 |
| cosmic | 250 | 63 | 288 | 9160 |
| normalization | 3379 | 845 | 3886 | No same-command comparator |

The investigation did not measure OncoKB execution. Later local cases bind the shipped output budget, and the approved demo capture supplies source proof. No further licensed production capture is required for that roadmap box. CGI has no accepted new section payload and remains deferred. A retained `all` ceiling must sum the selected bounded pages and one envelope, not pull every original record. Search defaults to three measured candidate rows in these probes; the shipped default page and its complete envelope must be measured before choosing the final limit. No arbitrary token ceiling is presented as observed proof.

| Historical helper or section | Cold seconds | Bytes | Observed outcome |
| --- | --- | --- | --- |
| `variant normalize car NM_004333.4:c.1799T>A` | 0.100 | 1254 | exit 0 |
| `variant normalize mutalyzer NM_004333.4:c.1799T>A` | 0.607 | 2734 | exit 0 |
| `variant normalize variantvalidator NM_004333.4:c.1799T>A` | 0.443 | 1256 | exit 0 |
| `variant normalize all NM_004333.4:c.1799T>A` | 0.561 | 5149 | exit 0 |
| `variant structure NM_004333.4:c.1799T>A` | 1.362 | 338 | exit 1 |
| `variant erepo CA123643` | 0.066 | 256 | exit 0 |
| `gene cspec TP53 --limit 3` | 0.417 | 1027 | exit 0 |
| `search gwas --gene CHEK2 --limit 3` | 4.186 | 1749 | exit 0 |
| `batch variant NM_004333.4:c.1799T>A,NM_007194.4:c.470T>C` | 1.944 | 3159 | exit 1 |

The 21 historical search examples include 9 usage errors because the transcript/genomic text is treated as a gene filter and rejected by its gene-symbol validation, 2 other errors, and 10 successful searches. Exact stderr/stdout are retained. This is behavior of the released argument surface, not evidence those variants are absent. The direct resolver results support the accepted reference-qualified positional HGVS form. Search outputs retain source-native candidates rather than locally declaring the most familiar result exact.

The initial historical section run used Q09's transcript input and failed before sections. The corrected positive rsID section run succeeded for all 12 section words. `predict` and `predictions` differ in output byte size and should share one canonical capability in the new schema. The 0.9.4 source inventory contains `gwas`; the 0.9.3 runtime's same word explicitly reports `unavailable`, permanent legacy-endpoint retirement, and a v2 migration gap in the positive run. Do not call that current v2 coverage. `variant structure` and generic batch were executed and failed at the transcript-resolution prerequisite; their source joins remain unmeasured, and deferral rests on the absence of a harvested need, not a claim that those endpoints fail.

### Investigation scope and unproved comparisons

The corpus does not establish an exact same-allele germline-versus-somatic conflict. It does establish a source-paper/reference inconsistency, source name-search ambiguity and multiple alternate alleles for one rsID. The investigation did not prove cBioPortal exact allele recurrence in a selected cohort. The implemented recurrence contract in section 6 reports page-local sample observations without inferring an assayed denominator. A field from the public tier is retained only after a valid untruncated source capture shows it; source docs alone cannot enable a section. Several publishers' output redistribution terms remain unverified, so captures stay outside Git. The proposed source limits, final schema, page sizes and full-envelope ceilings are implementation decisions within the accepted design. No new parser, clinical interpretation, ticket or release is authorized by this file.


### Native ClinVar haplotype subjects

ClinVar VCV000441268 version 5 states archive kind `Haplotype`. Its `ClassifiedRecord/Haplotype` has VariationID `441268` and literal `NumberOfChromosomes="1"`. The direct `SimpleAllele` children have VariationID `17864` with AlleleID `32903`, then VariationID `242765` with AlleleID `32909`. The retained public EFetch reply is `tests/fixtures/clinvar/haplotype-441268.xml`, SHA-256 `5045d9632b079fcf6a4e9c0cdb9a49add9b467fa3239cf96ea2b24c211b92cdd`. These attributes state the grouping. The adapter derives no phase, biological meaning or alternate allele identity.

`VariantRef::clinvar_haplotype(ClinvarHaplotype)` constructs the new `level: haplotype` reference. `VariantRef::haplotype()` borrows its payload. The payload preserves fixed native kind `Haplotype`, checked ClinVar variation identity, unversioned `VcvId` accession, explicit version text, literal chromosome count text and ordered `ClinvarHaplotypeMember` values. Each member preserves checked ClinVar variation identity and `ClinVarAlleleId`. The existing VCV parser checks the composed accession and version. Separate version storage prevents accession equality from hiding a version difference. `members()` keeps its profile API, and existing allele JSON stays unchanged.

The ClinVar subject helper supplies the same source reference to native VCV, RCV and SCV claims. It uses the existing parent identity and cache admission checks. An incomplete compound identity refuses the whole subject with a field problem instead of silently removing a member. The retained reply regression checks the complete subject through the CLI. Removing either member from the recording fails that expectation.
