# Trial slice

This design follows steps 1 through 4 of [the slice method](../slices.md). It proposes the trial slice for milestone 2. It changes no product code. The investigation starts from `7a7ce2a0f09b5981353753e2f1c94bb39484b06c`.

The central contract is direct retrieval. BioMCP echoes the exact source query, preserves the registry's order and total, returns trial headers, and offers bounded sections. A text match discovers a trial. It never proves that a person meets its eligibility criteria. No local rule classifies inclusion, exclusion, mutation relevance, treatment line, or suitability.

## Evidence and limits

ClinicalTrials.gov captures use its public v2 API without a key. Each request has a bounded timeout, runs serially, and leaves a raw response and a request manifest. Measurements identify the host, executable, source revision and capture date. These are observations of one source snapshot, not permanent counts or speed promises.

A fresh clinical reviewer checked the question bank below against primary publications. Its confirmed answers describe each publication's population, intervention, comparison or endpoint. Current recruitment, locations and search counts come from the dated registry captures. Differences between a paper and a later registration remain attributed to their respective sources and dates.

No dedicated clinical ruling file was named in the assignment or found in this repo. The review is retained here beside the questions it governs. The reviewer changed no product code. Neither the public question bank nor its answers use the curation team's licensed answer bank.

## Questions and accepted answers

Questions are adapted from publications. They are not quotations. Mechanical followups are marked. Each row names the exact trial to preserve through search, get and section requests. The acceptance target is retrieval of the stated source facts, not a patient eligibility judgment.

| Case | Adapted question | Accepted answer | Source |
| --- | --- | --- | --- |
| T01 | Which trial compared first-line osimertinib with gefitinib or erlotinib in advanced NSCLC with EGFR exon 19 deletion or L858R? | FLAURA, NCT02296125. Do not broaden the two named entry alterations to all EGFR alterations. | [Soria 2018, PMID 29151359](https://pubmed.ncbi.nlm.nih.gov/29151359/) |
| T02 | What did FLAURA require regarding prior treatment for advanced disease? | NCT02296125 studied previously untreated advanced NSCLC. Return the complete registry eligibility text, including exceptions. | [FLAURA](https://pubmed.ncbi.nlm.nih.gov/29151359/) |
| T03 | What phase and primary endpoint did FLAURA report? | Phase 3; investigator-assessed progression-free survival. Preserve each registered primary outcome and its time frame. | [FLAURA](https://pubmed.ncbi.nlm.nih.gov/29151359/) |
| T04 | Which trial tested pembrolizumab with pemetrexed and platinum for previously untreated metastatic nonsquamous NSCLC? | KEYNOTE-189, NCT02578680. A pembrolizumab monotherapy trial is a different comparison. | [Gandhi 2018, PMID 29658856](https://pubmed.ncbi.nlm.nih.gov/29658856/) |
| T05 | What EGFR and ALK restriction did KEYNOTE-189 report? | NCT02578680 enrolled a population without sensitizing EGFR or ALK mutations. A keyword occurrence does not establish a positive mutation requirement. | [KEYNOTE-189](https://pubmed.ncbi.nlm.nih.gov/29658856/) |
| T06 | What were KEYNOTE-189's primary outcomes? | Overall survival and progression-free survival. Preserve radiologic review wording where the registry supplies it. | [KEYNOTE-189](https://pubmed.ncbi.nlm.nih.gov/29658856/) |
| T07 | Which trial studied trastuzumab deruxtecan in HER2-low metastatic breast cancer, and how did its publication define HER2-low? | DESTINY-Breast04, NCT03734029; IHC 1+ or IHC 2+/ISH-negative. Do not equate this with any ERBB2 sequence alteration. | [Modi 2025, PMID 41062831](https://pubmed.ncbi.nlm.nih.gov/41062831/) |
| T08 | What was the comparator in DESTINY-Breast04? | Physician's choice of chemotherapy versus trastuzumab deruxtecan. Keep permitted comparator interventions distinct from a combination regimen. | [DESTINY-Breast04](https://pubmed.ncbi.nlm.nih.gov/41062831/) |
| T09 | Mechanical followup: which DESTINY-Breast04 sites does the registry list near New York City? | Use NCT03734029 and an explicit center and radius. Return source site coordinates and each site's status. Never substitute author affiliations, travel time, or a current recruitment claim. | [Publication](https://pubmed.ncbi.nlm.nih.gov/41062831/), [registry](https://clinicaltrials.gov/study/NCT03734029) |
| T10 | Which randomized trial compared sotorasib with docetaxel in KRAS G12C-mutated advanced NSCLC? | CodeBreaK 200, NCT04303780. Do not broaden G12C to all KRAS variants. | [de Langen 2023, PMID 36764316](https://pubmed.ncbi.nlm.nih.gov/36764316/) |
| T11 | What prior treatment did CodeBreaK 200 require in its published population? | Progression after platinum chemotherapy and a PD-1 or PD-L1 inhibitor. Return the registry's exact eligibility wording. | [CodeBreaK 200](https://pubmed.ncbi.nlm.nih.gov/36764316/) |
| T12 | Retrieve NCT04303780 and show its comparison arms. | Exact ID NCT04303780; sotorasib versus docetaxel. Do not substitute CodeBreaK 100 or 300. | [CodeBreaK 200](https://pubmed.ncbi.nlm.nih.gov/36764316/) |
| T13 | Which adjuvant trial studied olaparib in high-risk HER2-negative early breast cancer with germline BRCA1 or BRCA2 variants? | OlympiA, NCT02032823; pathogenic or likely pathogenic germline BRCA1/2 variants. This is not an unrestricted somatic-BRCA or metastatic population. | [Tutt 2021, PMID 34081848](https://pubmed.ncbi.nlm.nih.gov/34081848/) |
| T14 | What treatment duration and comparator did OlympiA report? | One year of oral olaparib versus placebo after its stated local treatment and chemotherapy. This is a study fact, not an individual treatment recommendation. | [OlympiA](https://pubmed.ncbi.nlm.nih.gov/34081848/) |
| T15 | What primary outcome did OlympiA use? | Invasive disease-free survival. Preserve its source definition and time frame. | [OlympiA](https://pubmed.ncbi.nlm.nih.gov/34081848/) |
| T16 | In PROfound, which alterations defined cohort A? | NCT02987543: BRCA1, BRCA2 or ATM. Other prespecified genes formed cohort B. | [Hussain 2020, PMID 32955174](https://pubmed.ncbi.nlm.nih.gov/32955174/) |
| T17 | What disease setting and prior progression did PROfound require? | Metastatic castration-resistant prostate cancer with qualifying HRR alterations and progression during prior next-generation hormonal treatment. A BRCA finding alone does not determine eligibility. | [PROfound](https://pubmed.ncbi.nlm.nih.gov/32955174/) |
| T18 | Mechanical followup: is PROfound recruiting, and what does its registry record say about sites? | Return the captured overall status and update date for NCT02987543. Return site statuses separately. A publication date or address cannot establish current recruitment. | [Publication](https://pubmed.ncbi.nlm.nih.gov/32955174/), [registry](https://clinicaltrials.gov/study/NCT02987543) |
| T19 | Which VISION population was selected using PSMA imaging? | NCT03511664: metastatic castration-resistant prostate cancer after the stated androgen-receptor-pathway inhibitor and taxane treatments, with qualifying gallium-68 PSMA-11 PET/CT scans. A generic PSMA mention does not establish that requirement. | [Sartor 2021, PMID 34161051](https://pubmed.ncbi.nlm.nih.gov/34161051/) |
| T20 | What did VISION compare? | Lutetium-177–PSMA-617 plus protocol-permitted standard care versus standard care alone. The comparator is not unrestricted treatment. | [VISION](https://pubmed.ncbi.nlm.nih.gov/34161051/) |
| T21 | Which primary publication and primary endpoints belong to VISION? | PMID 34161051; DOI 10.1056/NEJMoa2107322; imaging-based progression-free survival and overall survival. Keep registry reference types separate from publication links. | [VISION](https://pubmed.ncbi.nlm.nih.gov/34161051/) |
| T22 | Which first-line colorectal trial compared pembrolizumab with chemotherapy for MSI-H/dMMR metastatic disease? | KEYNOTE-177, NCT02563002; phase 3. Do not broaden its population to all colorectal cancer. | [André 2020, PMID 33264544](https://pubmed.ncbi.nlm.nih.gov/33264544/) |
| T23 | Mechanical followup: does exact NCT02563002 combined with a phase-1 restriction return studies? | Accept zero only after a successful response for the exact conjunction. Do not put a sole NCT ID into a query parameter that causes the source to ignore other filters. | [KEYNOTE-177](https://pubmed.ncbi.nlm.nih.gov/33264544/) |
| T24 | Mechanical failure variant: what should that lookup say if the registry fails? | The source could not answer and the match count is unknown. Never return zero or trial absence after a timeout, rate limit, server error or malformed body. This is a proposed retrieval contract, not an observed outage. | [Trial identity](https://pubmed.ncbi.nlm.nih.gov/33264544/) |

The clinical reviewer labels the publication facts confirmed with high confidence. T09 and T18 require dated registry captures. T23 is a query-semantic check. T24 is a proposed failure contract. A changed protocol or conflicting primary publication requires an attributed discrepancy; it does not authorize a silent correction.

## Source findings and query construction

ClinicalTrials.gov remains the default. [Its API](https://clinicaltrials.gov/data-api/api), [search areas](https://clinicaltrials.gov/api/v2/studies/search-areas), [metadata](https://clinicaltrials.gov/api/v2/studies/metadata) and [enums](https://clinicaltrials.gov/api/v2/studies/enums) are the source of field and enum names. Never invent a field because a CLI flag has a similar name.

The [official OpenAPI specification](https://clinicaltrials.gov/api/oas/v2) declares the `/version` data version UTC in `yyyy-MM-dd'T'HH:mm:ss` format. Its `Version.dataTimestamp` property is a string. Retained native replies contain `2026-10-23T09:00:04` without an offset suffix. The published convention establishes UTC for this provider field. It establishes no timezone for another provider's unsuffixed text. The registry processing timestamp, each study's posted update date and the response retrieval instant describe separate events.

Search uses source parameters for condition, intervention, text, location, overall status and geography. Explicit field restrictions use `filter.advanced`. Fixed construction is echoed verbatim. The source applies its own synonyms and relevance order. BioMCP never expands drugs with chemical-name guesses, rewrites disease abbreviations, infers treatment line, or removes exclusion-only text matches after retrieval.

`--condition` passes literal source text to `query.cond`. A checked disease identifier can be resolved explicitly through MyDisease; ambiguity returns candidates. That resolution must not silently replace an already supplied condition search with its highest-scoring ontology label. `--gene` uses the existing checked gene resolver and echoes the selected source-approved symbol as registry search text. It grants no molecular eligibility claim. A variant is text that the caller explicitly supplies through `--variant`; any shared typed variant normalization follows the variant slice's reviewed contract.

### Disclosure and commands

The proposed command tree keeps one path for each operation:

| Command | Default output | Next step |
| --- | --- | --- |
| `search trial [TEXT]` | One bounded page of headers, source total, exact request and source state | Opaque continuation; `get trial NCT…`; named sections |
| `search trial --count-only` | Source total and exact query, no rows | Search page with identical filters |
| `get trial NCT…` | Identity, title, overall status, phases, study type, conditions, intervention names, sponsor, dates and full brief summary | Available section selectors |
| `get trial NCT… eligibility` | Source age bounds, sex, healthy-volunteer flag and intact eligibility text | Bounded text continuation if necessary |
| `get trial NCT… locations` | Bounded site page, coordinates, site status and all contacts attached to each selected site | Site page continuation |
| `get trial NCT… contacts` | Central contacts plus bounded, explicitly linked site contacts | Locations |
| `get trial NCT… outcomes` | Registered primary, secondary and other outcomes with descriptions and time frames | Bounded outcome continuation |
| `get trial NCT… arms` | Exact arm labels and intervention relationships | Further arm/intervention pages |
| `get trial NCT… references` | Registry publication IDs, citation text and source reference type | `get article PMID` |
| `get trial NCT… documents` | Source document manifest, date, type, size if supplied and exact filename | Explicit file retrieval |
| `trial document NCT… FILE --save PATH` | Bounded bytes saved outside the response and a receipt | Caller chooses its own reader |
| `get trial NCT… all` | Explicitly requested sections with their states and caps | Continuations for every truncated section |

Document retrieval preserves the existing capability without turning the trial slice into a PDF reader. It follows the already reviewed shared binary transport, validates the advertised filename and fixed publisher CDN/redirect policy, and never invents a URL from a filename. Large posted results remain a separate proposed section if the source capture proves the fields and a harvested question needs them. Registered outcomes and posted results are different data.

### Proposed types and schemas

`NctId` is a checked identifier with the registry's exact grammar. Parse it once and compare the returned identity with the requested identity before admitting data to cache. A syntactically valid absent record is distinct from invalid input and from source failure.

Domain values belong in `biodata`: `TrialId`, `TrialHeader`, `TrialPhase`, `TrialStatus`, `StudyType`, `TrialAgeBound`, `TrialSex`, `TrialLocation`, `TrialContact`, `TrialOutcome`, `TrialArm`, `TrialIntervention`, `TrialReference` and `TrialDocument`. Reuse existing checked dates, URLs, coordinates, text, counts and pagination values. A source's missing age limit remains an explicit no-limit state; unparseable data preserves raw source text beside an unavailable normalized value. Do not assume a missing phase means phase 1 or a missing site status means recruiting.

Response types hold cards, page metadata, section states and typed next steps. Proposed schemas are `biomcp.trial_page/1`, `biomcp.trial_card/1`, `biomcp.trial_locations/1`, `biomcp.trial_outcomes/1`, `biomcp.trial_arms/1`, `biomcp.trial_references/1` and `biomcp.trial_documents/1`. Generate schemas from one set of types. Follow the existing shared enum macro; maintain no parallel hand-written enum or field registry.

An outcome's time frame stays attached to that outcome. An arm's intervention relationship stays attached to the source arm label or source identifier. A site contact stays attached to its source site. No derived clinical classification enters these types. Site geographic coordinates use an explicit latitude/longitude pair, not a genomic assembly type.

The adapter owns provider raw shapes and maps them once. The query layer owns commands and explicit source capabilities. Search `--id` binds a checked NCT ID in the advanced field expression when another filter is present. An explicitly supplied raw registry query remains literal; a sole NCT ID combined with narrowing options is either represented with that checked field conjunction or refused before a request. It never relies on the source's query-ID shortcut. Fetch owns deadlines, retries, pacing, decoding, cache admission, source policy, record/replay and binary limits. Store owns continuation state only where necessary. Views render the same envelope for CLI and MCP. There is no trial-only transport, retry loop, cache or global state.

### The agent's next questions

1. **What does the agent ask first?** “Find trials for this condition with this exact intervention or text.” A single `search trial` answers it with source totals and one page.
2. **What is the smallest useful header?** Exact NCT ID, complete brief title, overall status, phases, conditions, intervention names, last registry update and a source URL. A header never claims that the trial accepts a patient's biomarker or location.
3. **Which drill-down comes next?** Eligibility, sites, arms and outcomes. The response names those sections as typed next steps and states whether data is present, absent or not established for this source.
4. **Which identities must survive?** NCT IDs, registry reference PMIDs, exact document filenames, arm labels and stable section offsets. Aliases never replace the returned canonical NCT ID.
5. **How are disagreements shown?** Keep source, timestamp and source-native field values. NCI and ClinicalTrials.gov statuses do not become an invented universal “active” state. Publication endpoints and later registered endpoints remain attributed separately.
6. **How does empty differ from failure?** A successful search with `totalCount: 0` is complete and empty. A 400, timeout, 429, 500, invalid body or unsupported filter has a source problem and no asserted count. A key-required NCI source is skipped before any request when its key is absent.
7. **What sets cost?** A search and a detail each need one projected study request. When no fresh source-version metadata is cached, add one small version request for the registry processing timestamp; share it through the ordinary transport cache instead of calling it per result. Eligibility inspection is an explicit detail request, not an invisible per-result verification loop. Warm calls use the shared cache. Warm command timings were not measured in this investigation, so no warm latency claim is made. Timings below describe source calls and the released no-cache command on the captured machine.
8. **What does 0.9 carry that these questions do not need?** Duplicate entity-specific pivots, chemical alias guessing, treatment-line phrase lists and post-retrieval inclusion/exclusion classification. Keep their useful retrieval questions through explicit source text and intact sections.
9. **When can an agent stop?** The search header answers identity/status discovery; the source's complete count answers a count question; an exact get plus one section answers a specific protocol question. More detail is never loaded merely because it exists.
10. **What supports trust?** An exact source query, registration update date, retrieval date, source URL, field provenance, current source state and an explicit warning that text matches are discovery only.

## Accepted design decisions

The PM accepted this design at `15ae48b1`. The table points to [the accepted trial slice rulings](../../decisions/2026-10-24-trial-slice-rulings.md). Those decisions govern the design; implementation belongs to the PM's trial tickets.

| Question | Options and trade-offs | Accepted PM decision |
| --- | --- | --- |
| What should molecular text mean? | Keep distinct legacy biomarker/mutation heuristics; or expose explicit source text and eligibility-text search with no inference. Legacy behavior changes counts after retrieval and can mistake exclusions for requirements. | Keep `--variant` and `--gene` as explicitly labeled discovery inputs; `--criteria` searches source eligibility text. Echo the exact fields and never verify clinical inclusion locally. |
| Should `--age` remain? | Local conversion and post-filtering breaks source-count parity; native source range constraints retain parity but require proved no-limit semantics. | Ruling 2: use the native expression that admits each missing bound as no limit. Malformed values never become no limit. This proves only the structured age filter. |
| Should facility and radius mean the same site? | Independent trial-wide terms may match different sites; a source-native nested location expression can bind them to one site. | Ruling 3: refuse `--facility` together with `--lat/--lon/--distance`. Keep them as separate trial-wide filters and expose site coordinates in `locations`. |
| Should NCI be folded with ClinicalTrials.gov? | A fold requires additional identity/status/conflict rules; named sources preserve each source's own coverage and counts. | Ruling 4: default ClinicalTrials.gov; explicit keyed `--source nci` with native statuses/counts. Disease filters require an NCIt identifier; free text uses an explicit keyword option. No fold or fallback. |
| Should old therapy-line options survive? | Reproduce word lists; or remove semantic claims and offer literal eligibility text. | Ruling 5: retire `--line-of-therapy`, `--prior-therapies` and `--progression-on` with messages naming `--criteria`. |
| Where should bulky detail live? | Full protocol in the default card; or bounded sections with honest continuations. | Header, card, then sections. Keep full criteria and descriptions recoverable without arbitrary sentence truncation. |
| Should documents be readable here? | Add another PDF parsing path; or list metadata and save exact advertised bytes. | Ruling 7: list documents and save exact advertised bytes. Never read document text. |

## Structural risks

The main risks are duplicated field mappings, hidden post-filter state, excessive requests and tests that prove a fake count. Define each source field mapping once; derive schema and help; capture the exact direct query beside every filter's known-positive case; compare the product total with that same captured `totalCount`. Keep source order and total before rendering. Use shared isolation for CLI/MCP cases and source record/replay. Cap sections and body sizes in enforcing code, with truthful continuations. Do not turn this design's illustrative budgets into a second configuration system.

The implementation sequence is types/schema, one source adapter, search/count, exact get, sections and document retrieval, optional NCI, then accounting/help and installed-command proof. Each stage gets outside-in cases from the bank; the PM chooses tickets after reviewing this design.


## Dated source measurements

Captures were made on 2026-10-24 on the M5, without a whole-machine hold. Other jobs ran concurrently. Source probes were serial, keyless and paced at one second between requests. Every source request used a bounded read and a timeout. The request manifests preserve exact URLs, status, bytes, elapsed milliseconds and returned identifiers. There was no load-controlled speed benchmark. Source responses may also benefit from the publisher's own cache.

Jobs `j10332`, `j10339` and `j10344` captured registry metadata, filter positives, exact records and primary publisher documentation. Job `j10362` ran the released executable and the eight question-specific discovery queries. Job `j10371` captured the official OpenAPI documents and exercised one local failure. Setup and execution used `sh -ec`; the scripts recorded expected nonzero product exits explicitly. The failure job asserted a nonzero product exit. No shell pipeline hid a failed command.

### Known-positive filter witnesses

Each row is a successful direct registry request with `countTotal=true`, one page of ten headers and a retained exact parameter map. These captured counts define replay assertions; they are not permanent live expectations. Rows with different fields or expressions are different questions and must not be compared as if they were interchangeable.

| Filter or explicit field | Native request beyond the common condition | Captured total | First NCT ID | Source ms | Reply bytes |
| --- | --- | ---: | --- | ---: | ---: |
| Condition | `query.cond=melanoma` | 3780 | NCT04631731 | 148 | 5668 |
| Explicit discovery text | `query.term=BRAF V600E` | 78 | NCT02968303 | 142 | 5467 |
| Intervention | `query.intr=pembrolizumab` | 399 | NCT04631731 | 147 | 6409 |
| Overall status | `filter.overallStatus=RECRUITING` | 483 | NCT06952400 | 137 | 5697 |
| Phase | `filter.advanced=AREA[Phase]PHASE3` | 222 | NCT07381972 | 147 | 5788 |
| Combined phase | `AREA[Phase]PHASE1 AND AREA[Phase]PHASE2` | 480 | NCT04631731 | 143 | 5857 |
| Study type | `AREA[StudyType]INTERVENTIONAL` | 3143 | NCT04631731 | 140 | 6221 |
| Sex, breast cancer condition | `aggFilters=sex:f` | 16889 | NCT00089973 | 168 | 6704 |
| Age, finite bounds only | `AREA[MinimumAge]RANGE[MIN,18 years] AND AREA[MaximumAge]RANGE[18 years,MAX]` | 619 | NCT01874119 | 138 | 6097 |
| Age, including unstated/no-limit bounds | Each bound uses the preceding range OR its own `MISSING` term | 3647 | NCT04631731 | 161 | 5668 |
| Location text | `query.locn=Boston` | 397 | NCT05355701 | 137 | 6274 |
| Radius | `filter.geo=distance(42.3601,-71.0589,50mi)` | 424 | NCT05355701 | 156 | 6274 |
| Exact facility field | `AREA[LocationFacility]"Dana-Farber Cancer Institute"` | 201 | NCT05355701 | 146 | 6191 |
| Site city and recruitment together | `SEARCH[Location](AREA[LocationCity]Boston AND AREA[LocationStatus]RECRUITING)` | 34 | NCT02896335 | 158 | 6199 |
| Eligibility text | `AREA[EligibilityCriteria]"BRAF V600E"` | 95 | NCT05355701 | 137 | 5506 |
| Lead sponsor | `AREA[LeadSponsorName]"National Cancer Institute"` | 229 | NCT00924001 | 144 | 5960 |
| Sponsor category | `aggFilters=funderType:nih` | 629 | NCT00521001 | 138 | 6040 |
| Updated since | `AREA[LastUpdatePostDate]RANGE[2025-01-01,MAX]` | 1306 | NCT05355701 | 140 | 5731 |
| Updated until | `AREA[LastUpdatePostDate]RANGE[MIN,2025-12-31]` | 2908 | NCT04631731 | 140 | 5514 |
| Posted results | `AREA[ResultsFirstPostDate]RANGE[MIN,MAX]` | 806 | NCT00847522 | 147 | 5478 |

The exact advanced conjunction `AREA[NCTId]NCT02563002 AND AREA[Phase]PHASE1` returned HTTP 200 with zero studies. A deliberately invalid status `OPEN` returned HTTP 400. The two outcomes remain distinct. A numeric all-zero NCT ID was refused with HTTP 400; the syntactically shaped but absent `NCT01811710` returned HTTP 404. Do not assume every eight-digit suffix is an accepted registry identity.

The age expressions are materially different: the finite-bounds expression returns 619 studies, while the expression that admits a missing minimum or maximum returns 3,647. The official API specification provides the finite-range syntax. Source metadata says N/A means no limit. The clinical reviewer confirmed, with high confidence, that an absent bound in a conforming registration means no registered limit for that bound, following the [minimum-age](https://clinicaltrials.gov/policy/protocol-definitions#EligibilityMinAge) and [maximum-age](https://clinicaltrials.gov/policy/protocol-definitions#EligibilityMaxAge) definitions. Admit the documented missing-bound state alongside each finite comparison, preserve absence without inventing numeric limits, and never convert a parsing failure or an unexpectedly omitted required field into no limit. An age-bound match proves only that structured filter, never full eligibility or precedence over the complete text.

A facility query plus a separate trial-wide radius may match different sites. The same-location city/status grouping succeeded with 34 trials. The attempted nested `LocationGeoPoint` `DISTANCE` expression returned HTTP 400. This investigation therefore does not claim that a facility plus radius can be bound to one site by a proved source expression. Recommendation: refuse that combined same-site promise until the source can express it; expose trial-wide radius and intact site coordinates separately. There is no local geographic or clinical post-filter in the proposed product.

For T09, the exact NCT03734029 record plus the New York center `(40.7128,-74.006)` and 50-mile radius returned one trial. That establishes a trial-level geographic match. It does not establish a source-returned subset of sites. The complete record lists 208 sites. The proposed site page preserves coordinates and statuses and makes this distinction explicit.

### The harvested discovery queries

A paper names a known trial; a broad contemporary search need not put that historical trial in its first page. Exact retrieval is independently proved for all eight IDs. These discovery counts preserve the source's meaning and order without asserting clinical relevance for each returned row.

| Cases | Direct source query | Count | First returned ID | ms | Bytes |
| --- | --- | ---: | --- | ---: | ---: |
| T01 and its followups | See exact URL in `question-query-manifest.json` | 66 | NCT05498428 | 146 | 4834 |
| T04 and its followups | See exact URL in `question-query-manifest.json` | 682 | NCT04631731 | 164 | 4807 |
| T07 and its followups | See exact URL in `question-query-manifest.json` | 491 | NCT03923166 | 393 | 4788 |
| T10 and its followups | See exact URL in `question-query-manifest.json` | 141 | NCT04933695 | 167 | 5280 |
| T13 and its followups | See exact URL in `question-query-manifest.json` | 127 | NCT05238831 | 146 | 7941 |
| T16 and its followups | See exact URL in `question-query-manifest.json` | 65 | NCT05238831 | 159 | 6635 |
| T19 and its followups | See exact URL in `question-query-manifest.json` | 201 | NCT06855277 | 146 | 4330 |
| T22 and its followups | See exact URL in `question-query-manifest.json` | 228 | NCT03724851 | 157 | 6119 |

T01/T02/T03 use NCT02296125; T04/T05/T06 use NCT02578680; T07/T08/T09 use NCT03734029; T10/T11/T12 use NCT04303780; T13/T14/T15 use NCT02032823; T16/T17/T18 use NCT02987543; T19/T20/T21 use NCT03511664; T22/T23/T24 use NCT02563002. Each exact source record was captured once and supplies all its mechanical followups. T23 has its own successful empty conjunction; T24 has a separate, explicitly synthetic local HTTP 500 witness.

### Fields proved on the public tier

The eight exact records returned identity, status, design, conditions, intervention/arm relationships, outcome definitions, eligibility text and reference modules. NCT03734029 includes primary, secondary and other outcomes. Its eligibility record supplies minimum age and omits maximum age. The source does not require fabricated null strings or an inferred upper bound. NCT02578680 and NCT02563002 have no locations in these captures; absence does not mean the API lacks site support.

| Registry field path below `protocolSection` | Response ownership |
| --- | --- |
| `identificationModule.nctId`, `briefTitle`, `officialTitle` | Exact identity and title; retain source acronym only when present |
| `statusModule.overallStatus`, `lastUpdatePostDateStruct`, `startDateStruct`, `completionDateStruct` | Card and header provenance; preserve date precision and actual/estimated type |
| `conditionsModule.conditions` | Source condition array, without semantic classification |
| `designModule.studyType`, `phases`, `designInfo`, `enrollmentInfo` | Study design and source enrollment; enrollment is not the number of current openings |
| `eligibilityModule.eligibilityCriteria`, `minimumAge`, `maximumAge`, `sex`, `healthyVolunteers` | Eligibility section; raw criteria stay intact |
| `contactsLocationsModule.locations`, `centralContacts`, `overallOfficials` | Sites, contacts and officials have distinct roles; optional fields stay optional |
| `armsInterventionsModule.armGroups`, `interventions` | Preserve label relationships and source intervention type/other names |
| `outcomesModule.primaryOutcomes`, `secondaryOutcomes`, `otherOutcomes` | Each measure retains description and time frame |
| `referencesModule.references` | Preserve PMID, reference type and citation; do not infer that all are results papers |

NCT02987543 is `COMPLETED`, updated 2023-10-06. NCT03734029 is `COMPLETED`, updated 2026-09-28. NCT02032823 is `ACTIVE_NOT_RECRUITING`. These dated facts answer T18 and illustrate why “active” cannot be translated to “recruiting.”

### MyDisease resolution

[MyDisease's query documentation](https://docs.mydisease.info/en/latest/doc/disease_query_service.html) and its captured metadata support MONDO identities and cross-references. A literal `melanoma` query returned 126 hits; exact label `melanoma`, MONDO:0005105, appeared fourth in the first five, with NCIt C3224. Source rank is therefore not an exact-label identity guarantee. `MF` returned two source candidates, MONDO:0020481 and MONDO:0009691. The program returns those identities and labels as candidates; it does not choose a disease by the context of a trial question.

The metadata uses `mondo.synonym`, not `mondo.synonyms`; the final exact annotation request used the actual singular field. An unrecognized projection must not be treated as evidence that synonyms do not exist. Resolve an explicit MONDO ID once; use only publisher-provided cross-references. For NCI, require an unambiguous NCIt identifier or an explicitly chosen literal keyword path. Do not silently switch an intended disease filter into unrestricted keyword search.

### NCI source boundary

[NCI's current public OpenAPI document](https://clinicaltrialsapi.cancer.gov/api/v2/docs) declares API version 2.2, an `X-API-KEY` header, search, exact get and disease/intervention/organization/biomarker vocabularies. Its [trial field document](https://clinicaltrialsapi.cancer.gov/api/v2/docs/trial.json) names sites, eligibility, arms, outcome measures, disease concepts, biomarker fields and native status. The older [NCI overview](https://www.cancer.gov/syndication/api), updated 2025-03-25, says publicly available information excludes biomarker data. Documentation therefore does not establish a public biomarker payload. A named NCI section remains unsupported or unproved until a public-tier keyed capture supplies its fields; its mere appearance in a schema is insufficient.

One released-product smoke inherited `NCI_API_KEY`, ran one `search trial --source nci --condition melanoma --limit 1`, and exited zero. No credential was printed, logged, hashed, stored or searched for. The product returned total 2,149 and NCT05929768, a breast-cancer trial, with an explicit note that disease grounding carried no NCIt ID and the product used a broad keyword instead. This is evidence of a degraded keyword path, not evidence that the NCI disease filter works or that NCI has 2,149 melanoma trials. No second keyed smoke or unkeyed trial request was made.

The proposed source keeps native NCI statuses and coverage, accepts only explicitly supported filters, and returns unsupported sections distinctly. It never falls back to NCI after a ClinicalTrials.gov failure or calls any keyed source before configuration supplies its key. NCI's documentation describes `keyword` as an OR across the configured text fields and repeated values, so it is not interchangeable with a grounded disease-concept conjunction.

### Terms and source selection

[ClinicalTrials.gov terms](https://clinicaltrials.gov/about-site/terms-conditions) permit access without charge and require attribution, current data, the source processing date and a description of modifications when distributing data. They prohibit asserting proprietary rights and promotional use of extracted email addresses, and warn about international and third-party copyrights. Classify this as publisher terms, not a blanket CC0 grant. Record registry dates and distinguish frozen replay evidence from current live data. The captured version endpoint reports API 2.0.5 and source processing timestamp `2026-10-23T09:00:04`. Preserve that source processing timestamp separately from the record's last update and the retrieval timestamp; a fetched-today claim must not replace it. Contacts appear only for the requested trial and never feed a marketing list.

NCI's OpenAPI points to its [publisher terms page](https://www.cancer.gov/syndication/api). Preserve that source reference and the user's key boundary. MyDisease aggregates separately licensed sources; retrieve only the ontology fields the resolution needs. The captured [MONDO license](https://github.com/monarch-initiative/mondo/blob/master/LICENSE) is CC BY 4.0. Carry its source attribution and do not turn the whole MyDisease aggregate into one inferred license.

Keep ClinicalTrials.gov, explicit keyed NCI and explicit MyDisease resolution. Drop automatic cross-provider fallback, local clinical ranking, inferred eligibility, phrase-list treatment lines and chemical alias guessing. Source outage proof remains a followup if a provider cannot be measured; it never becomes a fabricated zero. No published hard request quota was established for these sources in the captured primary documents. Start with serial requests at one second between admissions as a stated conservative assumption, then use the shared transport to honor source Retry-After and configured limits.

## Released-product comparison

The official v0.9.3 archive was downloaded from `https://github.com/genomoncology/biomcp/releases/download/v0.9.3/biomcp-darwin-arm64.tar.gz`. SHA-256 `e637062c18c6692f3ec49c5847e69b29f6feee7f7909345970bd504d72174994` matches the publisher's adjacent `.sha256`. The extracted executable reports `biomcp 0.9.3` and hashes to `54f66769d9193c569d1b65756ad766eafd1a2c5e92d08f8c855cc8964e9a7145`. Tag v0.9.3 resolves to `5641de7d7b79b53872f8599dbb4dba22e611dcaf`. It was not built from the current source tree. The M5's installed executable reports 0.9.1 and was used only for initial help discovery, never mislabeled as this baseline.

Thirty-two ordinary baseline calls used `--json --no-cache`, a 25-second process limit and serial execution. Their median was 158.5 ms and range 131–2,292 ms. These are elapsed command measurements on a loaded machine, not quiet-machine speed claims. The baseline is keyless except for the single explicitly separated NCI smoke. The later local T24 failure is synthetic: HTTP 500 produced four bounded attempts, exit 1, 220 output bytes and 5,152 ms. It correctly stated source failure and `not_found: false`.

| Question | v0.9.3 result | Consequence for the design |
| --- | --- | --- |
| Recruiting melanoma | 483, matching the direct source | Preserve native status filtering and total |
| Phase-3 melanoma | 222, matching the direct source | Preserve native phase filtering |
| 50-mile Boston melanoma radius | 424, matching the direct source | Keep a trial-wide radius promise distinct from a selected site subset |
| Female breast-cancer count | 16,889, matching the direct source facet | Keep the source's exact facet semantics |
| Age-18 melanoma count | 3,780, marked approximate; direct finite-bound expression 619 or explicit missing-bound expression 3,647 | Replace client post-filter/count mismatch with a reviewed native expression |
| Eligibility text BRAF V600E | Ten returned rows, total unknown; the direct eligibility-field query counted 95 | Drop local inclusion/exclusion verification and retain the source total |
| EGFR exon-19 deletion discovery | Ten rows, total unknown; direct broad source text counted 66 | Keep the exact source field scope visible; do not imply molecular eligibility |
| KRAS G12C discovery | Ten rows, total unknown; direct broad source text counted 141 | Preserve source order/count and full criteria access |
| HER2-low biomarker | 75 from keyword/intervention/condition fields, versus 491 from broad source text | These are different field scopes. Rename the operation honestly; never claim these counts are equivalent |
| NCI melanoma | A degraded keyword path returned a breast-cancer first hit and note, with 2,149 total | Require explicit condition identity or explicit keyword mode; no silent grounding fallback |

The v0.9.3 source audit confirms that `--biomarker` maps only Keyword, InterventionName and Condition, while `--criteria` maps EligibilityCriteria and `--mutation` maps a fixed union of eligibility/title/summary/keyword fields. Prior-therapy, progression and treatment-line options add hand-written phrase lists. Age is applied after discovery, and count-only can skip it. These are explicit legacy mappings and correctness gaps to decide, not clinical interpretations. The decisive wrong-filter witness is T23: `search trial NCT02563002 --phase 1 --limit 1` returned one PHASE3 study with exit zero in 129 ms and 716 bytes, recorded in `j10375`. The official OpenAPI says a query parameter containing only NCT IDs causes filters to be ignored. The successful exact advanced conjunction returned zero. Preserve both raw outputs and reject or explicitly construct that conjunction in the new command. The earlier absent-ID get in the ordinary baseline batch is a separate absence witness, not a substitute for this same-question comparison. No additional wrong-field claim is made merely because a broad query returned an unexpected row.

### Output budgets

The comparison uses actual 0.9 JSON bytes. Proposed 1.0 bytes come from a scratch projection of the already captured source replies into the stated card/section/page shapes. No product code or shipped schema exists for this slice yet. The projection includes source/state/query/next metadata and complete source text for the selected section. Bytes divided by four is only a token estimate; exact enforcing ceilings are measured on the eventual command, then stored in its cases. The proposed payloads and size manifest are retained with the raw evidence.

| Case/section | Actual 0.9 bytes | Actual 0.9 ms | Projected 1.0 bytes | Facts preserved |
| --- | ---: | ---: | ---: | --- |
| card: card | 7276 | 158 | 1262 | Exact identity, title, status, phase, conditions, intervention names, dates, sponsor and whole brief summary |
| T02: eligibility | 11091 | 578 | 3841 | Whole criteria and source age/sex/healthy-volunteer fields |
| T03: outcomes | 15767 | 172 | 8677 | All source outcome entries with their measure, description and time frame |
| T08: arms | 5991 | 131 | 2354 | Arm labels, source intervention names and relationships |
| T09: locations | 7411 | 443 | 2220 | Ten sites with coordinates/status and their full source contact rows |
| T11: eligibility | 4207 | 156 | 1056 | Whole eligibility text and structured source bounds |
| T15: outcomes | 8128 | 157 | 5122 | Every registered outcome and time frame |
| T19: eligibility | 10606 | 132 | 7056 | Whole imaging and treatment criteria text without classification |
| T21: references | 11658 | 191 | 5991 | All registry citations, IDs and source types |

The projection's complete VISION outcome section is 20,836 bytes. It must page by whole outcome records, with a named continuation. The default card projection is 1,035–2,437 bytes across the eight trials; the smallest useful card still preserves the complete brief summary. The illustrative search pages span 3,589–4,540 bytes for ten nonempty rows in the filter bank. Status rows project to 3,920 bytes against 0.9's 4,639, and phase rows to 3,826 against 4,135. At four bytes per token these are about 980 and 957 tokens. Additional source update dates and intervention names explain any individual page that exceeds a narrower legacy row.

Proposed initial case budgets are the measured projection plus headroom for the real envelope: a search page of ten headers 6,000 bytes, card 3,200, eligibility page 8,400, locations page of ten 3,600, outcome page 10,800, arm page 6,000, reference page 10,400, contacts page 4,000 and document manifest page 2,000. Count-only and empty/error envelopes target 1,200 bytes. Explicit `all` uses each section's cap rather than an unbounded full record; section continuations preserve the omitted records or characters. These are design targets; implementation cases replace them with exact measured output plus 15 percent, bounded by comparable 0.9 output unless the responsible added facts justify an exception.

Keep complete eligibility text recoverable when it exceeds one page. Preserve character offsets and source identity across continuation; never split or classify a criterion by recognizing English headings. A zero-location record is an empty section, not a false unavailable source. A source that has never proved a section is unsupported/unproved, not empty.

## 0.9 command, option and section accounting

The accepted [trial rulings](../../decisions/2026-10-24-trial-slice-rulings.md) define these migration answers. Generated `help search trial`, `help get trial` and `help trial document` describe the shipped schemas. Their examples come from executable replay steps. Global flags and aliases lie outside the historical command-specific option denominator. Section values are accounted separately below. The gene slice owns the `gene trials` ledger row.

| 0.9 surface | 1.0 answer | Owning proof |
| --- | --- | --- |
| `search trial` | Search one provider with native counts and order; default ClinicalTrials.gov | `tests/cases/trial/condition.yaml`, `walk.yaml` |
| `get trial` | Exact NCT cards, then requested bounded sections | `tests/cases/trial/get/*.yaml` |
| `batch trial`; `--sections`, `--source` | Multiple NCT inputs and trailing section words on `get trial`; explicit source remains | Multi-input get in `crates/biomcp/tests/runner/trial_get.rs`; retirement in `input-boundaries.yaml` |
| Batch `--mode`, `--pdf` | Dropped for trials; migration message explains these were article-only | `input-boundaries.yaml` |
| `gene trials`; `--limit`, `--offset`, `--source` | `search trial --gene`; bounded limit, opaque `--next-page`, explicit source; NCI refuses this unsupported gene filter | `gene.yaml`, `input-boundaries.yaml`, `nci-boundaries.yaml` |
| `variant trials`; `--limit`, `--offset`, `--source` | `search trial --variant` accepts literal discovery text; no variant identity or eligibility inference; bounded limit and opaque next page; NCI refuses this unsupported filter | `biomarker.yaml`, `input-boundaries.yaml`, `nci-boundaries.yaml` |
| `disease trials`; `--limit`, `--offset`, `--source` | `search trial --condition` searches registry condition text; NCI requires an explicit NCIt identifier; no automatic disease resolution | `condition.yaml`, `nci-search.yaml`, `nci-boundaries.yaml`, `input-boundaries.yaml` |
| `drug trials`; `--limit`, `--offset`, `--source`, `--no-alias-expand` | `search trial --intervention` sends literal source text without inferred aliases; paging uses `--next-page`; NCI refuses this unsupported filter; alias flag retires | `intervention.yaml`, `input-boundaries.yaml`, `nci-boundaries.yaml` |
| Positional query; `--condition` | Positional text searches general registry text; condition filters only conditions. NCI free text requires explicit `--keyword` | `trial_tables.rs`, `condition.yaml`, `nci-boundaries.yaml` |
| `--intervention`, `--no-alias-expand` | Literal intervention filter; no inferred chemical aliases; retired flag explains the default | `intervention.yaml`, `input-boundaries.yaml` |
| `--facility` | Native facility restriction; combining facility and radius refuses because the source cannot bind them to the same site | `facility.yaml`, `input-boundaries.yaml` |
| `--phase`, `--study-type`, `--status`, `--sex` | Checked native registry vocabulary, including combined phases | `phase.yaml`, `combined-phase.yaml`, `study-type.yaml`, `status.yaml`, `sex.yaml` |
| `--age` | Native source range with unstated bounds treated as no registered limit; no eligibility judgment | `age-missing.yaml` |
| `--mutation`, `--biomarker` | Retired spellings name `--variant` for literal discovery | `biomarker.yaml`, `input-boundaries.yaml` |
| `--criteria` | Literal eligibility-field search for discovery; no inclusion or exclusion classification | `eligibility.yaml`, `help.yaml` |
| `--prior-therapies`, `--progression-on`, `--line-of-therapy` | Dropped semantic filters; each message names literal `--criteria` search | `input-boundaries.yaml` |
| `--sponsor`, `--sponsor-type` | Native sponsor field and funding facet | `sponsor.yaml`, `funder.yaml` |
| `--date-from`, `--date-to` | Retired spellings name `--updated-since`, `--updated-until` | `updated.yaml`, `until.yaml`, `input-boundaries.yaml` |
| `--lat`, `--lon`, `--distance` | Native trial-wide radius with explicit units; no same-site facility promise | `distance.yaml`, `input-boundaries.yaml` |
| `--has-results`; alias `--results-available` | Posted-results restriction; alias retires with `--has-results` guidance | `results.yaml`, `input-boundaries.yaml` |
| `--count-only` | Exact source count without detail rows | `count-only.yaml`, `nci-search.yaml` |
| `--source` | ClinicalTrials.gov default; explicit NCI needs the user's key; no fallback or merged statuses/counts | `nci-search.yaml`, `nci-boundaries.yaml`, `runner/nci.rs` |
| Search `--limit`, `--next-page`, `--offset` | Bounded source pages; offset spelling retires with opaque cursor guidance | `walk.yaml`, `input-boundaries.yaml` |
| Get `--source`, `--offset`, `--limit` | Explicit provider and bounded sections; continuations own the checked stored record and mechanical offset | `runner/trial_get.rs`, `runner/trial_sites.rs`, `runner/trial_documents.rs`, `runner/nci_walk.rs` |

Proof paths without a prefix are under `tests/cases/trial/`; `runner/` means `crates/biomcp/tests/runner/`. Trial search and get share the CLI grammar with MCP. Unsupported source filters return typed guidance before source or cache access. NCI accepts condition identities and explicit keyword searches; it does not pretend to implement the registry's other filters.

| 0.9 section | 1.0 answer | Owning proof |
| --- | --- | --- |
| `eligibility` | Intact source text and structured age/sex metadata; ClinicalTrials.gov character continuations preserve every character, while NCI pages whole unclassified source criterion records | `runner/trial_get.rs`, `runner/nci_walk.rs` |
| `outcomes` | Whole native outcome records with measures, descriptions and time frames | Get cases and both source walkers |
| `arms` | Whole native arms and intervention relationships; source-provided intervention names remain intact | Get cases and both source walkers |
| `locations` | Whole original registry sites with attached contacts and coordinates when supplied. NCI reports unsupported | `sites/*.yaml`, `runner/trial_sites.rs`, `nci-get.yaml` |
| `contacts` | Registry central contacts and officials with original roles; independent from site contacts. NCI reports unsupported | `sites/*.yaml`, `runner/trial_sites.rs`, `nci-get.yaml` |
| `references` | Registry publication identities, native types and citation order; article action fetches the page's PMIDs without inferring a primary paper. NCI reports unsupported | `references.yaml`, `runner/trial_documents.rs`, `nci-get.yaml` |
| `documents` | Registry manifest, source URLs and document metadata. NCI reports unsupported | `documents.yaml`, `nci-get.yaml` |
| `document FILE` | `trial document NCT FILE --save PATH` saves only an exact manifest member; never extracts text. CLI and stdio MCP save; HTTP refuses with the URL | `document-examples.yaml`, `runner/trial_documents.rs`, `input-boundaries.yaml` |
| `all` | Every implemented section for the selected source; each truncated section owns its continuation. No section becomes supported merely because all was requested | Get cases and both source walkers |

NCI additionally exposes its native disease concepts and complete source summary through `diseases` and `summary`. These sections refuse ClinicalTrials.gov. NCI biomarker interpretation is not a shipped section. Synthetic keyed wire cases prove that missing keys skip before any request and that configuration reload reaches the next stdio and HTTP call. Public-tier NCI captures prove one NCIt query and its exact get; their recorded count is not keyword-search parity.

Case ceilings enforce measured compact output plus headroom under the accepted budget rule. Comparable released outputs remain the upper bound unless added required provenance has an approved exception. Existing cases retain their source and baseline qualifications. No matching released NCI card or section comparator was established. Complete source walkers compare every omitted record or character to its original capture. Generated help has no comparable 0.9 schema-help baseline; its case ceilings use measured output plus headroom. The MCP discovery case keeps its existing ceiling.

<!-- parity-accounting -->
| 0.9 leaf command | Command outcome | Ported or renamed options | Dropped options | Deferred options | Undecided options | Ruling |
| --- | --- | --- | --- | --- | --- | --- |
| `search trial` | Ported or renamed | `--condition`, `--intervention`, `--facility`, `--phase`, `--study-type`, `--age`, `--sex`, `--status`, `--mutation`, `--criteria`, `--biomarker`, `--sponsor`, `--sponsor-type`, `--date-from`, `--date-to`, `--lat`, `--lon`, `--distance`, `--has-results`, `--count-only`, `--source`, `--offset`, `--next-page`, `--limit` | `--no-alias-expand`, `--prior-therapies`, `--progression-on`, `--line-of-therapy` | — | — | The accepted trial rulings map literal discovery, source selection, bounded sections, saves and explicit retirements. |
| `get trial` | Ported or renamed | `--source`, `--offset`, `--limit` | — | — | — | The accepted trial rulings map literal discovery, source selection, bounded sections, saves and explicit retirements. |
| `batch` | Ported or renamed | `--sections`, `--source` | `--mode`, `--pdf` | — | — | The accepted trial rulings map literal discovery, source selection, bounded sections, saves and explicit retirements. |
| `variant trials` | Ported or renamed | `--limit`, `--offset`, `--source` | — | — | — | The accepted trial rulings map literal discovery, source selection, bounded sections, saves and explicit retirements. |
| `disease trials` | Ported or renamed | `--limit`, `--offset`, `--source` | — | — | — | The accepted trial rulings map literal discovery, source selection, bounded sections, saves and explicit retirements. |
| `drug trials` | Ported or renamed | `--limit`, `--offset`, `--source` | `--no-alias-expand` | — | — | The accepted trial rulings map literal discovery, source selection, bounded sections, saves and explicit retirements. |
<!-- /parity-accounting -->


## Evidence location and review boundary

Raw responses, exact request manifests, released-product outputs, official specification documents, baseline archive/checksum/provenance, proposed JSON projections and the size analysis script live on Yellow at the evidence store under ticket 0079. `SHA256SUMS` covers the retained files. The local failure is labeled synthetic and remains separate from actual provider replies. The source question bank and design are the only new Git content; raw captures and executables stay outside Git.

The clinical review covered all 24 questions and the bounded age-metadata clarification. It confirmed publication facts and the no-limit convention, kept recruitment and count claims tied to captured registry data, and rejected clinical eligibility conclusions from text search or a structured age match. The investigator read both results before using them.

The investigation did not compile product code, build release artifacts, change production or schedule source refreshes. The implementation proof above covers the accepted trial capabilities. A same-site facility-plus-radius filter remains unsupported by ruling. Implicit disease-name resolution and interpretation of NCI biomarker fields remain outside these shipped capabilities. Explicit `--condition-id MONDO:…|DOID:…` uses exact MyDisease annotation and sends its primary label unchanged to ClinicalTrials.gov. NCI accepts `--condition-id NCIT:C…` directly or one unique source-provided NCIt mapping. The response retains the input, resolver, returned concept, mapping choices and actual source request. Missing mappings, ambiguous mappings and resolver failures stop before trial I/O. Missing NCI credentials skip before the resolver. Literal `--condition` is unchanged; it cannot combine with `--condition-id`, and NCI keyword selection also conflicts. This new option adds no historical 0.9 option slot.

### Exact condition identity output measurements

The retained-body command proof reuses the original MyDisease projection and published trial recordings without a provider call. Local record/replay sends the returned MONDO:0005105 primary label `melanoma` unchanged to ClinicalTrials.gov and the unique native NCIt C3224 mapping to NCI. Duplicate C3224 rows retain their separate MONDO and Disease Ontology provenance. DOID:3910 resolves to MONDO:0005061, but its NCIt targets C27745 and C3512 remain ambiguous and stop before NCI. MONDO:0800444 has no native NCIt mapping and also stops. Explicit `NCIT:C3224` bypasses MyDisease. These are discovery filters and establish no eligibility claim.

| Exact command shape | Full bytes | Comparable bytes | Estimated tokens | Comparable byte ceiling |
| --- | ---: | ---: | ---: | ---: |
| MONDO:0005105 to ClinicalTrials.gov | 6,194 | 5,893 | 1,474 | 6,777 |
| MONDO:0005105 to NCI | 2,910 | 2,632 | 658 | 3,027 |
| DOID:3910 ambiguous NCIt mappings | 2,143 | 2,015 | 504 | 2,318 |
| MONDO:0800444 missing NCIt mapping | 994 | 866 | 217 | 996 |
| Explicit NCIT:C3224 to NCI | 1,727 | 1,577 | 395 | 1,814 |
| Explicit NCIT:C3224 with rejected NCI reply | 926 | 776 | 194 | 893 |

Comparable bytes subtract only the validated source outcome terms fields under the shared rule. Component terms and mapping provenance remain included. Token figures use `ceil(bytes/4)`, not a tokenizer. New case ceilings use each measured comparable output plus 15 percent. The older literal `--condition melanoma` case and its cap stay unchanged. No equivalent 0.9 explicit identifier-resolution command measurement exists, so no same-command reduction is claimed. Original capture metadata stays separate from each local delivery recording outside Git.

### Repeated source filters

ClinicalTrials.gov accepts Boolean expressions in `query.cond`, `query.intr` and `query.term` through its [v2 search API](https://clinicaltrials.gov/data-api/api). Repeated condition, intervention and keyword options retain each original value inside parentheses and join those values with OR. Different options combine with AND. A keyword group also preserves the positional query's Boolean grouping. Exact condition search quotes each original phrase in its own `AREA[Condition]EXPANSION[None]` clause and groups the OR clauses before other advanced filters. The mismatch notice evaluates each condition value separately and never removes a source row. Existing replay recordings retain their original publisher response bytes and capture times; their request identities migrate to the equivalent parenthesized queries. Those replay snapshots are not new live captures.

NCI's [Clinical Trials Search API](https://clinicaltrialsapi.cancer.gov/) receives repeated `diseases.nci_thesaurus_concept_id` parameters for condition alternatives. Each positive proof retains native disease codes from its returned trial, so an ignored parameter cannot qualify through a positive count alone. NCI continues to accept one native keyword and refuses repeated keywords and intervention filters. The typed query's `MAX_ALTERNATIVES` bounds each list; condition identity resolution remains one `--condition-id` input.

### Exact NCI biomarker filters

NCI's [trial schema](https://clinicaltrialsapi.cancer.gov/api/v2/docs/trial.json) lists `biomarkers.nci_thesaurus_concept_id`, `biomarkers.eligibility_criterion` and `biomarkers.inclusion_indicator`. The [API reference](https://clinicaltrialsapi.cancer.gov/api/v2/docs) permits filtering by schema fields but does not state that separate biomarker parameters bind one nested entry. Trial search sends the exact code and optional native criterion, requests `include=biomarkers`, and retains all entries with that code in source order. Missing or malformed entries and a requested criterion absent from the code's own entries produce field problems without removing or moving a trial. `inclusion_indicator` retains the source value without interpretation. Literal variant text and descendant searches remain separate.

The bounded 2026-10-29 capture confirms public-tier biomarker payloads. C96022 returns 324 trials. The current biomarker endpoint restricted to C96022 and `inclusion` reports count 318, equal to the trial endpoint restricted to that code and criterion. Both returned trial entries publish `inclusion` and `TREE`. The intentionally wrong field `biomarkers.codes` returns zero. The earlier biomarker count of 326 is a dated source observation; comparisons use the current code and criterion observation rather than assuming a fixed live total.

Physical evidence remains outside Git on Yellow under `data/biomcp-redux-evidence/0411/`. The first archive `native-capture.tar.gz` has SHA-256 `c394d30e26b6520add79b24797154a3f261cb961bfb08fc6a5f555da4376c615`; it preserves two successful responses and the subsequent 429. The one bounded continuation `native-retry.tar.gz` has SHA-256 `89f2b3a89007d2c1003ec19d6663a0ab569cd6e4a9d4779bf12f8a83471aa5d1` and preserves the three successful remaining calls. Published plans retain deadlines, body limits, pacing and selectors. Replay cases retain native response bodies under the product's credential-free request identities.

### Registry publication value grammar

The [ClinicalTrials.gov OpenAPI 2.0.5](https://clinicaltrials.gov/api/oas/v2), checked on 2026-10-29, defines `Reference.type` with `ReferenceType`: `BACKGROUND`, `RESULT` and `DERIVED`. `LargeDoc.uploadDate` references `DateTimeMinutes`, whose documented spelling is `yyyy-MM-dd'T'HH:mm`. `TrialReferenceKind` retains the native codes. `TrialUploadTime` checks that grammar and the calendar day without adding seconds or a timezone. Bad optional metadata produces a field problem while retaining its publication or document row.

`Text::MAX_CHARS` bounds checked text after trimming or visible source escaping. Direct construction and deserialization reject unsafe controls. Native human text uses the adapter admission wrapper and shared mapping helpers to render controls visibly through `Text::from_source` before construction. Identity, protocol, URL, date and native code parsing remain strict. NCI biomarker `eligibility_criterion` and `inclusion_indicator` retain their native code bytes; surrounding spaces cannot turn a source code into a filter match. Source Markdown retains LF, CR and tab formatting, escapes other unsafe controls and checks its actual retained character count against `Text::MAX_CHARS` through its own schema. `SourceText` retains its existing exact source-value contract.

### Country discovery

Country filters use the exact labels in ClinicalTrials.gov's [LocationCountry field statistics](https://clinicaltrials.gov/api/v2/stats/field/values?fields=LocationCountry), retrieved on 2026-10-29. The retained metadata's unique value count equals its complete published value list. The query owns one derived vocabulary file and preserves each label's whitespace. Unknown labels fail with the country argument named before source access. Repeated countries combine with OR. ClinicalTrials.gov receives quoted `AREA[LocationCountry]` alternatives in `filter.advanced` and returns the country projection for source response validation. Trial headers retain their existing shape.

NCI's [OpenAPI reference](https://clinicaltrialsapi.cancer.gov/api/v2/docs), retrieved on 2026-10-29, explicitly demonstrates `sites.org_country=Canada` and `sites.org_country=Uganda`, permits schema fields as selectors and documents OR for repeated values of one field. NCI receives repeated `sites.org_country` parameters without translation. A country filter makes no claim about site recruitment or patient eligibility.
