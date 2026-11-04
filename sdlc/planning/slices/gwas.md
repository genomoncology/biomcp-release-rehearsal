# GWAS slice

This investigation proposes the GWAS slice through steps 1 to 4 of [the slice method](../slices.md). It starts from `85e6ba864c3f13ec50432a58670ec2af13e12857`. It changes no product code. Design acceptance does not mark a capability ported.

## Scope and evidence

The entity is a source association or study. Its output answers which source reported an association, for which allele and trait, in which study context, with which reported statistics. It does not calculate personal risk, infer a causal variant, diagnose disease, or recommend treatment. Human health research questions are in scope. Running an association analysis, installing another tool, computing a polygenic score, fine-mapping and local ontology or gene-name inference are outside this design.

The current variant slice already owns `get variant ID gwas` and its exact-rsID adapter. This slice extends that adapter rather than creating a second GWAS transport, terms declaration, association decoder or effect representation. The variant section remains owned by the variant slice. Study and association identities add context without silently resolving a variant allele from an rsID.

The evidence store holds `data/biomcp-redux/0193-gwas-investigation/2292-gwas-source-and-cost-evidence.tar.gz`, SHA-256 `9ae7b649d7a391dfc67206bf8c9a21c2ec28fd6cd6a3081c5a7a9c1808c6912f`. It contains 160 manifest entries plus the manifest: request plans/scripts, raw responses and headers, source timings, closed-issue inventory, the captured OpenAPI, superseded and final cost projections, and the experiment report. The exact archive member set and all content hashes were independently checked after transfer. Six jobs made 63 bounded GETs and four HEADs in total; `j14247` used retained inputs only. The original M5 scratch is removed after verification. No source body or historical-binary output is checked into this branch.

## Public questions

The bank paraphrases retrieval questions anchored to primary papers. It does not claim the authors asked these questions verbatim. The bank preserves the publication's finding separately from later catalog curation. A source-local empty result and an unavailable source are investigator controls. They establish retrieval behavior only.


The clinical reviewer checked the six primary sources and accepted the attributed answer bank below. These 22 published examples satisfy the 20–40-example floor. Q23–Q25 are additional technical controls. A publication's numerical contrast must not be relabeled as the Catalog's contrast unless the exact association, allele and study match.

| Case | Published retrieval question | Accepted attributed answer | Primary source |
| --- | --- | --- | --- |
| Q01 | Which exact variant labels the first of Easton's five Figure 2 association plots? | rs2981582; the figure reports a per-allele odds-ratio contrast. | [Easton 2007, PMID 17529967](https://pubmed.ncbi.nlm.nih.gov/17529967/), Figure 2a. |
| Q02 | Which variant labels the second plot? | rs3803662; retain that study's per-allele contrast. | Easton 2007, Figure 2b. |
| Q03 | Which variant labels the third plot? | rs889312. | Easton 2007, Figure 2c. |
| Q04 | Which variant labels the fourth plot? | rs13281615. | Easton 2007, Figure 2d. |
| Q05 | Which variant labels the fifth plot? | rs3817198. | Easton 2007, Figure 2e. |
| Q06 | Which sample counts describe the paper's first two stages? | 4,398 breast-cancer cases and 4,316 controls. These are paper-stage counts. | Easton 2007, abstract. |
| Q07 | Which sample counts describe the paper's confirmation stage? | 21,860 cases and 22,578 controls from 22 studies; keep this stage separate from Q06. | Easton 2007, abstract. |
| Q08 | Does the paper's FGFR2 association establish the causative variant? | No such conclusion follows from association alone. The paper says identifying the causative variant remains difficult. Do not substitute a mapped gene as a causal-gene finding. | [Easton full paper](https://pmc.ncbi.nlm.nih.gov/articles/PMC2714974/), discussion. |
| Q09 | What exact allele and contrast accompany Stacey's rs13387042 finding? | A-allele homozygotes have an estimated 1.44-fold risk versus noncarriers in the reported context. This is a homozygote comparison, not a per-allele odds ratio. | [Stacey 2007, PMID 17529974](https://pubmed.ncbi.nlm.nih.gov/17529974/), abstract. |
| Q10 | What exact allele and contrast accompany that paper's rs3803662 finding? | T-allele homozygotes have an estimated 1.64-fold risk versus noncarriers in the reported context. Do not attach this value to another allele or contrast. | Stacey 2007, abstract. |
| Q11 | Which population and tumor context qualifies Q09–Q10? | European-descent study populations and the reported estrogen-receptor-positive tumor association. This does not generalize every later study's population. | Stacey 2007, abstract. |
| Q12 | Which two variants does the later 5p12 paper identify? | rs4415084 and rs10941679. | [Stacey 2008, PMID 18438407](https://pubmed.ncbi.nlm.nih.gov/18438407/), abstract. |
| Q13 | Which sample counts accompany that paper's reported finding? | 6,145 cases and 33,016 controls; retain the paper's preferential estrogen-receptor-positive association context. | Stacey 2008, abstract. |
| Q14 | Can the abstract's OR 1.27 and p value 2.5×10⁻¹² be assigned to rs4415084? | No. Those abstract values concern rs10941679. Exact variant-to-effect identity is required. | Stacey 2008, abstract. |
| Q15 | Which SMAD7-region variant does Broderick report for colorectal cancer? | rs4939827, with a combined trend p value of 1.0×10⁻¹² across the reported four sample sets. | [Broderick 2007, PMID 17934461](https://pubmed.ncbi.nlm.nih.gov/17934461/), abstract. |
| Q16 | Are all 940 discovery individuals in that paper colorectal-cancer cases? | No. The reported familial colorectal-tumor set contains 627 colorectal-cancer cases and 313 advanced adenomas, with 965 controls. Preserve these categories. | Broderick 2007, abstract. |
| Q17 | Which replication sample counts does that paper report? | 7,473 cases and 5,984 controls. Do not merge them with Q16 as unique people. | Broderick 2007, abstract. |
| Q18 | Which locus and variant does Tomlinson identify? | rs6983267 at 8q24.21 for colorectal-cancer susceptibility in the reported study. | [Tomlinson 2007, PMID 17618284](https://pubmed.ncbi.nlm.nih.gov/17618284/), abstract. |
| Q19 | Which significance values belong to the original and overall-replication contexts? | Initial allelic p value 1.72×10⁻⁷; overall replication p value 1.27×10⁻¹⁴. They are distinct results. | Tomlinson 2007, abstract. |
| Q20 | What are the two genotype contrasts in the reported odds ratios? | Heterozygotes: 1.27 (1.16–1.39); rare homozygotes: 1.47 (1.34–1.62). Neither is automatically the Catalog's per-allele OR. | Tomlinson 2007, abstract. |
| Q21 | Which 9p21.3 variant anchors Samani's coronary finding? | rs1333049. | [Samani 2007, PMID 17634449](https://pubmed.ncbi.nlm.nih.gov/17634449/), abstract. |
| Q22 | Which distinct cohorts and phenotypes qualify that finding? | WTCCC: 1,926 coronary-artery-disease cases and 2,938 controls, p=1.80×10⁻¹⁴. German MI Family: 875 myocardial-infarction cases and 1,644 controls, p=3.40×10⁻⁶. Keep CAD and MI labels distinct. | Samani 2007, abstract. |
| Q23 | What does a successful empty rs113488022 Catalog query establish? | Only zero source records for the exact endpoint, query and capture date; no clinical negative. | Investigator control using the existing variant slice's exact-rsID input. |
| Q24 | What does a failed GWAS source establish? | Retrieval failed. Do not fabricate zero associations or a risk conclusion. | Synthetic provider-failure control; no deliberate live fault request. |
| Q25 | If the first rsID page lacks the target publication, is its association absent? | No. The first page is incomplete when source metadata reports further pages. Preserve continuation and search by exact publication/study when needed. | Investigator control using Q01 and source page metadata. |


### Product question coverage

The clinical review for this accounting preserves the accepted question wording and source discrepancies. Answered means that an existing case supplies the requested retrieval fact. Partial means that a requested facet is retrieved while the full paper answer remains unavailable. Unanswered means that the implemented Catalog commands do not supply the question's answer. These statuses do not change the attributed publication answers above. Native witness names refer to the retained external cases and budget files below; they require no new source request.

| Question | Product status | Existing case witness | Boundary |
| --- | --- | --- | --- |
| Q01 | Answered | pubmed-first | 6110 retains the requested rsID. |
| Q02 | Answered | pubmed-first | 6111 retains the requested rsID. |
| Q03 | Answered | pubmed-first | 6112 retains the requested rsID. |
| Q04 | Answered | pubmed-second | 6113 retains the requested rsID. |
| Q05 | Answered | pubmed-second; association-6114 | 6114 retains the requested rsID. |
| Q06 | Unanswered | study-GCST000035; ancestries | Native sample prose differs from the requested paper-stage counts. |
| Q07 | Unanswered | study-GCST000035; ancestries | Native sample prose does not supply the paper confirmation-stage counts. |
| Q08 | Unanswered | loci | Reported gene IDs do not answer causality. |
| Q09 | Partial | stacey; study-GCST000037 | A allele and native per-copy effect are retained; the paper homozygote contrast is not supplied. |
| Q10 | Partial | stacey; study-GCST000037 | T allele and native per-copy effect are retained; the paper homozygote contrast is not supplied. |
| Q11 | Partial | study-GCST000037 | Native ancestry context does not supply the complete paper population and tumor context. |
| Q12 | Unanswered | pubmed-negative | The exact publication query returns zero; no substituted answer. |
| Q13 | Unanswered | pubmed-negative | The zero reply supplies no paper sample counts. |
| Q14 | Unanswered | pubmed-negative | The zero reply supplies no variant-to-effect assignment. |
| Q15 | Answered | broderick | 5949 retains rs4939827-T and exact 1e-12. |
| Q16 | Unanswered | study-GCST000113 | Native counts do not supply the paper discovery category split. |
| Q17 | Answered | study-GCST000113 | Native replication prose retains 7,473 cases and 5,984 controls; the separate ancestry-summary discrepancy stays visible. |
| Q18 | Partial | tomlinson; association-6063 | The requested rsID is retrieved; no current case proves 8q24.21. |
| Q19 | Unanswered | tomlinson | The native statistic does not supply either requested paper context. |
| Q20 | Partial | tomlinson; association-6063 | The source-labeled effect and interval do not supply the paper genotype contrasts. |
| Q21 | Answered | samani; association-11386 | The requested rsID is retrieved. |
| Q22 | Partial | samani; study-GCST000057 | Native study context does not supply the separate paper cohorts and p values. |
| Q23 | Answered | variant-negative; variant/population/catalog-zero.yaml | Only source-local zero is established. |
| Q24 | Answered | gwas-search/failures-are-never-empty | Synthetic failures retain failed status and never become zero associations. |
| Q25 | Answered | variant-positive; variant-publication; pubmed-first; pubmed-second; associations-0; associations-1 | Further source pages remain explicit and publication records remain reachable. This is not a completed rsID-wide walk. |

### Publication and Catalog disagreement

The live Catalog study for PMID 17529967 is `GCST000035`. Its initial sample text says “390 European ancestry cases, 364 European ancestry controls”. Its replication text says “4,364 East Asian ancestry cases, 24,174 European ancestry controls, 3,564 East Asian ancestry controls, 24,391 European ancestry controls”. These native strings differ from Q06–Q07 and include a repeated control label. Catalog metadata differs from the publication; no reconciliation performed. Paper-wide stage counts and one Catalog study record have different scopes, but that possible scope difference does not explain away the unresolved prose. Do not correct, sum or deduplicate it.

The Catalog publication query reports six associations, including rs981782 as well as the Figure 2 variants. Do not discard the sixth record to make the source fit the question bank. A later source page and the source publication are separate attributed evidence. The original paper's effects are not silently substituted for current Catalog effects.

## Source contracts

The ROADMAP names GWAS Catalog as the GWAS source. The public v2 API is the proposed initial source. The [official API guide](https://www.ebi.ac.uk/gwas/docs/programmatic-access/rest-api/) and [migration guide](https://www.ebi.ac.uk/gwas/docs/news/rest-api-v2-migration-guide/) distinguish curated association data from complete genome-wide summary statistics. V1 is retired. No request falls back to v1.

The source exposes studies, associations, traits, genes, publications and study ancestry metadata. Source filters, their combinations and sort order need captured known-positive checks before implementation enables them. Public documentation does not by itself prove that a particular response carries a field. The implementation must compare returned study, association, rsID and ontology identities with its checked request.

### Measured public-tier behavior

Jobs `j14205` and `j14220` made 35 and 18 bounded GET requests from the M5. Each URL was requested once per plan; the second job completed an observed next page and investigated newly returned identifiers. Both refused redirects and retries, capped each body at 3 MiB and each request at 20 seconds, and stopped on a provider 429 or 5xx. These are source discovery measurements, not product acceptance or speed baselines. The source Date headers place this capture on 2026-10-26 UTC. No credentials or summary-statistics files were used.

| Direct query | Native source result | Requests, response bytes and seconds |
| --- | --- | --- |
| PMID 17529967 associations, both three-row pages | Six unique associations for GCST000035, including all five Figure 2 variants and rs981782. | 2; 3,261 + 3,238 bytes; 0.580 + 0.738 s. |
| PMID 17529974 associations | Two records for GCST000037: rs13387042-A and rs3803662-T. | 1; 2,022 bytes; 0.581 s. |
| PMID 18438407 associations | Valid zero. This is an unresolved Catalog retrieval gap for Q12–Q14, not absence of the publication or its association. | 1; 179 bytes; 0.415 s. |
| PMID 17934461 associations | One record: 5949, GCST000113, rs4939827-T, reported colorectal cancer, mapped MONDO_0005575. | 1; 1,122 bytes; 0.744 s. |
| PMID 17618284 associations | One record: 6063, GCST000053, rs6983267-G. Preserve the effect-label discrepancy below. | 1; 1,148 bytes; 0.455 s. |
| PMID 17634449 associations | Seven records for GCST000057, including 11386/rs1333049-C. | 1; 6,715 bytes; 0.862 s. |
| Exact association 6114 | Returned association_id 6114, GCST000035, rs3817198-C, PMID 17529967. | 1; 915 bytes; 0.456 s. |
| Exact study GCST000035 | Returned accession_id GCST000035 and publication/sample metadata. | 1; 1,237 bytes; 0.523 s. |
| GCST000035 ancestries | Three source rows; this endpoint supplies an unpaged collection, not a source page object. | 1; 2,210 bytes; 0.555 s. |
| Exact gene FGFR2 | gene_name FGFR2, Ensembl ENSG00000066468 and Entrez 2263. These source-provided stable IDs support a gene pivot. | 1; 317 bytes; 0.410 s. |
| MONDO_0004989 associations, without/with child traits | 2,094 / 2,422 source associations. Child expansion is explicit source behavior. | 1 each; 3,178 / 3,174 bytes; 0.647 / 0.908 s. |
| FGFR2 standard/extended gene set | 212 / 223 source associations. The larger set is not interchangeable with the default. | 1 each; 3,544 / 3,540 bytes; 3.147 / 13.738 s. |
| FGFR2 and MONDO_0004989 together | 40 source associations in one combined request. This proves a native conjunction rather than intersection of independent prefixes. | 1; 3,967 bytes; 3.127 s. |
| rs2981582 and PMID 17529967 together | One exact association, 6110/GCST000035, rs2981582-A. | 1; 1,129 bytes; 0.658 s. |
| rs113488022 | Valid zero at both tested page sizes; the absence is source-local. | 1 per size; 176 bytes each; 0.489 / 0.508 s. |

The first job also probed 22 distinct rsIDs to expose field variability, separate from the accepted question denominator. Source totals range from zero to 353. Its 30 API-response timings have median 0.805 s and range 0.404–13.738 s on that host and load. No warm product call was executed. The extended-gene request approaches the current adapter's 15-second deadline; it does not justify a silent deadline increase.

Two discovery mistakes are retained as negative evidence. The historical trait ID EFO_0000305 returned zero while source records use MONDO_0004989 for breast carcinoma. Do not locally rewrite the former into the latter. A documented-looking `/genes?gene_name=FGFR2` request returned an unrelated page of 38,436 genes. Its HTTP 200 and echoed URL do not prove that the filter applied. The corrected direct `/genes/FGFR2` request supplies checked identity. A guessed OpenAPI path returned 404 and a legacy path returned 410; neither is a source-data failure or an API capability verdict.

### Fields proved by capture

| Public resource | Fields actually captured | Proposed use |
| --- | --- | --- |
| Association list and direct record | association_id, accession_id, reported_trait, efo_traits with ID and label, snp_allele with rsID and effect allele, p_value plus mantissa/exponent, odds-ratio fields or beta text, range, risk_frequency including NR, source mapping strings, PMID, author, optional numeric interval bounds and source links. | Compact association header and selected full card. Preserve precision and source text markers; complete mapping/locus context is deeper. |
| Study | accession_id, integer PMID, disease_trait, mapped traits, initial/replication sample text, discovery/replication ancestry summaries, genotyping technologies, SNP count, flags, summary-statistics availability and license URL. | One study card, with distinct stages and attributed source prose. A publication title was not in this capture; it needs its publication resource before being promised. |
| Study ancestry collection | stage/type, number_of_individuals, ancestry-group array, country-of-origin and recruitment arrays, per-row source self link. | Bounded retained section pages after one bounded study metadata read. Preserve NR and missing country regions, and verify every self link belongs to the requested study. |
| Exact gene | gene_name, description, location string, cytogenetic region, biotype, Ensembl-ID array and Entrez-ID array. | Exact source identity bridge to the existing gene entity. Do not perform substring filtering on the unrelated collection response. |
| Trait list and exact trait | source trait ID, whole label and ontology URI; the collection has a page object. | Preserve generic ontology namespaces, including MONDO. Do not restrict the type to an EFO prefix. |

The source ancestry endpoint is unpaged. Section limits bound the user output, while the transport admission limit bounds the one whole-study response. The source receives no invented size parameter. Subsequent section pages read the retained response; they do not refetch it. The default study card keeps only the source ancestry summaries. Recruitment detail sits behind `ancestries`. The capture establishes three rows for this study, not a global upper bound.

The Catalog's 6063 record labels 1.27 [1.16–1.39] as `or_per_copy_num`, while Tomlinson's paper attributes those same values to heterozygotes. This is an explicit source-label inconsistency. Preserve both source identities and labels without conversion or reconciliation. Stacey's Catalog per-copy effects (1.20 and 1.28) and paper homozygote contrasts are different scopes, rather than an established inconsistency. Samani's Catalog p=3×10⁻¹⁹ and paper cohort-specific p values likewise remain separate analyses unless an exact comparison proves otherwise.

Job `j14249` completed four exact study cards and the missing publication's study query. GCST000113 initially reports 930 European-ancestry cases and 960 controls, rather than the paper-wide Q16 counts. Its replication sample prose reports 7,473 cases and 5,984 controls while its ancestry summary says 13,475; preserve both native statements without reconciliation. GCST000037 has multiple replication ancestry categories, so Q11's paper context cannot replace its full source metadata. GCST000057 has one native “Coronary heart disease” label even though Q22 distinguishes the paper's CAD and MI cohorts. The study query for PMID 18438407 also returned zero. Every published question therefore has an actual Catalog request or an explicitly reported source limitation; shared publication/study responses serve several questions without duplicate provider requests.

| Question group | Direct Catalog evidence and answer scope |
| --- | --- |
| Q01–Q05 | Association IDs 6110–6114: respectively rs2981582-A / OR 1.26 / p=2×10⁻⁷⁶; rs3803662-T / 1.20 / 1×10⁻³⁶; rs889312-C / 1.13 / 7×10⁻²⁰; rs13281615-C / 1.08 / 5×10⁻¹²; rs3817198-C / 1.07 / 3×10⁻⁹. All retain GCST000035 and PMID 17529967. |
| Q06–Q08 | GCST000035 metadata and 6110's explicit locus/gene IDs; paper counts and causal-variant caution remain separately attributed. The API does not perform the paper's interpretation. |
| Q09–Q11 | GCST000037 and associations 6116/6117 preserve exact A/T alleles, native per-copy effects and study ancestry. Homozygote comparisons remain paper facts. |
| Q12–Q14 | Both association and study queries by PMID 18438407 return zero. The paper remains the evidence; no repaired query or substitute publication is used. |
| Q15–Q17 | GCST000113 and association 5949 retain rs4939827-T and native p=1×10⁻¹². Paper-wide sample categories and discrepant source metadata remain distinct. |
| Q18–Q20 | GCST000053 and association 6063 retain rs6983267-G, the exact source-labeled effect and its interval. The source does not settle the contrast discrepancy. |
| Q21–Q22 | GCST000057 and association 11386 preserve rs1333049-C, source OR 1.36 and p=3×10⁻¹⁹. Paper cohort-specific p values are not overwritten. |
| Q23–Q25 | Actual exact-rsID zero, a separately planned synthetic failure, and a first page with further source pages. No live error was induced. |

Job `j14229` followed the Swagger initializer's actual configuration link and captured the 84,674-byte publisher OpenAPI YAML at `/gwas/rest/api/v2/rest-api-doc.yaml`. It confirms `show_child_trait` in the singular and no `gene_name` collection filter or native p-value-threshold parameter. The human guide's gene-collection example therefore does not replace the actual endpoint contract. The captured exact association locus endpoint supplies `author_reported_genes` with FGFR2, ENSG00000066468 and Entrez 2263, separately from the association's mapped-gene strings. Its 333-byte response proves the proposed `loci` section. The same job proves publication associations sorted by native `p_value` ascending: the first three identities are 6110, 6111 and 6112. This is source order under an explicit selector, not a local significance or clinical ranking.

### Additional selector and threshold admission evidence

Job `j14351` ran the landed source helper at `6e20aa069bbb981f9c9c6f0da3fd4ba31e556ceb` on the M5. Its fixed plan made 14 keyless GET requests once each with one second between admissions, a 20-second deadline and a 3 MiB body bound. It refused redirects and retries and would stop on a failed reply, including 429 or 5xx. All 14 replies were HTTP 200. This is native API evidence, not product execution or clinical interpretation.

Every row below cites `data/biomcp-redux/0205-gwas-captures/j14351-native-evidence.tar.gz`, SHA-256 `06efd647584171055e499d02edb3630d4fa75f4bc070045676f13aa286363a4b`. The archive contains the exact URL plan, raw bodies and headers, per-attempt status, bytes, SHA-256 and elapsed time, the analysis recipe and ledger, and reused publisher OpenAPI and PMID evidence. The transferred archive's exact 52-member set and 51 content hashes were independently verified on Yellow before scratch cleanup. Every attempted reply's self link echoed every sent selector, sort, direction, page and size. All positive pages carried exact p-value parts in nondecreasing order. These observations cover the captured pages and do not establish an immutable catalog snapshot.

| Witness in the archived ledger | Native result | Capability unlocked or deferred |
| --- | --- | --- |
| `variant-positive`: `rs_id=rs2981582`, ascending `p_value` | Five total; first IDs 6110, 191182970, 217418586; exact values 2e-76, 2e-73, 1e-12. | Unlock exact variant selection under the required default order. |
| `trait-text-positive`: literal `efo_trait=breast carcinoma`, ascending `p_value` | 2,094 total; IDs 41310323, 93928402, 41311109; exact values 2e-322, 2e-291, 1e-245. | Unlock `--trait` as the native literal `efo_trait` label selector under the accepted ruling. It does not search `reported_trait` prose; see the field distinction below. |
| `trait-id-positive`: `efo_id=MONDO_0004989`, `show_child_trait=false`, ascending `p_value` | 2,094 total; the same first three identities and exact values as the literal-label witness. | Unlock exact trait-ID selection under the required order. No ID rewriting. Earlier child-inclusion captures remain separate. |
| `gene-positive`: `mapped_gene=FGFR2`, `extended_geneset=false`, ascending `p_value` | 212 total; IDs 41310323, 93928402, 105594966; exact values 2e-322, 2e-291, 4e-254. Each returned mapped-gene array contains FGFR2. | Unlock the native mapped-gene selector under the required order. Earlier extended-set evidence remains separate. |
| `study-positive` and `study-page-one`: `accession_id=GCST000035`, ascending `p_value`, size 3, pages 0 and 1 | Six total; distinct IDs 6110, 6111, 6112, then 6113, 6114, 11346. All six retain the requested study accession. Page 0 values are 2e-76, 1e-36, 7e-20; page 1 values are 5e-12, 3e-9, 9e-6. | Unlock study association selection and its `associations` section, a non-PMID continuation and the native threshold page-boundary witness. |
| `gene-trait-positive`: FGFR2 plus MONDO_0004989, both expansion flags false, ascending `p_value` | 40 total; IDs 41310323, 93928402, 41311109, each with both requested source identities; exact values 2e-322, 2e-291, 1e-245. | Unlock this captured native conjunction under the required order. It does not authorize arbitrary selector combinations. |
| Six negative selector rows | `rs113488022`, literal `BioMCP0205NoSuchReportedTrait`, `EFO_0000305`, mapped gene `BIO0205NOSUCHGENE`, study `GCST999999999` and PMID 18438407 each return zero. | Prove source-local empty boundaries for variant, literal trait label, trait ID, gene, study and publication. Synthetic sentinel text and IDs supply no clinical meaning. EFO_0000305 is never rewritten. |
| `underflow-search`: unfiltered ascending `p_value`, size 3 | IDs 95284497, 93538837 and 192643093 retain exact parts 2e-28539, 3e-26677 and 1e-25902. Each native numeric `p_value` is 0.0. | Unlock an actual outside-finite-double-range witness for exact numeric decoding. Preserve mantissa/exponent; a native floating-point zero cannot replace them. No whole-catalog scan occurred. |

The positive text request sends the publisher's captured label `breast carcinoma` unchanged through its documented `efo_trait` parameter. Its rows have mapped label `breast carcinoma` and separate `reported_trait` text `Breast cancer`. The capture proves literal source trait-label retrieval; it does not prove a filter on the reported-trait array or authorize rewriting caller text between those fields. The accepted PM ruling limits `--trait` to the literal native `efo_trait` label. Help must say that it does not search `reported_trait` prose. Arbitrary reported-trait prose filtering remains outside this command contract.

The retained `j14229` PMID 17529967 sorted page remains the publication-positive proof; its raw reply and original receipt are copied unchanged into this archive. Together with the new source-local zero, it supplies the publication boundary without repeating the successful request.

For the exact threshold 1e-15, all three GCST000035 page-0 rows qualify and the first page-1 row, 5e-12, is above the threshold. This proves an actual native page crossing for the accepted continuation algorithm. The PM's GWAS ruling supersedes the earlier page-scoped threshold proposal in this document: use exact ascending order, continue while all received rows qualify, stop at the first above-threshold row, include equality and report a qualifying total only after reaching the end. The first page alone supplies no complete qualifying total. No threshold answer was calculated as a clinical result.

### Identity and joins

| Identifier | Meaning and allowed join | Forbidden inference |
| --- | --- | --- |
| `GCST…` | Exact source study accession; links study metadata and its association collection. | A publication is not one study; never collapse distinct accessions by PMID. |
| Association ID | Positive source record identifier qualified as a GWAS Catalog association. | Equal rsID, trait or effect values do not make two association records equal. |
| rsID | Exact source rsID, passed to the existing variant lookup. | An rsID need not name one genomic allele. Never select the first allele or invent a build. |
| EFO or other source ontology ID | Exact mapped trait identifier plus the source label and reported trait text. | Do not equate a trait label with a disease ID or match labels locally. |
| Source gene identifier | Exact returned stable gene identity, with mapping origin retained. | A reported or nearest mapped gene is not established as the causal gene. A symbol alone does not authorize a cross-source identity join. |
| PMID | Exact source publication identity, passed to the article entity. | Do not copy a paper's abstract into every association or treat one PMID as one analysis population. |

Coordinates, if later exposed, carry the build actually stated by the source. This first design needs rsID joins and does not add interval search, remapping or a local build assumption.

## Proposed commands and disclosure

The proposed grammar follows the existing `search`, `get` and entity relation forms. Search returns association headers. A selected association returns its source record. A selected study returns study metadata. Source order and source sort remain explicit.

| Form | Default header or card | Further disclosure |
| --- | --- | --- |
| `search gwas --variant rsID` | One bounded association page with association ID, study accession, reported and mapped traits, effect allele, p value and reported effect. | Each exact association get, continuation, and bounded source-qualified pivots. |
| `search gwas --trait TEXT` | Literal publisher trait search, echoed unchanged; no local expansion. | Caller can choose exact ontology IDs from source results. |
| `search gwas --trait-id ID` | Associations with the exact mapped ontology ID. | Explicit child-trait inclusion changes the source request and is echoed. |
| `search gwas --gene GENE` | Associations selected by the publisher's documented mapped-gene definition. | Explicit extended gene-set choice preserves legacy v1 meaning where needed. |
| `search gwas --study GCST…` | Associations belonging to the exact study. | Source-native association continuation. |
| `search gwas --pubmed PMID` | Associations from the exact publication, retaining all study IDs. | Study gets and article pivot. |
| `get gwas association:ID` | Exact association identity, all source effect alleles and traits, reported effect and interval, source study and publication IDs. | Study card, exact-rsID variant actions and publication. |
| `get gwas association:ID loci` | Source locus descriptions, strongest-risk-allele strings and author-reported gene IDs. | Exact gene pivots only from the returned stable IDs; no causal-gene assertion. |
| `get gwas GCST…` | Exact study accession, publication, reported and mapped traits, sample descriptions, summary-statistics availability and source date where supplied. | `associations` and separately paged `ancestries` when proved by capture. |
| `get gwas GCST… associations` | One bounded native association page belonging to this study. | Continuation with the original checked study and source query. |
| `get gwas GCST… ancestries` | One bounded retained ancestry/sample metadata page. | Local continuation over the one bounded source collection; never a patient-level denominator. |

All search filters are explicit. Empty search input is a usage error, rather than a scan of the whole catalog. The command-specific page limit and offset use the existing page/pool contract. A continuation preserves every original selector and sort choice. Reads never refresh a bundle or fan out into summary-statistics files.

Use the documented native p-value-ascending order as the default association order, matching the useful historical search behavior. Echo that choice in the query. The captured PMID 17529967 default provides a concrete top-hit story: 6110/rs2981582, 6111/rs3803662, then 6112/rs889312. Source ordering does not imply clinical importance, causality or a complete summary-statistics ranking.

Keep `--p-value` as an explicit reviewed capability. The public endpoint has no native threshold selector. The recommendation is a filter over one retained source page with separate unfiltered source count, returned count and unknown global qualifying count. Preserve a continuation even when the current page filters to zero. Never relabel a partial scan's count as the complete threshold result. A complete threshold search needs separately approved bounded scanning semantics; an implementation must not present the old bounded-prefix filter as a full-catalog search.

The API provides no immutable catalog snapshot token in these captures. A pool preserves the source pages already received and all checked selectors. It cannot promise an atomic database snapshot across later live page requests. Preserve received identities, reject duplicate/mismatched pages, and expose source drift instead of claiming a stable whole-catalog walk. No full-data walk is needed to land the initial commands.

The initial page candidate is three associations. The measured native projection, complete envelope and action bytes determine whether that remains useful; no unmeasured hard token ceiling is proposed as an observation. A caller may request a larger bounded page. Study metadata is fetched only by its explicit get or selected section, not once per search hit.

Each response has one pure next-step function under architecture 6.10. Search selects one association get per row before continuation. Source-order distinct linked study, rsID and publication pivots follow the architecture's bound. A source error yields the existing fresh action; an expired pool yields restart. An association get never emits its own get as a step. Steps use the current source-given identity rather than the caller's spelling or a locally guessed symbol.

## Proposed types and schemas

The shared biodata layer owns checked study accessions, qualified association IDs, source ontology IDs, exact rsIDs, text and finite/exact numeric representations. The query layer owns request selectors and response records. The source adapter owns source field spelling and validation. Each fact has one definition; generated schemas and help derive from those definitions.

`GwasSearch` contains explicit optional variant, trait text, trait ID, mapped-gene, study and PMID selectors, explicit child-trait and extended-gene-set choices, page window and any reviewed p-value selector. `GwasGet` is an explicit study-or-association sum. Study-only sections reject association input before I/O. Every selector requires a captured positive case and an invalid-input case.

An association record preserves its association ID, study accession, reported-trait array, mapped-trait ID/label array, all rsID/effect-allele pairs, p-value representation, odds ratio, beta text/number/units where supplied, interval, frequency text including unknown markers, PMID, author and source mapping context. Missing values are optional; an unknown frequency is not zero. Odds ratio and beta are separate optional fields. Neither overrides the other, and an effect cannot be interpreted without its allele, unit and study context. A zero caused by floating-point underflow cannot replace the source's exact mantissa/exponent representation.

A study card preserves source accession, publication and source-native sample descriptions. Discovery and replication populations remain separate. Ancestry rows preserve source categories, stage, counts and recruitment metadata where supplied; they never sum independently described samples into unique people. Summary-statistics availability is a source flag or link, not proof that full data were downloaded, validated or licensed for every use.

The new schemas are versioned response and request schemas. Extension of the existing shared association representation must preserve variant-section compatibility. A captured public field absent from some records remains optional. An unproved public field stays outside the initial schema. No dictionary of local risk labels, inferred disease names or gene heuristics is proposed.

The external biodata repository provides reusable identity/provenance concepts, while this workspace's current `crates/biodata` has no checked GCST or exact scientific p-value type. Those are proposed additions, not existing capabilities. Capture-before-types applies to them. Reuse existing variant identity and text types, add only the checked values these source contracts require, and keep all source decoding in the one existing GWAS adapter. A schema change that adds exact p-value parts to the existing variant section needs explicit compatibility/version handling rather than an unversioned field substitution.

## Source and bundle disposition

Keep the existing keyless GWAS Catalog API and source policy. API metadata and complete summary-statistics datasets have distinct rights. The [publisher FAQ](https://www.ebi.ac.uk/gwas/docs/faq/) places non-summary catalog data under EBI service terms and describes study-specific exceptions for summary statistics. Preserve the current restricted source category until the PM reviews any change. Enabling the source does not authorize every reuse of every linked dataset. No key is required and no key value enters the design or evidence.

Keep the original source captures outside Git. Prefer bounded API reads for the harvested questions. The [file download inventory](https://www.ebi.ac.uk/gwas/docs/file-downloads/) supplies the four current release URLs below. Job `j14240` used one HEAD per file with identity encoding, no redirects and a 20-second deadline. These are server-reported object lengths, not downloaded or decompressed sizes. No bundle body was downloaded, parsed or indexed.

| Candidate release object | HEAD Content-Length | Source Last-Modified, UTC | Disposition |
| --- | --- | --- | --- |
| gwas-catalog-associations-full.zip | 68,743,246 bytes | 2026-10-22 10:40:20 | Defer. Bounded association queries answer the measured needs without whole-catalog storage. Uncompressed size and build cost remain unmeasured. |
| gwas-catalog-studies.tsv | 77,196,598 bytes | 2026-10-22 10:40:17 | Defer. Exact study reads preserve current source identity and metadata. |
| gwas-catalog-ancestry.tsv | 51,955,277 bytes | 2026-10-22 10:40:18 | Defer. Explicit per-study metadata suffices for the bank. |
| gwas-efo-trait-mappings.tsv | 19,481,277 bytes | 2026-10-22 10:40:17 | Defer. The trait endpoint supplies source IDs and labels without a local name-matching index. |

The publisher describes curated releases approximately every two weeks; that is distinct from the adapter's one-day cache freshness. File Last-Modified and a fresh response are not a guarantee that every study changed on that date. The API responses do not carry an immutable catalog release token. Do not add a speculative `sync gwas` command for deferred bundles. No summary-statistics bundle or tool execution belongs to this first slice. Do not add GWAS Central, Enrichr or another source without a harvested unanswered question and a separate source decision.

## Measured disclosure costs

The original jobs `j14240` and `j14247` measured source-derived disclosure projections before the command envelopes existed. Those design projections are historical planning inputs, not full response measurements or approved output caps. The superseded prototypes also omitted relation self links. The following table uses the retained schema-validated native product measurements from the search and exact-get jobs below. Full CLI bytes include the final newline. Comparable MCP bytes remove only the publisher `use`, `terms` and `terms_url` fields; they exclude part of the envelope and must not be described as full responses. Estimated tokens are ceil(comparable bytes/4). Every row has the standing fifteen-percent full response ceiling in its owning budget file.

| Disclosure and retained case | Full CLI bytes | Comparable MCP bytes | Estimated comparable tokens | Full ceiling | 0.9 comparison |
| --- | ---: | ---: | ---: | ---: | --- |
| Default rs2981582 page (`variant-positive`) | 2230 | 2106 | 527 | 2565 | Not measured; contained binary deferred to 0098. |
| PMID 17529967 first page (`pubmed-first`) | 2254 | 2130 | 533 | 2593 | Same containment limitation. |
| FGFR2 plus mapped trait (`gene-trait-positive`) | 2341 | 2217 | 555 | 2693 | Same containment limitation. |
| Exact association 6114 (`association-6114`) | 1130 | 1006 | 252 | 1300 | No historical exact association leaf. |
| Exact study (`study-GCST000035`) | 1561 | 1437 | 360 | 1796 | No historical exact study leaf. |
| Study ancestry section (`ancestries`) | 2390 | 2266 | 567 | 2749 | No historical ancestry section. |
| Association 6110 loci (`loci`) | 795 | 671 | 168 | 915 | No historical locus section. |

`tests/gwas-search-budgets.yaml`, `tests/gwas-get-budgets.yaml` and `tests/gwas-threshold-budgets.yaml` own all retained native measurements. Their existing full and comparable ceilings remain unchanged. The locus output includes the shipped query, actions and envelope, which the historical 639-byte disclosure projection did not measure. Its earlier 735-byte candidate was a projection ceiling rather than a full-envelope allocation. The accepted exact-get proof measured and capped the complete 795-byte CLI response. No new native replay or provider call is needed for this accounting.

The same-question 0.9 comparison is unmeasured because no contained historical binary was executed. Native API bytes measure source transport and the projections measure proposed disclosure; neither substitutes for 0.9 product bytes, warm/cold product latency or a judged clinical quality comparison. There is no defensible numeric claim that these outputs outperform 0.9. The source fields and named legacy capabilities above define the quality floor that implementation and the later contained comparison must check.

The search implementation emits only supported native page, variant and publication actions. Association and study get actions join when the exact GWAS get handlers land; a search never suggests an unsupported command. Native offset continuation retains every selector and source order. It does not claim a pinned whole-search snapshot. Exact warm pages reuse the existing transport cache.

### Additional command request captures

Job `j14615` sent eight previously uncaptured exact sorted requests once each through the landed bounded capture helper. It retained the existing adapter deadline of 15 seconds and a 3 MiB body bound, admitted requests one second apart, and refused redirects and retries. All eight returned HTTP 200. The extended gene-set request took 14.357 seconds; the deadline remains unchanged.

The archive is `data/biomcp-redux/0234-gwas-search/native-evidence.tar.gz`, SHA-256 `4a10b3ed6d0b0186092d590fe84473482b4b85ed80d39fb125d26750ab92b59a`. The job copied the archive to Yellow and verified its exact manifest and member hashes before removing source-machine scratch. Original replies remain outside Git. The other selectors reuse the sorted requests from `j14351` and the first sorted publication page from `j14229`.

| Exact request witness | Native result |
| --- | --- |
| PMID 17529967, page 1, size 3, ascending p value | IDs 6113, 6114 and 11346; six total. The last record retains rs981782. |
| PMID 17529974, page 0, size 3, ascending p value | IDs 6117 and 6116; two total for GCST000037. |
| PMID 17934461, page 0, size 3, ascending p value | ID 5949; one total. |
| PMID 17618284, page 0, size 3, ascending p value | ID 6063; one total with the native effect label preserved. |
| GCST000057, page 0, size 10, ascending p value | Seven IDs: 11386, 11850, 6056, 11851, 6057, 11852 and 11385. |
| MONDO_0004989, child traits true, page 0, size 3, ascending p value | 2,422 total, distinct from the 2,094 without children. |
| FGFR2, extended gene set true, page 0, size 3, ascending p value | 223 total, distinct from the standard 212. |
| rs2981582 plus PMID 17529967, page 0, size 3, ascending p value | Only association 6110. |

Self links reorder query parameters and use form-encoded spaces. Admission therefore compares the exact endpoint and decoded parameter map, rejects duplicate parameters, and checks source page metadata and returned identities before cache publication. It does not use substring URL matching.

### Measured command response ceilings

Job `j14693` replayed all 22 retained requests through CLI and MCP at the round-trip-safe JSON parser setting. Native source bodies were unchanged. The values below include supported page, variant and publication actions; association and study get actions remain withheld until their handlers land. These outputs do not claim a pinned catalog snapshot.

`tests/gwas-search-budgets.yaml` owns each measured pair `[full CLI bytes, comparable MCP bytes]`. Full CLI bytes include the final newline; MCP bytes are one less. Comparable bytes remove only `use`, `terms` and `terms_url` from validated source outcomes. The external case generator applies `max_bytes` to comparable CLI bytes and `max_tokens` to comparable MCP bytes; both use the measured size plus 15 percent. The retained native Trial separately guards the full response ceiling, so a full ceiling is not confused with a terms-subtracted case ceiling.

| Witness | Full CLI bytes | Comparable MCP bytes | Full CLI ceiling | Comparable CLI ceiling |
| --- | --- | --- | --- | --- |
| `broderick` | 1056 | 932 | 1215 | 1073 |
| `children` | 2321 | 2197 | 2670 | 2528 |
| `extended` | 2308 | 2184 | 2655 | 2513 |
| `gene-negative` | 491 | 367 | 565 | 424 |
| `gene-positive` | 2289 | 2165 | 2633 | 2491 |
| `gene-trait-positive` | 2341 | 2217 | 2693 | 2551 |
| `pubmed-first` | 2254 | 2130 | 2593 | 2451 |
| `pubmed-negative` | 484 | 360 | 557 | 416 |
| `pubmed-second` | 2032 | 1908 | 2337 | 2196 |
| `samani` | 3967 | 3843 | 4563 | 4421 |
| `stacey` | 1555 | 1431 | 1789 | 1647 |
| `study-negative` | 488 | 364 | 562 | 420 |
| `study-page-one` | 2033 | 1909 | 2338 | 2197 |
| `study-positive` | 2257 | 2133 | 2596 | 2455 |
| `tomlinson` | 1082 | 958 | 1245 | 1103 |
| `trait-id-negative` | 489 | 365 | 563 | 421 |
| `trait-id-positive` | 2298 | 2174 | 2643 | 2502 |
| `trait-text-negative` | 504 | 380 | 580 | 439 |
| `trait-text-positive` | 2298 | 2174 | 2643 | 2502 |
| `variant-negative` | 488 | 364 | 562 | 420 |
| `variant-positive` | 2230 | 2106 | 2565 | 2424 |
| `variant-publication` | 1071 | 947 | 1232 | 1091 |

## Agent questions

1. **What do I ask first?** Ask for associations by the exact variant, trait, study, publication or source-defined gene selector. A literal trait query remains literal. It is never expanded into a diagnosis or inferred disease hierarchy.
2. **What lets me choose?** A search row carries the source association ID, study, trait, allele and reported statistic. It carries enough information to choose one source assertion without another identity call.
3. **What comes next?** Get the association or study, then select ancestry/sample context, a variant, the publication or the next association page. The output names those actions explicitly.
4. **Which IDs must survive?** All association IDs, study accessions, rsIDs, mapped ontology IDs and PMIDs survive unchanged. A gene pivot needs an exact stable source join, not a locally matched label.
5. **How does disagreement appear?** Preserve separate source records and their context. A later study or a different ancestry may report another effect. Do not combine them or choose the smaller p value as clinical truth.
6. **What does zero mean?** A successful source page with zero records means that this source query returned no rows. A failed request is failed. Neither establishes no association in nature, benignity or lack of risk.
7. **What controls latency?** One bounded API page controls a search. Warm reply reuse follows the existing source cache. Native request timings and output projections are measured separately from product latency; no product latency is claimed by a design projection.
8. **What changes from 0.9?** Preserve the ability to retrieve mapped-gene and trait associations, bound disclosure and retain effect context. Add exact association/study identity. Replace incomplete prefix intersections with a reviewed source query or an honest unsupported combination.
9. **When may I stop?** Stop after the selected source assertion answers the retrieval question with its allele, effect type, study and publication. Do not fetch ancestry, every association or full summary statistics unless the question needs them.
10. **What makes the assertion trustworthy?** Show publisher identity, original study/publication, checked identifiers, retrieval date, source status and terms. Report the source's statistic without a clinical interpretation.

## Legacy surface and public issues

The static inventory reads 0.9 source revision `4050fdc7dbdf4d713d94cf02a39a147b863665be`. Command declarations, dispatch, source adapter, entity mapper and user guide were read together. The guide's suggestion that gene and trait can be combined does not prove the old prefix intersection was complete.

| 0.9 surface | Options and sections | Proposed disposition |
| --- | --- | --- |
| `search gwas` | Positional query alias, `-g`/`--gene`, `--trait`, `--p-value`, `-l`/`--limit`, `--offset`. | Keep mapped-gene and literal trait access, bounded pages and the finite p-value selector's useful capability. Review exact source-supported combinations and threshold semantics before implementation; do not claim local first-50-row filtering as a complete answer. |
| `get variant ID gwas` | `gwas` section, also selected by `all`; shared variant get options. | Already owned by variant. Retain exact-rsID behavior and source status. Reuse its adapter; do not duplicate ownership or count this as a new GWAS leaf. |
| Generic `batch` | Variant batch with shared section selector. | Shared ownership. No new GWAS batch command or hidden multi-ID fan-out. |
| JSON/Markdown and MCP | Shared `--json`, cache/source policy and transport options. | Shared envelope and existing Markdown/MCP path. They do not multiply the command-specific option count. |

The old search maps either gene or trait to one v2 request sorted by p value. A combined gene/trait search fetches two independent 50-row prefixes and intersects them locally. That bounded-prefix result cannot establish completeness for the source intersection. The old p-value filter applies locally to fetched rows. Exact p-value parts are preserved by the old mapper when supplied. New types must retain that precision and cannot replace underflow with a literal probability of zero.

The proposal retains the positional query as the documented `--gene` alias and preserves the `-g` and `-l` aliases; it does not infer a trait from their text. `--trait` remains literal publisher trait text. `--limit` and `--offset` become the checked source/retained page view. `--p-value` retains an explicitly scoped numeric filter as described above. Proposed new flags are `--variant`, `--trait-id`, `--study`, `--pubmed`, `--include-child-traits` and `--extended-geneset`; the latter two default to false and map to the singular native child-trait and extended-gene-set parameters. Every new long option needs its own positive case. The landed implementation and the accounting below now name each kept command, option and alias.

All 57 closed public issues were enumerated through GitHub. The GWAS-relevant bodies were read. [Issue 283](https://github.com/genomoncology/biomcp/issues/283) uses a GWAS call to expose Windows cache-write stdout corruption. Keep the shared clean JSON-RPC stream behavior; add GWAS coverage to the shared Windows/MCP proof when this source writes cache. This is not a source-specific new subprocess mechanism. [Issue 78](https://github.com/genomoncology/biomcp/issues/78) includes GWAS disease associations as an enrichment-library use. Source association retrieval is kept; local enrichment analysis, other organisms and installation/execution of another tool are outside the amended entity-access scope. Neither issue authorizes a new analytic command.

No 0.9 binary is executed until operating-system containment and a local delivery witness establish that it cannot contact a provider. Without that proof, its timings and byte costs remain unmeasured. Static accounting is not runtime comparison. The proposed design must not claim clinical or output parity from static code reading.

## Outside-in case plan

Every command above needs a known-positive case anchored to a direct request and original publication. CLI and MCP share the same expected envelope and next actions. Cases cover exact study and association identity, multiple studies per PMID, multiple rsIDs/effect alleles per association, multiple ontology mappings, distinct reported/mapped genes, unknown frequency text, optional effect values, exact p-value underflow, default and explicit page sizes, continuation with every selector, terminal pages and source ordering.

Every filter needs its own positive and negative boundary. Combined filters require a source-known intersection outside both independent early prefixes. Trait-child inclusion and extended gene sets require distinct captures that prove their different semantics. Numeric thresholds require rows on both sides, missing values and underflow, with the searched population and truncation made explicit. Invalid limits/offsets/IDs fail before I/O. There is no fuzzy query repair or name-based disease join.

Failure cases cover 429, 500, timeout, malformed JSON, HTTP 200 wrong identity, duplicate association IDs, incorrect page metadata, missing required arrays, valid empty, expired continuation, source disabled and restricted sources disabled. Synthetic failures prove handling; an operator must not trigger provider failure deliberately. Terms/source policy apply before cache or network. Warm reuse, explicit fresh, stale partial status and unchanged source-bound continuation are separate stories. No absent catalog entry becomes a negative clinical finding.

## Ian's questions

The PM decides the product forms and scope through design review. The source contracts and measurements below supply the trade-offs. No clinical or configuration question needs Ian. A material expansion beyond entity access, a new paid service or publication still needs the existing approval path.

The recommendation for PM review is the API-only command tree above, a three-association default page, exact study and association gets, and explicit ancestry and locus sections. The measured largest three-row projection is 2,837 bytes; adopting every candidate bundle would transfer at least the four server-reported object lengths above and add storage/indexing work without answering a demonstrated additional question. Keep the current restricted source policy and the existing adapter deadline. Review the page-scoped `--p-value` semantics explicitly; its lower request cost trades away a claim of complete threshold enumeration. Clinical source-label discrepancies and the PMID 18438407 gap remain visible limitations. The contained 0.9 runtime comparison remains unmeasured and must not be treated as passed.

### Exact positive thresholds

The exact positive-threshold command supersedes the earlier page-scoped proposal. It compares the retained native mantissa and exponent, includes equality, and stops at the first above-threshold row or the source end. A threshold of zero is a usage error before source access, as PM ruling m6762 answer 1 requires. The source total stays in `page.total`; `result.qualifying_total` is null until the walk proves its end. A retained continuation binds every selector, the native order, threshold and page size. It rejects duplicate identities, decreasing p values and changed source totals across pages. The retained pool keeps its original lifetime and offers an explicit restart when absent or mismatched.

The native proof reuses the exact size-three publication and study pages from the existing investigation and selector archives. It makes no source request. Per PM ruling m6762 answer 2, the threshold between associations 6112 and 6113 returns three rows with an unknown qualifying total first, then zero rows with qualifying total three. Synthetic command cases cover equality, underflow, usage, out-of-order pages, duplicate identities and later-page source failures.

The evidence store holds `data/biomcp-redux/0215-gwas-threshold/product-ff179f837f0200d0fa1c72c5fd3e19d8a5608f96.tar.gz`, SHA-256 `97c18347e6be2d189fa7f3d9e94d5996020f2ff7b1f281ee32af4e7e131549f6`. The job verified the transferred manifest before scratch cleanup. Each file is a schema-validated product response from retained source replies. The initial command steps also prove CLI/MCP parity; a synthetic two-page walk proves continuation parity. Full bytes include the CLI newline. Comparable bytes use the shared response measurement. `tests/gwas-threshold-budgets.yaml` owns the measured values and the runner applies a fifteen-percent ceiling to full and comparable bytes. Every external case step has a comparable byte and estimated-token ceiling.

| Native response | Full bytes | Comparable bytes | Full byte ceiling | Comparable byte ceiling |
|---|---:|---:|---:|---:|
| `pubmed-all-first` | 2,648 | 2,524 | 3,046 | 2,903 |
| `pubmed-all-last` | 2,347 | 2,223 | 2,700 | 2,557 |
| `pubmed-below-first` | 760 | 636 | 874 | 732 |
| `pubmed-crossing` | 801 | 677 | 922 | 779 |
| `study-all-first` | 2,651 | 2,527 | 3,049 | 2,907 |
| `study-all-last` | 2,348 | 2,224 | 2,701 | 2,558 |
| `study-below-first` | 761 | 637 | 876 | 733 |
| `study-crossing` | 802 | 678 | 923 | 780 |
| `pubmed-crossing-first` | 2,654 | 2,530 | 3,053 | 2,910 |
| `study-crossing-first` | 2,657 | 2,533 | 3,056 | 2,913 |

### Exact record and context response costs

The exact get commands use one association or study resource and never enrich a card by fetching its study, a publication or a gene. The locus and ancestry relations retain native row self links. The ancestry command admits one whole unpaged source collection and stores that received collection through the existing snapshot store. Its page actions retain the checked study, section size and snapshot token. The source receives no invented page-size parameter. Study associations reuse the checked native ascending p-value selector and preserve their study in every continuation.

Job `j14840` captured eleven exact keyless requests once each with the existing adapter deadline, a 3 MiB body bound, one second between admissions, zero redirects and zero retries. All returned HTTP 200. Its original transfer wrapper used the source-machine scratch path on the destination; transfer-only continuation `j14846` verified the archive manifest on Yellow without another request. The source archive is `data/biomcp-redux/0218-gwas-get/native.tar.gz`, SHA-256 `d83af0fc0c0f6d456e634860ae693a0b8c1c7053e3b6269af7acc7c820355d72`.

Job `j14895` replayed all eleven native responses through CLI and MCP, checked their schemas and parity, and followed the actual study-association and local ancestry page commands without gaps or repeated row identities. Its product archive is `data/biomcp-redux/0218-gwas-get/product-80ace01568729171889f0415461681ffa014fa6b.tar.gz`, and its SHA-256 is `fd890eb070b182c8cdde73f1273bb650a8f09208487c529998509c8b1f002b68`. The archive manifest was verified on Yellow in that job. Native source bodies and product responses remain outside Git.

`tests/gwas-get-budgets.yaml` owns each pair of measured full CLI bytes and comparable MCP bytes. The runner applies fifteen-percent ceilings to both full and comparable bytes and estimated tokens. Full CLI bytes include its final newline. Comparable bytes remove only the three publisher terms fields through the shared measurement helper.

| Response | Full CLI bytes | Comparable MCP bytes | Full byte ceiling | Comparable CLI ceiling |
|---|---:|---:|---:|---:|
| `association-6114` | 1130 | 1006 | 1300 | 1159 |
| `association-6063` | 1150 | 1026 | 1323 | 1182 |
| `association-11386` | 1141 | 1017 | 1313 | 1171 |
| `loci` | 795 | 671 | 915 | 773 |
| `study-GCST000035` | 1561 | 1437 | 1796 | 1654 |
| `study-GCST000037` | 1743 | 1619 | 2005 | 1863 |
| `study-GCST000113` | 1336 | 1212 | 1537 | 1395 |
| `study-GCST000057` | 1356 | 1232 | 1560 | 1418 |
| `ancestries` | 2390 | 2266 | 2749 | 2608 |
| `associations-0` | 2100 | 1976 | 2415 | 2274 |
| `associations-1` | 1860 | 1736 | 2139 | 1998 |

The routine collection in `j14903` measured whole-tree generated help at 76,572 bytes after adding the exact get grammar. Its standing measured-size-plus-fifteen-percent formula sets the help fixture ceiling to 88,058 bytes. This is whole-tree command help rather than a source response or a per-command disclosure budget.


## Implemented legacy accounting

<!-- parity-accounting -->
| 0.9 leaf command | Command outcome | Ported or renamed options | Dropped options | Deferred options | Undecided options | Ruling |
| --- | --- | --- | --- | --- | --- | --- |
| `search gwas` | Ported or renamed | `--gene`, `--trait`, `--p-value`, `--limit`, `--offset` | — | — | — | Native mapped-gene and literal mapped-trait selectors replace local prefix intersection. Exact positive thresholds require default native p-value-ascending order. Positional gene, -g and -l aliases remain. |
<!-- /parity-accounting -->

| Legacy surface | 1.0 spelling or disposition | Owning proof |
| --- | --- | --- |
| `search gwas` | `search gwas` | `gwas_search.rs`; retained search case names above. |
| Positional query | Kept as mapped-gene selector, equivalent to `--gene`. No trait inference. | `checked-native-selectors-and-aliases` |
| `-g`, `--gene` | Kept unchanged. | Same selector case. |
| `--trait` | Kept as literal native `efo_trait`, excluding `reported_trait` prose. | `trait-text-positive`; `native-trait-id-is-never-rewritten` |
| `--p-value` | Kept as inclusive exact positive threshold with native ascending order and qualifying total known only at the end. | `exact-threshold-page-walk`; retained threshold cases. |
| `-l`, `--limit` | Kept as bounded native page size. | Selector alias and paging cases. |
| `--offset` | Kept as native row offset and retained continuation window. | `native-offset-and-continuation` |
| Combined gene/trait prefix intersection | Retired. Native conjunction replaces intersecting two truncated local prefixes. | `gene-trait-positive`; checked native selector case. |
| `get variant ID gwas` | Kept by variant accounting; `gwas` section uses exact rsID access. | `variant/population/catalog-data.yaml` and `catalog-zero.yaml` |
| `get variant ID all` selecting GWAS | Kept by variant accounting with shared section paging and source policy. | Variant section tests. |
| Shared variant-get options | Owned by variant accounting; no duplicate GWAS option count. | `slices/variant.md` |
| Generic variant `batch` with GWAS section | Shared ownership; no new GWAS batch or hidden fan-out. | Shared batch accounting. |
| JSON, Markdown, MCP and cache/source/transport flags | Shared envelope and source policy; JSON is default and Markdown uses `--format md`. | `cli-mcp-help-markdown`; warm/fresh and source-policy cases. |

The new selectors `--variant`, `--trait-id`, `--study`, `--pubmed`, `--include-child-traits` and `--extended-geneset` are covered by the named retained positive search rows and the checked synthetic selector case. New exact association/study gets and `loci`, `associations` and `ancestries` sections have their retained cases above. The four release bundles, `sync gwas`, summary-statistics files, interval search, other GWAS sources and a GWAS batch remain deferred or dropped for the reasons in Source and bundle disposition. Issue 78 enrichment remains outside scope. Issue 283 continues to use the shared clean JSON-RPC stream proof and now includes a synthetic GWAS cache-writing call.


## Implemented synthetic and help response costs

The bounded synthetic collection in `j14978` and its two expiry-case continuation checks in `j14981` measured the CLI and MCP command phases. `tests/gwas-synthetic-budgets.yaml` owns all 107 named phase pairs. Each call site names its phase; loops retain their largest measured page. The shared checker requires a known phase and applies fifteen-percent ceilings independently to complete CLI and MCP envelopes. MCP checks reuse the checked reply without another provider request. Existing CLI tests continue to own explicit-fresh request counts. Native output measurements and ceilings remain unchanged.

Job `j14984` measured the YAML examples and help through both surfaces after composing accepted study help. These are synthetic replies, not publication evidence. Full CLI bytes include the newline. The case runner's `max_bytes` applies to comparable bytes after validating and removing the three publisher terms fields. This differs from the full-envelope phase checker above. Each YAML step now applies the standing measured-size-plus-fifteen-percent rule. The historical binary remains unmeasured under the containment deferral to 0098 for all synthetic comparisons.

| Case step | Full CLI bytes | Full MCP bytes | Comparable CLI bytes | Comparable MCP bytes | Case byte ceiling |
| --- | ---: | ---: | ---: | ---: | ---: |
| `examples` publication | 1712 | 1711 | 1589 | 1588 | 1828 |
| `examples` variant | 1718 | 1717 | 1595 | 1594 | 1835 |
| `examples` threshold | 2091 | 2090 | 1968 | 1967 | 2264 |
| `get-examples` association | 863 | 862 | 740 | 739 | 851 |
| `get-examples` study | 845 | 844 | 722 | 721 | 831 |
| `help` search | 2360 | 2359 | 2360 | 2359 | 2714 |
| `help` get | 1406 | 1405 | 1406 | 1405 | 1617 |

The same job measured whole-tree CLI help at 77,167 bytes, the MCP command description at 2,750 bytes and the source-schema enum at 659 bytes. Whole-tree help and the source enum use measured-size-plus-fifteen-percent ceilings. MCP discovery keeps its existing ceiling. The first compact form measured 2,337 bytes in `j14989`. Shortening the shared paging legend and the adjacent default-order wording removes 28 bytes without removing an option, section or interpretation boundary. The instrumented cases in `j14992` measured the resulting description at 2,309 bytes and passed the existing 2,311-byte allocation. Whole-tree help and discovery are shared command infrastructure; they are not GWAS source-response budgets. The source enum size and its existing ceiling are unchanged.
