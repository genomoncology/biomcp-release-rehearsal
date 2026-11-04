# Adverse event slice

## Basis and evidence boundary

This investigation starts from `d18bfbba511e3852c99228f1a5166a14633e5cee`. The revision applies the PM review of 21ad15d2 and merges main cdc677b0bc2c1abff17d04e3138d8bd5e9dcc4b4. The legacy inventory reads `repos/biomcp` revision `4050fdc7dbdf4d713d94cf02a39a147b863665be`; it does not execute that binary. This proposal follows steps 1–4 of [slices](../slices.md). It changes no product code. The PM reviews this branch before filing implementation tickets.

Shared types cannot be accepted before exact-request native captures. The following recipe documents the completed separate-delegate capture; it authorizes no repeat. The investigation author made no provider API requests, product executions, cold/warm timings, output token measurements or legacy comparison runs. A separate delegate ran the bounded native recipe; its verified receipts and findings are recorded below. Documentation fields are questions for capture, never proof that all public records carry them.

## Executed bounded native capture recipe

Run through `pm job` from an exact pushed revision. Retain recipe, exact URL or POST body, start/end times, status, headers, unchanged bytes, SHA-256, byte size and classification for every attempted request. Store raw evidence on Yellow outside Git. Send each listed request once, sequentially, with zero retries, redirects, fallback spellings, extra pages, refreshes or health probes. A failed row stays failed; a 429 stops that source's remaining rows. Never provoke a provider failure. Local synthetic controls are separate evidence. Use HTTPS verification, 30-second request deadlines, 8 MiB openFDA/body cap and 2 MiB WONDER/body cap. These are capture limits, not measured costs.

Maximum provider budget: **34 requests**: 21 FAERS base rows, four device rows, three conditional exact report/identity rows, two CDC files, three WONDER rows, and one reserved exact FAERS aggregation query defined below. No request replaces a skipped row. No account creation, keys borrowed from another user, production data or licensed curator answers. openFDA runs keyless under its documented public quota. WONDER is keyless, includes `accept_datause_restrictions=true`, observes its [data use restrictions](https://wonder.cdc.gov/datause.html) and keeps the existing legacy gap of at least 16 seconds between requests. No re-identification or suppressed-cell reconstruction.

F01–F20 and F23 use `GET https://api.fda.gov/drug/event.json`, `limit=5`, `skip=0`, `sort=receivedate:desc`, except where stated. Encode the exact decoded expressions below. `N(x)` means `(patient.drug.openfda.generic_name:"x" OR patient.drug.openfda.brand_name:"x" OR patient.drug.medicinalproduct:"x")`. Expand it literally in the recipe and retain the final expression. These are explicit text searches, never identity joins.

| Row | Exact search or operation | Native question |
|---|---|---|
| F01 | `N(pembrolizumab)` | Complete drug/reaction arrays, report IDs/versions and optional harmonized identities. |
| F02 | `N(pembrolizumab) AND patient.reaction.reactionmeddrapt:"MYOCARDITIS"` | Co-reporting, never causation. |
| F03 | `N(pembrolizumab) AND patient.reaction.reactionmeddrapt:"PNEUMONITIS"` | Distinct source reaction term. |
| F04 | `N(trastuzumab) AND patient.reaction.reactionmeddrapt:"CARDIOMYOPATHY"` | Distinct products, including conjugates, must remain distinct. |
| F05 | `N(fam-trastuzumab deruxtecan-nxki) AND patient.reaction.reactionmeddrapt:"INTERSTITIAL LUNG DISEASE"` | No Enhertu spelling fallback if empty. |
| F06 | `N(axicabtagene ciloleucel) AND patient.reaction.reactionmeddrapt:"CYTOKINE RELEASE SYNDROME"` | CAR-T report scope. |
| F07 | `N(axicabtagene ciloleucel) AND patient.reaction.reactionmeddrapt:"NEUROTOXICITY"` | Distinct neurologic report scope. |
| F08 | `N(tamoxifen) AND patient.reaction.reactionmeddrapt:"PULMONARY EMBOLISM"` | Native reaction, indication and drug roles. |
| F09 | `N(venetoclax) AND patient.reaction.reactionmeddrapt:"TUMOUR LYSIS SYNDROME"` | Exact term; no spelling retry. |
| F10 | `N(bevacizumab) AND patient.reaction.reactionmeddrapt:"GASTROINTESTINAL PERFORATION"` | Native reporting term. |
| F11 | `N(ibrutinib) AND patient.reaction.reactionmeddrapt:"HAEMORRHAGE"` | Native reporting term. |
| F12 | `N(osimertinib) AND patient.reaction.reactionmeddrapt:"ELECTROCARDIOGRAM QT PROLONGED"` | Report versus label warning. |
| F13 | `N(pembrolizumab) AND seriousnessdeath:1` | Report-level death flag. |
| F14 | `N(pembrolizumab) AND seriousnesshospitalization:1` | Hospitalization is not a reaction-outcome code. |
| F15 | `N(pembrolizumab) AND receivedate:[20240101 TO 20241231]` | Receipt interval is not event onset or exposure interval. |
| F16 | `N(pembrolizumab) AND patient.drug.drugcharacterization:1` | Check roles inside each drug object; cross-object conjunction cannot establish that this drug was suspect. |
| F17 | `N(pembrolizumab) AND patient.patientsex:2 AND patient.patientonsetage:[65 TO 120] AND patient.patientonsetageunit:801 AND primarysource.qualification:1` | Explicit recorded female sex, years and physician; no age-unit assumption. |
| F18 | `safetyreportid:"BIOMCP-NO-SUCH-REPORT-0152"` | Exact-ID negative control; no-match versus transport failure. |
| F19 | F01 expression with `count=patient.reaction.reactionmeddrapt.exact`, `limit=5`, no sort/skip | Buckets and native count unit; no sum-of-buckets report denominator. |
| F20 | F01 expression, `limit=5`, `skip=5`, same sort | Actual next page. |
| F23 | `N(osimertinib) AND patient.reaction.reactionmeddrapt:"PNEUMONITIS"` | Separate ILD/pneumonitis question. |

D01–D04 use `GET https://api.fda.gov/device/event.json`, `limit=5`, `skip=0`, `sort=date_received:desc`.

| Row | Exact decoded search | Native question |
|---|---|---|
| D01 | `device.generic_name:"BREAST IMPLANT"` | Complete device list and report identity; implant presence does not establish cancer or reconstruction. |
| D02 | `device.generic_name:"BREAST IMPLANT" AND event_type:"Death"` | Device event types are separate from drug seriousness. |
| D03 | `mdr_text.text:"SQUAMOUS CELL CARCINOMA" AND device.generic_name:"BREAST IMPLANT"` | Literal capsule-SCC discovery query; no automatic narrative classification or BIA-ALCL equivalence. |
| D04 | `device.generic_name:"BREAST IMPLANT" AND date_received:[20240101 TO 20241231]` | Received period and optional narrative; no inference of event onset. |

F21 conditionally gets the first actual F01 report in native order with `safetyreportid:"<exact native ID>" AND safetyreportversion:"<exact native version>"`, limit 1. If version is absent, query exact ID alone and preserve every returned version. D05 gets the first actual D01 `mdr_report_key` by that exact field, limit 1. I01 selects the first F01 drug object carrying `openfda.unii` and queries `patient.drug.openfda.unii:"<first exact native UNII>"`, limit 5. All native UNIIs remain visible; this one selected query proves no universal product join. Missing prerequisite IDs skip the conditional row. F22 repeats F01 with `count=seriousnessdeath`, limit 5, no sort/skip, to inspect the aggregate unit without calculating a risk.

C01 downloads `https://www2.cdc.gov/vaccines/iis/iisstandards/downloads/cvx.txt` once. C02 downloads `https://www2.cdc.gov/vaccines/iis/iisstandards/downloads/TRADENAME.txt` once. Record whole-file receipts, delimiters, optional columns and notices. Inspect exact CVX 165, 62, 118 and trade rows whose CVX column equals those values. No MVX file or CPT crosswalk is needed. Never infer a CVX-to-WONDER mapping from text or numerals.

V01–V03 POST to `https://wonder.cdc.gov/controller/datarequest/D8` as `request_xml=<XML>&accept_datause_restrictions=true`. Start with the exact committed legacy `spec/fixtures/vaers/reactions-request.xml` at the cited revision. Change only `F_D8.V14` from `MMR` to `HPV9`; change `B_1` respectively to `D8.V13-level2`, `D8.V10`, `D8.V1`. Preserve every other parameter including declared vintage. Archive final XML before sending. Record actual returned dataset/date/version, notices, suppression, row labels/codes, totals and counted unit. This is a caller-selected native code, not an inferred CVX165 join. Rejected HPV9 ends that row; no HPV4 or alternate-code fallback.

Each harvested question will map to these rows or a specific out-of-scope reason. Cancer drug questions do not call VAERS/CVX; vaccine questions do not call FAERS automatically; devices do not call drug endpoints. An API error is never negative clinical evidence.

## MAUDE exact report capture

The separate MAUDE capture `j12974` ran at `82bac28e85fb1b30b6abc41761903cbc86d7c73d` with four fixed keyless requests and no retries, redirects, fallbacks or refreshes. The archive is `data/biomcp-redux/0160-maude-native-capture/j12974-native-capture.tar.gz` on Yellow, SHA-256 `233ed6086a44cf443bd3adb3410de610ab703fe70200df04a77ee7f89cf01b10`, 92,360 bytes, with all 36 files independently verified. Capture times span 2026-10-25T08:29:56.452326Z to 2026-10-25T08:30:00.864363Z. Publisher `meta.last_updated` is 2026-10-13, separate from retrieval.

D01 retained total 23,863 and ordered native keys 50952244, 50884084, 50884224, 50866859 and 50866966. Exact D05 key 50952244 carries one device and an empty narrative array. The selected first D02 key 2993151 carries one device and one narrative. The selected third D04 key 20988896 carries one device and five narratives. Every captured native device and patient count is a present empty string. Every captured `date_report` is present and `report_date` is absent. D02 keeps event date 20120301 and report/received date 20130305; D04 keeps event date 20241217, report date 20241219 and received date 20241220. These source strings remain independent. Full native nested inventories and unchanged bodies remain outside Git. This capture is source evidence and does not claim CLI or MCP acceptance.

## Documentation findings and source proposal

| Candidate | Proposed use and official reference | Use and terms | Native questions |
|---|---|---|---|
| openFDA FAERS | Keep explicit report search and exact report get; captured aggregation stays investigation-only. Independent drug and reaction arrays. [Overview](https://open.fda.gov/apis/drug/event/), [fields](https://open.fda.gov/apis/drug/event/searchable-fields/). | `open`, [openFDA terms](https://open.fda.gov/terms/), CC0 default with stated third-party exceptions; attribution visible. | Versions, optional UNII/RxCUI/NDC/SPL identities, dates, roles, multiplicity. |
| openFDA MAUDE | Separate device report family with native key/event type. [Overview](https://open.fda.gov/apis/device/event/), [endpoint](https://open.fda.gov/apis/device/event/how-to-use-the-endpoint/). | `open`, same FDA terms and exceptions. | Full device list, product code/UDI availability, report multiplicity, narrative size. |
| CDC WONDER VAERS | Ported for MMR, COVID-19 and separately selected native influenza types after approved captures returned aggregate tables. The earlier HPV9 denials remain historical evidence. [API](https://wonder.cdc.gov/wonder/help/wonder-api.html), [help](https://wonder.cdc.gov/wonder/help/vaers.html). | Restricted statistical use under [CDC WONDER data use restrictions](https://wonder.cdc.gov/datause.html); the runtime registry enforces source and restricted-group controls. | Actual fields, totals/counting unit; no promised individual report get. |
| CDC CVX/trade names | Dropped: no proved WONDER mapping. Previously considered vaccine identities. [Code sets](https://www.cdc.gov/iis/code-sets/index.html). No token-based name bridge or automatic search-time bundle sync. | Historical research terms: government tables with native notices retained; no runtime registry addition. CPT/AMA tables excluded. | Status/dates, native product-to-CVX/MVX relations, no proved WONDER join. |
| ClinicalTrials.gov results | Drop automatic fallback after FAERS empty. Trial-owned explicit evidence remains separate. | Existing trial source terms. | No capture requested; trial denominators cannot replace spontaneous report counts. |
| FDA recalls | Drop the legacy recall route from this entity; regulatory notices need separate question-driven ownership. | Existing FDA terms. | No capture requested; no claim of recall parity. |

[Authentication](https://open.fda.gov/apis/authentication/) lists public keyless limits and [the event guide](https://open.fda.gov/apis/drug/event/how-to-use-the-endpoint/) permits keyless use. Optional higher quotas do not change source rights. The registry will own use categories, terms and disable policy once; response types will reference that source truth.

## Native capture finding

A separate build delegate ran `j12641` from landed `d18bfbba511e3852c99228f1a5166a14633e5cee` with the exact proposed 34-row ledger. It made 34 physical provider requests: FAERS 23 HTTP200 plus the intentional exact-ID HTTP404; devices four HTTP200 plus the literal SCC query HTTP404; both CDC files HTTP200; WONDER three HTTP403 HTML denials. There were no retries, redirects, refreshes or fallbacks. This is source evidence, not product acceptance or legacy execution.

The unchanged evidence is on Yellow at `data/biomcp-redux/0152-adverse-event-native-proof/j12641-native-capture.tar.gz`, SHA-256 `b96dcbbfb7349f79596eaea262674a9711f9968c605572241a2c191caabf101a`, 1,198,872 bytes. The delegate independently verified all 225 manifest files. Archive root `2293-adverse-event-native-capture` contains unchanged bodies, request/receipt/headers, both job logs, exact original/final XML, `ledger.json`, `summary.json`, `native-field-inventory.json` and explicitly non-product `offline-investigation.json`. The independent verification JSON sits beside the archive. Original source dates remain separate from local investigation dates.

F01 returned native total 105020 and report 26940567 version 1 first. F21 exact get returned that native report. Historical or multi-version retrieval is unproved; no --version option is approved. F20 returned native skip 5 and total 105020. The five F01 report objects contained respectively 2/9/6/5/2 drug rows and 1/3/4/1/2 reaction rows. The default page must not select one drug as the report's sole drug. I01 selected the first actual drug object's UNII DLE8519RWM from PADCEV, not the later KEYTRUDA object's UNII, and returned total 7224. This proves one exact native-ID query and no pembrolizumab identity join. F19 string reaction terms and F22 numeric terms are separate native bucket shapes, not a common clinical count vocabulary or denominator.

The first device key 50952244 returned unchanged through exact D05 get. Device event, received and reporting dates are distinct. Optional source strings can be present and empty; empty must not silently become absent without an explicit reviewed rule. C01 carried 290 seven-column pipe-delimited rows. C02 carried 225 nine-column rows, including its trailing empty column. Exact CVX 118/62/165 and their trade rows were selected by the native CVX column while preserving original whitespace. These native code relations do not prove a WONDER relation.

All three WONDER XML requests returned 403 Access Denied HTML. No aggregate schema, code acceptance, vintage, total, suppression or report-level identifier was proved. Do not accept VAERS response types or advertise a working summary from these replies. The initial design dropped VAERS/CVX pending Ian. His later retry ruling and the successful MMR/COVID-19 captures supersede that source disposition; the VAERS contract below defines the supported summaries. No alternate HPV code, user-agent workaround or repeated attempt is authorized by this evidence. The HPV questions remain unanswered by this tier. The literal SCC device query's 404 is a verified no-match for that exact expression, not evidence that capsule SCC does not occur.

### Measured native request table

Each row used one physical request. Retrieval ran October 25, 2026, from 05:50:13 through 05:51:19 UTC. Successful FAERS JSON carried publisher `last_updated` July 30, 2026; device JSON carried October 13, 2026. Those dates are distinct from retrieval. Duration is one HTTP capture rounded to milliseconds, not cold/warm CLI/MCP time. Bytes are unchanged source bodies, not product output. A dash means the provider supplied no report total or this response is not a report page. C01/C02 file row counts are recorded above. Native aggregation totals remain absent.

| Row | HTTP | Returned rows | Native report total | Source bytes | HTTP seconds |
|---|---|---|---|---|---|
| F01 | 200 | 5 | 105020 | 220,919 | 0.737 |
| F02 | 200 | 5 | 2147 | 57,911 | 0.607 |
| F03 | 200 | 5 | 2608 | 95,557 | 0.521 |
| F04 | 200 | 5 | 727 | 107,070 | 0.564 |
| F05 | 200 | 5 | 1020 | 22,325 | 0.690 |
| F06 | 200 | 5 | 4503 | 24,410 | 0.522 |
| F07 | 200 | 5 | 3355 | 50,902 | 0.510 |
| F08 | 200 | 5 | 343 | 498,926 | 0.709 |
| F09 | 200 | 5 | 1147 | 122,064 | 0.608 |
| F10 | 200 | 5 | 1131 | 80,611 | 0.572 |
| F11 | 200 | 5 | 6160 | 25,409 | 0.448 |
| F12 | 200 | 5 | 53 | 189,906 | 0.576 |
| F13 | 200 | 5 | 18819 | 334,430 | 0.659 |
| F14 | 200 | 5 | 44253 | 234,202 | 0.630 |
| F15 | 200 | 5 | 16093 | 92,376 | 0.493 |
| F16 | 200 | 5 | 104869 | 220,919 | 0.636 |
| F17 | 200 | 5 | 8060 | 97,917 | 0.703 |
| F18 | 404 | 0 | — | 80 | 0.354 |
| F19 | 200 | 5 | — | 769 | 0.437 |
| F20 | 200 | 5 | 105020 | 60,551 | 0.506 |
| F23 | 200 | 5 | 134 | 54,278 | 0.485 |
| D01 | 200 | 5 | 23863 | 30,568 | 1.262 |
| D02 | 200 | 5 | 17 | 30,951 | 0.453 |
| D03 | 404 | 0 | — | 80 | 0.365 |
| D04 | 200 | 5 | 2046 | 36,916 | 0.516 |
| F21 | 200 | 1 | 1 | 6,681 | 0.372 |
| D05 | 200 | 1 | 1 | 5,931 | 0.364 |
| I01 | 200 | 5 | 7224 | 416,953 | 0.662 |
| F22 | 200 | 2 | — | 551 | 0.406 |
| C01 | 200 | — | — | 55,005 | 0.257 |
| C02 | 200 | — | — | 28,227 | 0.187 |
| V01 | 403 | — | — | 401 | 0.363 |
| V02 | 403 | — | — | 401 | 0.307 |
| V03 | 403 | — | — | 401 | 0.302 |

### Observed native fields and disclosure consequences

The JSON field inventory contains strings, integer metadata/counts, objects and arrays. No null-valued field was observed in this sample; this does not establish a universal no-null schema. Age/outcome/role/date values are source strings, not inferred JSON integers. Report drug arrays reached 19 rows and reaction arrays 22. Harmonized `package_ndc` arrays reached 2,087 entries and `spl_set_id` arrays 841. Those many identifiers do not establish which marketed product was administered. Device arrays happened to contain one row per captured report; narrative arrays ranged from empty to five. A single captured device never justifies a scalar schema.

Drug rows carry medicinal text, role, indication and identity collection counts. Full arrays use closed named sections unii/rxcui/product-ndc/package-ndc/spl-id/spl-set-id/application-number with --drug-index and section paging. The native absolute index belongs to the checked retained report snapshot. No arbitrary --field selector is proposed. Other harmonization collections need an explicit bounded component schema before advertisement.

The largest native five-report response in this bounded sample was F08 at 498,926 bytes. It motivates bounded layers but supplies no measured envelope size or approved allocation. Identity-list options need native long-list/full-walk cases with exact report snapshot binding and no duplicate/gap assertions. Paging a checked retained response makes no new provider request. Type review must account explicitly for administrative/address fields no harvested case needs; this proposal does not label a legacy struct complete.

F19 native reaction buckets were malignant neoplasm progression 12016, death 5857, diarrhoea 5740, fatigue 5131 and off-label use 4758. F22 numeric term 2 had 59345 and term 1 had 18819. A numeric term is not a Boolean without its field codebook; their sum is not the F01 total. No risk or denominator is inferred. The JSON negative controls F18/D03 carried native `NOT_FOUND` and `No matches found!`; the denied HTML WONDER replies are different failures.

## Clinical rulings and harvested questions

A fresh clinical reviewer read the existing drug/disease rulings and official sources on October 25, 2026. These are faithful reformulations of published safety concerns, not invented verbatim clinician quotations or licensed curator answers. Each accepted answer below is an evidence boundary. A label can establish its own warning; a report query cannot reproduce a trial incidence or establish causality.

The reviewer confirmed with high confidence: FAERS, MAUDE and VAERS counts describe reports, not exposed populations or causal events; report drug/reaction arrays supply no pairwise causal attribution; source suspect/concomitant roles remain unvalidated reporter classifications; seriousness is separate from reaction outcome and CTCAE grade; empty and unavailable sources provide no clinical reassurance. Harm from violating these boundaries is high. The contrary reading, that more reports prove a more dangerous product, lacks an exposure denominator and ignores reporting bias/duplicates. A separately attributed suitable clinical study could support a risk estimate; a larger report count would not change this ruling.

HPV vaccine questions belong here only as cancer prevention. The reviewer confirmed that boundary from CDC guidance with high confidence. General vaccine research stays outside scope. Implant presence alone establishes neither breast cancer nor reconstruction. Capsule SCC, BIA-ALCL and breast carcinoma remain distinct source claims. A report narrative is not a source-coded diagnosis established by our software.

| Case | Question and published concern | Accepted answer and direct-source coverage |
|---|---|---|
| A01 | What reports mention pembrolizumab and myocarditis? L1/P1. | Preserve matching reports, terms and drug roles; label myocarditis warning is independent. F01/F02. |
| A02 | What reports mention pembrolizumab and pneumonitis? L1. | Preserve pneumonitis without merging pneumonia; no trial/report denominator mixture. F03. |
| A03 | What cardiomyopathy reports mention trastuzumab? L2. | Keep product identities distinct, including antibody conjugates. F04. |
| A04 | What fatal ILD/pneumonitis reports mention trastuzumab deruxtecan? L3. | Preserve reaction and death flags independently; do not substitute trastuzumab. Unanswered as asked: F05 has 1020 matches but no fatal filter, so the fatal subset is unproved. |
| A05 | What serious CRS reports mention axicabtagene ciloleucel? L4. | Keep named product and native seriousness; label warning does not establish reporting incidence. F06. |
| A06 | What neurologic toxicity reports mention axicabtagene ciloleucel? L4. | Preserve native terms; do not require simultaneous CRS. F07. |
| A07 | What pulmonary embolism reports mention tamoxifen in breast cancer care or risk reduction? L5. | Preserve stated indication; missing indication prevents assigning a population. F08. |
| A08 | What TLS reports mention venetoclax, including fatal outcomes? L6. | Preserve TLS/outcomes; do not infer dialysis, phase or malignancy. F09. |
| A09 | What gastrointestinal perforation reports mention bevacizumab? L7. | Return reported terms/outcomes without causal attribution. F10. |
| A10 | What hemorrhage reports mention ibrutinib, including reports marked as deaths? L8. | Preserve other drugs and roles; co-listing does not prove an interaction. F11. |
| A11 | What QT-prolongation reports mention osimertinib? L9. | Do not translate QT prolongation into torsades, arrhythmia or death. F12. |
| A12 | What ILD/pneumonitis reports mention osimertinib? L9. | Preserve chemotherapy/radiation context when stated; no pooling label populations. F23. |
| A13 | Which pembrolizumab–myocarditis reports are serious? F3/F4/L1. | Preserve native overlapping flags; no CTCAE grade assignment. Inspect F02, compare F13 only as a broader report query. |
| A14 | Which venetoclax–TLS reports record hospitalization, death or both? F3/F4/L6. | Preserve independent flags; no forced mutually exclusive outcome. Inspect F09; F13/F14 prove field forms on a different named query. |
| A15 | Which pembrolizumab–pneumonitis reports were first received during 2024? F3. | First FDA receipt is `receivedate`, not event onset or `receiptdate`. Unanswered as asked: F15 proves only the broader drug/date range, not the combined reaction/date query. |
| A16 | Which ibrutinib–hemorrhage reports identify ibrutinib as suspect? F3/F4. | Role must belong to that drug object; source conjunction across arrays is insufficient. Inspect F11; F16 tests broader query mechanics only. |
| A17 | What matching trastuzumab/cardiomyopathy count and denominator support a percentage? F1–F3/L2. | Query/snapshot/unit required; no patient-risk percentage. F04 has a report total; F19/F22 measure aggregation on pembrolizumab, not this exact pair. |
| A18 | What breast-implant lymphoma reports record death in reconstruction? M1/M2. | Keep event type and stated indication; generic implant query does not establish lymphoma or reconstruction. D01/D02 supply candidates, not a complete answer. |
| A19 | What reports describe SCC in the capsule around breast implants? M3. | Preserve narrative text as such, distinct from BIA-ALCL and breast carcinoma. D03 is explicit text discovery; no locally inferred diagnosis. |
| A20 | Does a breast-implant report total represent unique patients/events? M1/M2. | State native report counting unit and duplicates; FDA curated BIA-ALCL cases differ from broad text matches. D01/D05. |
| A21 | What syncope reports follow HPV vaccination in cancer prevention? V1/V2. | Preserve native vaccine code/reaction rows; no causal claim or incidence. Unanswered: V01 is 403; code tables do not answer this. HPV remains unsupported by the current MMR/COVID-19 summary contract. |
| A22 | What serious allergic-reaction reports follow Gardasil 9? V1/V2. | Do not diagnose anaphylaxis from nonspecific symptoms or combine vaccines. Unanswered: V01/V02 are 403; neither marginals nor a joint answer were obtained. |
| A23 | What can an empty exact cancer-drug/reaction/date query establish? F1/V1. | Only no matching records under those selectors/snapshot. D03 is the real empty SCC query 404; F18 is a synthetic nonexistent-ID control, not this clinical query. |
| A24 | Synthetic engineering control motivated by source limits: can timeout/refusal/unreadable data display zero reports? F1/M1/V1. | No; unavailable/incomplete is separate from a verified empty. Local controlled failures can prove product handling later; no provider failure is manufactured. |

### Clinical source catalog

Pinned label versions establish the cited warning. They are not claims that each linked label is the newest approved revision. Undated documentation was accessed October 25, 2026. The reviewer reread the cited passages and rejected causal or incidence interpretations.

| Key | Source, date/version and passage relied upon |
|---|---|
| F1 | [FDA FAERS dashboard presentation](https://www.fda.gov/files/drugs/published/%22The-FAERS-Public-Dashboard-and-its-Value-to-the-Pharmaceutical-Industry-%22-July-10--2018-Issue-PDF.pdf), July 10, 2018, p2: FAERS cannot calculate incidence. |
| F2 | [FDA postmarket surveillance explanation](https://www.fda.gov/drugs/cder-conversations/understanding-cders-postmarket-safety-surveillance-programs-and-public-data), undated: reports do not establish drug causation. |
| F3 | [openFDA field reference](https://open.fda.gov/fields/drugevent_reference.pdf), undated; [official field definitions](https://raw.githubusercontent.com/FDA/open.fda.gov/master/static/fields/drugevent.yaml): latest report version, unvalidated roles, distinct receipt dates and reaction outcome codes. |
| F4 | [FDA drug dashboard FAQ](https://fis.fda.gov/extensions/fpdwidgets/2e01da82-13fe-40e0-8c38-4da505737e36.html), undated: quarterly extract concomitant drugs differ from dashboard suspect-only view. |
| L1 | [KEYTRUDA label](https://www.accessdata.fda.gov/drugsatfda_docs/label/2025/125514s173lbl.pdf), June 2025, §§5.1/6.1: immune-mediated pneumonitis and fatal myocarditis. |
| L2 | [HERCEPTIN label](https://www.gene.com/download/pdf/herceptin_prescribing.pdf), May 2026, boxed warning: clinical/subclinical cardiac failure. |
| L3 | [ENHERTU label](https://www.accessdata.fda.gov/drugsatfda_docs/label/2025/761139s038s042lbl.pdf), December 2025, boxed warning: ILD/pneumonitis including fatal cases. |
| L4 | [YESCARTA label](https://www.fda.gov/media/193167/download), June 2026, boxed warning: fatal/life-threatening CRS and neurologic toxicity with or after CRS. |
| L5 | [Tamoxifen DailyMed label](https://dailymed.nlm.nih.gov/dailymed/lookup.cfm?setid=509f8ba3-214d-aec8-e063-6294a90af498&version=2), version 2, April 30, 2026, boxed warning: fatal pulmonary emboli in the stated risk-reduction/DCIS context. |
| L6 | [VENCLEXTA label](https://www.rxabbvie.com/pdf/venclexta.pdf), May 2026, §5.1: TLS including fatal events and dialysis. |
| L7 | [AVASTIN label](https://www.gene.com/download/pdf/avastin_prescribing.pdf), September 2022, §5.1: serious/fatal gastrointestinal perforation. |
| L8 | [IMBRUVICA label](https://www.rxabbvie.com/pdf/imbruvica_pi.pdf), October 2025, §5.1: fatal bleeding. |
| L9 | [TAGRISSO label](https://www.dailymed.nlm.nih.gov/dailymed/drugInfo.cfm?audience=professional&setid=5e81b4a7-b971-45e1-9c31-29cea8c87ce7), September 14, 2026, §§5.1–5.2: severe/fatal ILD/pneumonitis and QTc prolongation. |
| M1 | [FDA About MAUDE](https://www.fda.gov/medical-devices/mandatory-reporting-requirements-manufacturers-importers-and-device-user-facilities/about-manufacturer-and-user-facility-device-experience-maude-database), undated: MDR data cannot evaluate adverse-event rates. |
| M2 | [FDA BIA-ALCL reports](https://www.fda.gov/medical-devices/breast-implants/medical-device-reports-breast-implant-associated-anaplastic-large-cell-lymphoma), tables through June 30, 2024: incomplete information can prevent duplicate removal. |
| M3 | [FDA implant risks](https://www.fda.gov/medical-devices/breast-implants/risks-and-complications-breast-implants), December 14, 2023: literature and MDR reports describe capsule SCC. |
| V1 | [VAERS interpretation guide](https://vaers.hhs.gov/data/dataguide.html), May 8, 2025 public-data expansion: added secondary reports concern already reported events; reporting does not document vaccine causation. Current data/guide pages differ on exact secondary-file inclusion. Native tier/version must settle the applicable count unit. |
| V2 | [CDC HPV safety/effectiveness](https://www.cdc.gov/hpv/hcp/vaccination-considerations/safety-and-effectiveness-data.html), July 9, 2024: vaccination prevents cancer-causing infections and cervical precancers. |
| P1 | [Salem et al.](https://escholarship.org/content/qt66j3w519/qt66j3w519.pdf), Lancet Oncology 19(12), December 2018, author manuscript p9: VigiBase cannot establish cardiovascular immune-event incidence. Disproportionate reporting and fatality among reported cases are not exposed-patient risk. |

## Identity and native semantics

Keep FAERS `safetyreportid` as an opaque string and its source `safetyreportversion` separately. Do not parse a checksum or fabricate a case-family ID from inconsistent overview prose. Exact get verifies the returned ID and, when requested, version before cache admission. MAUDE uses its own opaque `mdr_report_key`, with `report_number` retained separately. Numeric report IDs never trigger cross-source probing. Get defaults to FAERS; MAUDE requires `--source maude`. There is no source probing or speculative FAERS-then-MAUDE resolver.

Each FAERS drug object retains the original `medicinalproduct`, role code, indication, administration fields and optional harmonized identity arrays. Distinct objects do not merge by name. An exact source-provided UNII, RxCUI, NDC or SPL cross-reference may offer the corresponding existing drug command only after that command's accepted namespace and granularity are checked. An ingredient UNII does not identify a marketed product, application or package. Missing/multiple mappings remain missing/candidates. Literal name search returns reports/candidates, never a chosen drug identity. The source report title must not choose the first suspect or first listed drug as the whole report's drug.

FAERS reactions remain an independent ordered array. A role filter must validate the selected matching drug object rather than assuming an indexed conjunction binds nested objects. Captures must settle whether an exact identity query supports that check. Name-only queries cannot acquire a trustworthy suspect-only promise from a report-level search expression.

Official reaction outcomes at last observation are `1 recovered/resolved`, `2 recovering/resolving`, `3 not recovered/not resolved`, `4 recovered/resolved with sequelae`, `5 fatal`, `6 unknown`. Unknown/missing are distinct. Report seriousness flags can overlap. The legacy mapping of hospitalization to reactionoutcome1 and disability to3 is wrong. Use report seriousness hospitalization/disabling instead. Neither field is a CTCAE grade. `receivedate` is first FDA receipt; `receiptdate` is receipt of the most recent information; neither is onset. Preserve dates with their field roles and native formatting evidence.

A count row is a native aggregation bucket, not a distinct person. Never sum reaction buckets into a report denominator or add seriousness marginals into a total. WONDER reaction/seriousness/age tables stay independent; never use the maximum total from separate queries as a fabricated common denominator. Preserve each native table's own total, vintage and caveats. A CVX code is a vaccine administration code, not proof of a WONDER family mapping. Native exact mappings need explicit provenance. No regexes, token extraction, synonym rewriting, inferred ontology categories or hidden label lookup occurs.

## Reviewed command tree

Search and get are the only commands. FAERS is the default source. Input is positional and literal; there is no --query or raw query language. The default page proposal is five rows, matching the retained native requests; acceptance requires adapter-owned wire measurements. No automatic fan-out, name expansion, trial fallback or source probing occurs.

| Command/options | Header and body | Native proof |
|---|---|---|
| `search adverse-event TEXT [--source faers] [--reaction TERM] [--serious any\|death\|hospitalization\|lifethreatening\|disabling\|congenital-anomaly\|other] [--received-from YYYY-MM-DD] [--received-to YYYY-MM-DD] [--limit N] [--offset N]` | Native report headers, total/unit, source date and exact built query echo. Four typed filter names only. | F01–F15/F20/F23. Combined selectors absent from captures remain unproved. |
| `search adverse-event TEXT --source maude [--limit N] [--offset N]` | Device generic-name report candidates, exact keys, native event type/date and total. | D01; FAERS filters reject before I/O. |
| `get adverse-event ID [--source faers]` | Exact native ID/version, fixed dates/seriousness/patient/reporter fields, collection counts, links and sections. | F21 exact 26940567/version 1. No --version; historical/multi-version get unproved. |
| `get adverse-event ID drugs\|reactions [--source faers] [--section-limit N] [--section-offset N]` | Independent ordered drug or reaction rows. Identity arrays stay behind named pages. | Native arrays up to 19 drug and 22 reaction rows. |
| `get adverse-event ID unii\|rxcui\|product-ndc\|package-ndc\|spl-id\|spl-set-id\|application-number --drug-index N [--source faers] [--section-limit N] [--section-offset N]` | One named original identifier array from one absolute native drug row. | F01/F08/I01 long collections; no inferred administered product or name join. |
| `get adverse-event ID --source maude` | Exact key/report number, native event type/date roles, source counts, links and sections. | D05 exact 50952244 and D01/D02/D04 variants. |
| `get adverse-event ID devices\|narrative --source maude [--section-limit N] [--section-offset N]` | Native device/narrative rows; no clinical extraction from prose or silent text loss. | Device arrays 1 per captured report; narrative 0–5 rows. |

FAERS uses the exact expanded N(TEXT) expression from the ledger. `--reaction TERM` appends `AND patient.reaction.reactionmeddrapt:"TERM"`, escaped only as a literal. openFDA treats this analyzed field as a phrase match, not exact preferred-term equality; help and the query echo identify phrase-match semantics. No local synonyms or normalization. `--serious any` appends `AND serious:1`, using the native report-level flag. `--serious death` appends `AND seriousnessdeath:1`; hospitalization uses `seriousnesshospitalization:1`. The other exact mappings are lifethreatening to `seriousnesslifethreatening:1`, disabling to `seriousnessdisabling:1`, congenital-anomaly to `seriousnesscongenitalanomali:1` and other to `seriousnessother:1`. Receipt bounds append `AND receivedate:[YYYYMMDD TO YYYYMMDD]`; a missing bound is `*`. Both 2024 bounds reproduce F15. Neither one-sided request was separately captured, so its builder case is synthetic, not a native positive. MAUDE uses `device.generic_name:"TEXT"`, a phrase match on the analyzed generic-name field, as in D01's BREAST IMPLANT query. Help and the query echo identify this phrase match. Neither selector claims exact equality; a `.exact` selector requires its own captured positive first. Date meaning is first FDA receipt. Invalid/reversed dates, unknown values and unsupported source/filter/section combinations fail before I/O. Actual query output expands N fully; no shorthand is sent or echoed.

Cards hold `source_url` and `report_url`. Source URL is the provider overview; report URL is the exact successful report request from its receipt. No public browser URL is guessed. Links are data, not next steps. Architecture 6.10 governs one pure entity next function: selectable report headers get one disambiguate get each, then page, then at most three extras; ordinary cards/sections have at most five actions. Verified association links get one distinct pivot each in source order, then page, then at most three extras. Section/page actions retain source, exact ID, selectors and snapshot. An expired snapshot restarts explicitly. No unproved external identity pivot or report-version action is emitted.

## Types, field lists and JSON shapes

These are schematic design JSON, not executions, clinical zero counts or budget measurements. Angle-bracket strings mean exact native values. Numeric 0 illustrates an integer type only. Missing optional facts are omitted, not defaulted to zero/false. The existing envelope owns status/query/terms/source outcomes/Problems. Section pages share exact report identity/source/version/snapshot, native offset/total, capped records and typed next. Empty lists follow the existing omission rule; explicitly present empty source strings remain distinguishable from absent fields.

Search echoes the exact literal provider expression as `source_query`. A reaction-filtered page names its analyzed-field matching mode as `reaction_match: "phrase_match"`. The publisher `last_updated` date is `source_updated`; the received record’s UTC retrieval date is `fetched`. An undated legacy recording omits `fetched`. These are separate dates. Page totals stay in the envelope and have unit `reports`.

`ReportHeader`: published ID string, source, optional version, optional native serious code, optional MAUDE event type, optional first FDA receipt or device receipt date. There are no flattened drug/reaction lists or a selected causal-drug title. IDs 26940567/50952244 remain those strings with separate source; no FAERS: or MAUDE: wrapper.

```json
{"items":[{"id":"<native ID>","source":"faers","version":"<native version>","first_received":"<receivedate>","serious":"<native code>"}],"total":0,"unit":"reports"}
```

`FaersCard`: header; optional latest_received/transmitted plus source format; optional independent overlapping seriousness flags; optional fixed patient age/age_unit/sex/weight strings; optional reporter qualification/country; actual drug/reaction array counts; links; available_sections and next. Reporter is a card subobject, not a section. F01 inventory proves qualification and reportercountry strings in all five primarysource objects; I01 includes a missing qualification, which stays optional.

```json
{"header":{"id":"26940567","source":"faers","version":"1"},"latest_received":"<receiptdate>","seriousness":{"death":"<native flag>","hospitalization":"<native flag>"},"patient":{"age":"<native age>","age_unit":"<native unit>"},"reporter":{"qualification":"<native code>","country":"<native country>"},"drug_count":2,"reaction_count":1,"source_url":"https://open.fda.gov/apis/drug/event/","report_url":"<exact F21 request URL>","available_sections":["drugs","reactions"]}
```

`MaudeCard`: header; optional report_number/event_date/received/report_date/date_report_to_fda; source number_devices_in_event/number_patients_in_event strings; actual array counts; links/sections/next. Native report and device/patient counts are different units.

```json
{"header":{"id":"50952244","source":"maude","event_type":"<native type>"},"report_number":"<native number>","event_date":"<date_of_event>","received":"<date_received>","device_count":1,"narrative_count":0,"source_url":"https://open.fda.gov/apis/device/event/","report_url":"<exact D05 request URL>","available_sections":["devices","narrative"]}
```

Each section's record fields are fixed by its own schema. Optional fields below are included only when supplied. Drug rows also preserve native optional administration start/end/dose/route fields and identity collection counts from the field inventory. Absolute row indexes are source-array positions, not identity guesses.

```json
{"drugs":{"records":[{"index":0,"medicinalproduct":"<native text>","role":"<drugcharacterization>","indication":"<drugindication>","identity_counts":{"unii":0,"package_ndc":0}}],"offset":0,"total":0}}
```

```json
{"reactions":{"records":[{"index":0,"term":{"label":"<reactionmeddrapt>"},"vocabulary_version":"<reactionmeddraversionpt>","outcome":"<reactionoutcome>"}],"offset":0,"total":0}}
```

```json
{"package_ndc":{"drug_index":0,"records":["<original identifier>"],"offset":0,"total":0}}
```

The other six identifier sections use the identical string-record shape under their closed section name. UNII, RxCUI, NDC, SPL UUIDs and application numbers keep published forms; no numeric cast, normalized NDC or invented CURIE. Other harmonization arrays require their own reviewed bounded component page before advertisement.

```json
{"devices":{"records":[{"index":0,"sequence":"<device_sequence_number>","brand_name":"<native text>","generic_name":"<native text>","manufacturer":"<manufacturer_d_name>","product_code":"<device_report_product_code>","model":"<model_number>","catalog":"<catalog_number>","udi_di":"<native value>","udi_public":"<native value>"}],"offset":0,"total":0}}
```

```json
{"narrative":{"records":[{"index":0,"type":"<text_type_code>","text":"<unchanged source text>"}],"offset":0,"total":0}}
```

Device rows retain optional native lot/operator/implant/removal facts kept by field review. Administrative addresses without a harvested need are excluded explicitly from the consumer projection; unchanged raw evidence remains external. An oversized individual narrative cannot be silently cut: return a truthful too-large Problem or obtain review for a bounded text-part schema before acceptance.

The reviewed neutral record projection preserves the kept source fields rather than all administrative raw payload fields. FAERS sender/receiver/duplicate/company/address metadata, patient summary prose, drug recurrence and active-substance subobjects, and harmonization collections beyond the seven named identifier arrays remain outside this consumer slice. MAUDE administrative contacts, addresses, patient administrative rows and unreviewed device harmonization arrays remain external. Device lot/operator/implant/removal fields and native administration strings remain optional and unchanged. Both `date_report` and `report_date` are kept separately when present; no captured date is renamed to an unobserved to-FDA date.

Biodata owns neutral complete no-I/O source records, IDs and Concept. Consumer types own query selectors, response header/card/page schemas, closed sections and next. Reactions use `Concept {label}` without a CURIE because the native term carries no ontology ID; its MedDRA version stays separate. Schemas derive from types with the existing schema feature and compatibility check.

Report counts do not reuse `CohortObservation`: [its shared contract](../biodata-variants-and-annotations.md) requires `subject: VariantRef` and units samples/patients/tumors. These totals count reports with no variant subject. Casting them would invent identity and units. Keep the existing page total with explicit reports unit; no patient/sample denominator is present. F19/F22 remain investigation evidence only; there is no new runtime count schema or change to the shared type.

## Source and component terms

FAERS/MAUDE records remain `open` under [openFDA terms](https://open.fda.gov/terms/) and [licensing](https://open.fda.gov/license/), with CC0 limited by publisher exceptions. MedDRA is third-party terminology, not our ontology. Keep source-reported reaction labels/version under a separate `restricted` MedDRA component and [data-sharing terms](https://admin.meddra.org/sites/default/files/page/documents/000198_statement_on_meddra_data_sharing_1_1.pdf). No MedDRA dictionary is downloaded or separately licensed. This grants no dictionary/commercial rights.

openFDA's license explicitly marks GMDN term codes/names/definitions as licensed third-party content and requires a suitable license for listed commercial categorization/mapping/AI-training uses. Label proven GMDN content `restricted` with that publisher link. The `gmdn` component owns the exact source path `device.generic_name`. The registry ticket must name every additional covered device field by its exact source path; no broad device-name heuristic determines component membership. This conservative field assignment does not assert that every captured string is a proved GMDN term; preserve its native field provenance. No GMDN codes or inferred mappings are added. Source IDs/report counts/outcomes keep independent terms.

Registry-owned component names meddra/gmdn and the restricted group suppress applicable fields before cache/replay reads; source-wide disable suppresses the whole source. Every component stays on by default under the standing rule. No user key is required for the kept keyless public tier. VAERS has a restricted runtime registry entry and bounded summary pages. Dropped CVX has no runtime registry addition or budget.

## Delight questions

1. The first question names a drug and reaction phrase; default FAERS search answers it through a typed filter and exact query echo.
2. Headers expose source/identity/date/seriousness. Cards expose fixed native facts and links; growable lists sit behind sections.
3. Next follows architecture 6.10, with report choices, section/page actions and only proved pivots.
4. Native report and drug/device IDs retain published forms; missing/multiple mappings remain unresolved.
5. Label warnings, reported roles and report counts keep separate attribution and meanings.
6. Empty, disabled, malformed, stale and unavailable stay distinct; failures never become zero or another source's answer.
7. The table measures source HTTP only. Adapter tickets own cold/warm product latency and wire budgets.
8. Wrong outcome aliases, name bridges and trial fallback are dropped. VAERS summaries follow the later approved contract; CVX has no proved WONDER mapping.
9. An agent can stop at a source reporting answer; it cannot infer safety, causation or incidence.
10. Trust needs query/date/version/source links and explicit unanswered questions.

## Command and option cases

All native facts below were captured October 25, 2026 in j12641. Publisher dates remain distinct. New request combinations and malformed replies are synthetic recorded controls, never native clinical positives.

| Command/option | Native case/facts | Synthetic or unproved boundary |
|---|---|---|
| Default positional FAERS; explicit source | F01 five rows,total 105020; F20 skip 5 same total. | Literal escaping/empty input; default/explicit equality. |
| --reaction | F02 myocarditis 2147; F03 pneumonitis 2608. | No synonym rewrite; unsupported syntax. |
| --serious any plus the six native specific flags | Native serious:1, retained 2026-10-28 capture and replay case serious-any; F13 total 18819; F14 total 44253. | Any uses the source seriousness flag. Bare/unknown values reject; native flags can overlap. |
| --received-from/--received-to | F15 both bounds,total 16093. | Each one-sided builder, invalid/reversed dates and combination with reaction; those requests were not natively captured. A15 remains unanswered. |
| --limit/--offset | F01 five rows/F20 skip 5. | Bounds, result window, snapshot expiry and exact selector preservation. |
| MAUDE positional/source | D01 total 23863 and D05 exact key. | Four FAERS filters reject before I/O; no wildcard expansion. |
| Real empty | D03 exact historical SCC expression 404 NOT_FOUND. | Current simple CLI cannot express that two-field query; it is an adapter no-match fixture, not an invented CLI positive. F18 is a synthetic ID control. |
| FAERS get | F21 exact 26940567/version 1. | Wrong returned ID/version before cache; historical/multi-version get unproved and --version rejected. |
| drugs/reactions and section paging | F01 arrays 2/9/6/5/2 and 1/3/4/1/2; maxima 19/22. | Independent arrays/role on another drug; missing versus empty and snapshot full walk. |
| Seven identity sections and --drug-index | Native long F08/I01 collections; I01 is PADCEV UNII. | Each closed selector/bad index/multiple mappings; no name joins, no omitted rows or duplicated page. |
| MAUDE get/devices/narrative | D05 exact key and empty narrative; D01/D02/D04 arrays 0–5. | Multiple devices is a labeled synthetic case, not native proof; long narrative refusal. |
| Global format/disable/fresh/replay | Existing shared contract plus retained native source bodies. | CLI/MCP parity; no disabled cache leak; accepted stale data plus rejected refresh. |
| Source failure/A24 | V01–V03 real 403 HTML explain source loss. | FAERS/MAUDE 429/500/timeout/broken200 controls are synthetic. No live outage was captured. |
| Retirement routes | Complete source inventory below. | Every retired spelling returns named replacement or explicit loss with zero I/O. |

A04's fatal subset, A15's exact combined reaction/date question, A18's lymphoma/reconstruction restriction and A21–A22 vaccine questions remain unanswered. F13/F14 are broader than A13/A14; do not advertise combined-query parity from their totals.

## Measurement ownership

The measurements below cover actual native FAERS and MAUDE command output. The PM-filed adapter/command ticket owns measurement and the case ceiling at measured compact full-envelope size plus the PM-approved margin. Report terms-only deductions separately and validate exact metadata first. Estimated token counts name their estimator and never pretend to be actual tokenization. Native raw bytes and durations cannot substitute for these measurements.

| Budget row | Measurement owner and acceptance case |
|---|---|
| Default/filtered FAERS search | F01/F20/F02/F03/F23/F13/F14/F15 native replay measurements and +15% ceilings appear below. |
| FAERS card | j13004: 1459 full / 1344 comparable bytes; ceilings 1678 / 1546. |
| Drug/reaction sections | j13004: drugs 4120 / 4005 bytes, reactions 1528 / 1413; ceilings 4738 / 4606 and 1758 / 1625. Complete 19/22-row walks are retained. |
| Each of seven identity pages | j13004: UNII 760 / 645, RxCUI 948 / 833, product NDC 991 / 876, package NDC 1006 / 891, SPL ID 1102 / 987, SPL set ID 1118 / 1003, application number 1016 / 901 bytes. Individual ceilings and complete long-list walks appear below and in the retained results. |
| Default MAUDE search/card | D01/D05 actual CLI/MCP measurements and +15% ceilings appear below. |
| Devices/narrative sections | D05/D02/D04 actual CLI/MCP measurements, including the empty section and complete narrative walk, appear below. |
| Entity help/MCP listing/schema | j13144: search help 1663 compact / 1664 stdout bytes; get help 2242 / 2243. MCP description 1751 bytes within its existing ceiling. Schema bytes are measured separately below. |
| VAERS/CVX/count | VAERS summaries use the contract and budgets below. CVX identity lookup and unsupported count options remain dropped. |
| Legacy 0.9 comparison | Unavailable for every row here: no contained adverse-event 0.9 execution has been supplied. Full same-question comparison belongs to 0098. No old fixture, source body or design projection substitutes for it. |

`j12931` measured the actual CLI and MCP on `0c1707e9870a85e28e889793ea5633a912f2cf21`. All eight native five-report pages agreed across CLI and MCP. Default FAERS and explicit `--source faers` agreed. Native report IDs, versions, first-receipt strings, order, totals, offsets, publisher date and exact expressions were compared with the fresh pre-adapter captures from `j12888`. The publisher date was 2026-07-30; the source retrieval occurred on 2026-10-25. These pages establish the individual captured queries, not uncaptured filter combinations or clinical causation.

| Native search page | Full compact bytes | Comparable bytes | Full ceiling | Comparable ceiling | Comparable estimated tokens |
|---|---:|---:|---:|---:|---:|
| F01 default pembrolizumab | 1495 | 1380 | 1720 | 1587 | 345 |
| F20 native offset 5 | 1496 | 1381 | 1721 | 1589 | 346 |
| F02 myocarditis phrase | 1867 | 1752 | 2148 | 2015 | 438 |
| F03 pneumonitis phrase | 1867 | 1752 | 2148 | 2015 | 438 |
| F23 osimertinib/pneumonitis phrase | 1856 | 1741 | 2135 | 2003 | 436 |
| F13 report death flag | 1555 | 1440 | 1789 | 1656 | 360 |
| F14 report hospitalization flag | 1585 | 1470 | 1823 | 1691 | 368 |
| F15 first receipt during 2024 | 1649 | 1534 | 1897 | 1765 | 384 |

Full bytes contain the entire envelope. Comparable bytes remove only the validated FAERS `use`, `terms` and `terms_url` fields, exactly 115 bytes. Component terms remain counted. Each ceiling is the corresponding observed compact size plus 15 percent, rounded up. Estimated tokens are comparable UTF-8 bytes divided by four, rounded up. They are not actual tokenization. The generated search help measured 1637 compact bytes, with ceiling 1883. The two synthetic executable examples measured full/comparable 1441/1326 and 1596/1481 bytes, with ceilings 1658/1525 and 1836/1704. Synthetic examples prove command mechanics and no native positive.

The same proof recorded one cold local CLI replay and three warm ordinary-cache CLI reads per query. The eight cold observations ranged from 18.648 to 23.496 milliseconds; the 24 warm observations ranged from 16.173 to 23.598 milliseconds. These are process observations under shared machine load. The raw per-query samples and cache provenance remain in the proof log and archive. They establish no provider speed or tail latency.

The product proof retained real recorder files, raw CLI stdout/stderr, raw MCP JSON-RPC, exact arguments, source inputs, recipes and both attempt logs outside Git. Its first recipe attempt required redundant explicit default selectors in next commands and stopped after one local delivery. The corrected attempt used eight local deliveries; both attempts together made nine local deliveries and zero provider requests. Default-valued source and limit arguments are intentionally omitted by the shared checked speller. Gets select native reports without search filters; page actions preserve every search filter. The subsequent `Reply::check_status` correction rejects success-status error bodies and does not change these successful outputs.

The independently verified Yellow archive is `data/biomcp-redux/0158-faers-search-product-proof/j12931-product-proof.tar.gz`, SHA-256 `05b3e8b712f8c44d797b7b87c12a9760ed9c07b6e5a7e3b54264bb47c9a6b9fa`, with 392 manifest files. It includes the original fresh-capture archive, whose SHA-256 is `1c32e5327782e02585eab936163b1db4d01d40552651a20c0062393f29e97443`. Raw replies that carry terminology stay outside Git.

Legacy execution may retry, rename, auto-download bundles or fall back to trials. No verified containment exists for this assignment, so it has not run. Old fixtures are not new measurements. No legacy timing comparison or product clinical-parity claim is made.


### MAUDE actual command output

The offline native product proof executed 13 actual CLI/MCP pairs at `834b3338a42bd9bfcfb32ad77720f454d3749a9b`. The retained outputs independently verified equal, with D01 native order and total, exact D05/D02/D04 identities and date roles, native empty count strings, complete device/narrative rows and snapshot-bound next selectors. The five D04 narratives concatenate in source order across pages of 2/2/1 rows. There were four local fixture deliveries and zero provider requests. The replies are unchanged native bytes from `j12974`; new recorder times describe local delivery, and the separate provider receipts retain their original capture dates. These measurements establish local command output, not provider latency.

The `j13024` job failed after all pairs executed because its verifier indexed an omitted optional `next` on the terminal page. Independent byte-only validation of the retained outputs passed without rerunning the binary. The verified archive is `data/biomcp-redux/0160-maude-product-proof/j13024-product-outputs-verified.tar.gz` on Yellow, SHA-256 `f436b1bf6d77bbbb48381bf27e8ee763ce55c270dfd28861a6c5ed710baf242a`, 271,246 bytes, with all 192 files verified. Earlier local setup attempts failed before any body delivery and provide no product measurements. Retained D03 is an exact source error-body adapter fixture and supplies no SCC CLI positive.

| Native command | Full compact bytes | Comparable bytes | Full ceiling | Comparable ceiling |
|---|---:|---:|---:|---:|
| D01 default MAUDE search | 1616 | 1501 | 1859 | 1727 |
| D05 card 50952244 | 1259 | 1144 | 1448 | 1316 |
| D05 devices | 1381 | 1266 | 1589 | 1456 |
| D05 empty narrative | 616 | 501 | 709 | 577 |
| D02 card 2993151 | 1244 | 1129 | 1431 | 1299 |
| D02 devices | 1322 | 1207 | 1521 | 1389 |
| D02 narrative | 974 | 859 | 1121 | 988 |
| D04 card 20988896 | 1259 | 1144 | 1448 | 1316 |
| D04 devices | 1424 | 1309 | 1638 | 1506 |
| D04 default narrative | 2666 | 2551 | 3066 | 2934 |
| D04 narrative limit 2, offset 0 | 2052 | 1937 | 2360 | 2228 |
| D04 narrative limit 2, snapshot offset 2 | 1404 | 1289 | 1615 | 1483 |
| D04 narrative limit 2, snapshot offset 4 | 1042 | 927 | 1199 | 1067 |

Comparable output removes only the three validated publisher terms fields from the source outcome, exactly 115 bytes. Component terms, source context, query selectors and next commands remain counted. Every ceiling adds 15 percent to its observed UTF-8 compact size and rounds up. Estimated tokens use comparable bytes divided by four and rounded up. The whole-row narrative page bound lives in `NARRATIVE_PAGE_BYTES`; a row that cannot fit returns a too-large problem and no shortened text.

Synthetic command ceilings come from `j13034`. The shared case reader validates publisher terms before subtracting those three fields and keeps component labels, queries and next actions. It applies the following class maxima with 15 percent margin; argument, disabled-source, bad-reply, too-large and absent outcomes have separate smaller ceilings. These synthetic fixtures establish command mechanics and supply no native clinical positive.

| Synthetic command class | Full/comparable observed bytes | Full/comparable ceilings |
|---|---:|---:|
| Search | 824 / 709 | 948 / 816 |
| Card | 1301 / 1186 | 1497 / 1364 |
| Devices | 1079 / 964 | 1241 / 1109 |
| Narrative | 793 / 678 | 912 / 780 |
| Arguments | 143 / 143 | 165 / 165 |
| Missing snapshot | 217 / 217 | 250 / 250 |
| Retained D03 error-body adapter fixture | 637 / 522 | 733 / 601 |
| Failed search | 316 / 201 | 364 / 232 |
| Failed card | 410 / 295 | 472 / 340 |
| Failed devices | 418 / 303 | 481 / 349 |
| Failed narrative | 434 / 319 | 500 / 367 |



## Complete legacy command and option accounting

This is source inspection at the pinned legacy revision. Dispositions apply the PM review; they are not completed ports. Both search and `drug adverse-events` repeat many filters; the shared rows account for each explicitly. A dropped option returns an actionable usage error in future retirement cases and makes no source request.

| Legacy surface | Proposal and reason | Cases/limits |
|---|---|---|
| `search adverse-event` | Keep bounded one-source report search. | A01–A24; sources never auto-merge. |
| Positional `QUERY`, `-d/--drug` | Keep positional literal text; drop -d/--drug duplicate aliases. | F01 and retirement controls. |
| `--reaction` in search/helper | Keep typed reaction phrase filter on the analyzed MedDRA-term field, no exact-equality claim or rewriting. | F02/F03 known positive. |
| `--outcome death/fatal` in search/helper | Drop outcome aliases; --serious death means report seriousness, while native reaction outcome stays on each reaction. | F13 versus reactionoutcome5; no causal claim. |
| `--outcome hospitalization/hospitalisation/disability` in search/helper | Drop incorrect mappings. Legacy maps these to reactionoutcome1/3, which mean recovered/resolved and not recovered/not resolved. | A14; native report hospitalization/disabling fields only. |
| `--serious [any]` in search/helper | Port explicit any as native serious:1; require an explicit value; also restore lifethreatening, disabling, congenital-anomaly and other with their exact native flags. | The parity benchmark restores the native flags; F13/F14 remain; aliases fail before I/O and name accepted spellings. |
| Seriousness death, hospitalization/hospitalisation, lifethreatening/life-threatening, disability/disabling, congenital/congenital_anomaly, other | Keep canonical any, death, hospitalization, lifethreatening, disabling, congenital-anomaly and other. Reject aliases such as hospitalisation, life-threatening, disability, congenital and congenital_anomaly before I/O. | Exact request controls cover every canonical flag; alias refusals name the accepted spellings. |
| `--date-from`, alias `--since`, in search/helper | Rename to --received-from; strict full date, first FDA receipt. | F15 both-bound positive; one-sided builder synthetic. |
| `--date-to`, alias `--until`, in search/helper | Rename to --received-to; strict full date; reject reversed bounds. | F15 both-bound positive; one-sided builder synthetic. |
| `--suspect-only` in search/helper | Drop unsafe cross-object promise; retain each drug's own role. | A16/F16; exact identity and nested scope need proof. |
| `--sex m/male/f/female` in search/helper | Drop demographic search filter; preserve native record fields. | F17 mechanics do not justify a harvested clinical feature. |
| `--age-min`, `--age-max` in search/helper | Drop unitless age filters; retain source age and unit on card. | F17 plus retirement controls. |
| `--reporter` physician/doctor/pharmacist/other_health_professional/health-professional/lawyer/consumer/patient | Drop filter; retain optional native qualification/country on card; no reporter section. | F01/I01 inventory proof. |
| `--count reaction/reactionmeddrapt/patient.reaction.reactionmeddrapt[.exact]` in search/helper | Drop all count options and bucket fields; the harvest has no distinct most-reported-reactions question. Seriousness count duplicates F13 total. | F19/F22 stay investigation evidence only; retired options zero I/O. |
| Count medicinalproduct[.exact], openfda.generic_name[.exact], openfda.brand_name[.exact] | Drop all count options and bucket fields; the harvest has no distinct most-reported-reactions question. Seriousness count duplicates F13 total. | F19/F22 stay investigation evidence only; retired options zero I/O. |
| Count patientsex, patientonsetage, serious, seriousnessdeath/hospitalization/lifethreatening/disabling/congenitalanomali/other, reactionoutcome, primarysource.qualification | Drop all count options and bucket fields; the harvest has no distinct most-reported-reactions question. Seriousness count duplicates F13 total. | F19/F22 stay investigation evidence only; retired options zero I/O. |
| Fake count total/meta.total/results.total/meta.results.total | Preserve explicit refusal. Native `meta.results.total` is metadata, not an aggregation field. | Retirement usage case, zero I/O. |
| `--type faers` | Rename to explicit `--source faers`. | A01–A17. |
| `--type device/devices/maude` | Rename to explicit `--source maude`. | A18–A20. |
| `--type recall/recalls/enforcement`, `--classification` | Drop from adverse-event entity; medical device recalls use `search diagnostic --source fda-device-recall` under the benchmark ruling. | No recall capture or parity claim. |
| `--source faers/vaers/all`, default all | FAERS default; MAUDE and VAERS explicit. Drop all and automatic vaccine detection. | VAERS supports MMR, COVID-19 and fourteen separately selected native influenza types. Broad influenza text lists the qualified codes before I/O. |
| `--device` | Replace with positional literal MAUDE generic-name search; wildcard expansion is dropped. | D01–D04. |
| `--manufacturer`, `--product-code`, `--problem` | MAUDE accepts manufacturer phrases on `device.manufacturer_d_name`, repeatable exact product codes on `device.device_report_product_code` joined with OR, and problem phrases on native `product_problems`. Device text is optional with one filter. FAERS and VAERS refuse these filters. A report may list several devices, so filters may match different devices. | Native filter replay and source boundaries in `maude/filters.rs`. |
| Device serious any/death/injury | Drop device seriousness filter; keep native event_type on card/header. | D02 is evidence only; no unsupported CLI promise. |
| `-l/--limit`, `--offset` in search/helper | Keep long --limit/--offset for report pages; drop -l. Sections use section-limit/section-offset. | F01/F20 and synthetic bounds/window controls. |
| `get adverse-event REPORT_ID` | Keep native ID with default FAERS or explicit MAUDE; no --version or speculative source probing. | F21/D05; multi-version get unproved. |
| Get `reactions` | Keep independent reaction page including native outcome. | A01–A17. |
| Get `outcomes` | Rename to card seriousness plus each reaction's own outcome; do not mislabel flags as reaction outcomes. | A14. |
| Get `concomitant` | Rename to complete drugs page preserving every role; no name selection. | A10/A16. |
| Get `guidance` | Drop generated text-derived drug/indication guidance. Existing drug actions need exact source IDs. | No diagnostic/treatment inference. |
| Get `all` | Drop unbounded all-sections output. | Named pages retain complete collections. |
| `drug adverse-events NAME` | Drop duplicate helper and its FAERS404→ClinicalTrials.gov fallback. Typed exact drug-ID pivot may be added only after native identity join proof. | Existing drug ledger already marks adverse-event ownership; no duplicate leaf. |
| Drug card `top_adverse_events` and `faers_query` enrichment | Drop hidden name-based aggregate calls from a default drug card; explicit source-ID pivot requires proved mapping. | Existing drug ledger owns the card; A17 does not justify unconditional enrichment. |
| Helper `--type`, `--count`, all filter/page flags above | Same explicit dispositions as their search counterparts, including unused/misleading type routes. | No orphan option retained because helper existed. |
| `cvx sync` | Drop complete sync route from this slice; no read-triggered downloads or MVX fetch. | No proved WONDER mapping; retirement points to explicit VAERS report summary search. |
| VAERS summary total/seriousness/age/top reactions and source outcomes | Port native independent symptom, seriousness and age tables with source outcomes; drop the name bridge. | Approved MMR and COVID-19 captures support the VAERS contract below; earlier V01–V03 denials do not retire this source. |
| Global `--json/-j`, Markdown sections and section-parser acceptance of trailing json flags | Use existing `--format json\|md`; schema/CLI/MCP share typed tree. No section parser accepts format flags as clinical sections. | Existing shared formatting contract plus generated help cases. |

## Retirement messages and unanswered scope

Retired --drug/-d says use positional TEXT. Retired outcome aliases say use --serious death or hospitalization for report flags and reactions for native outcomes. Date aliases name --received-from/--received-to. Unsupported demographic/suspect/reporter/count/raw-query flags say this filter is not supported and make no request. Retired all/guidance says choose a named section. Retired helper says use search adverse-event TEXT. VAERS source selection uses the summary contract below. CVX routes and `cvx sync` say “Vaccine identity lookup and sync are unavailable; search vaccine report summaries with `search adverse-event --source vaers`.” Neither message claims vaccines are safe or redirects silently to FAERS. All retirement spellings get recorded zero-I/O cases.

A24, F18 and malformed/timeout/wrong-identity controls are explicitly synthetic. D03 is the real empty provider reply. A04/A15/A21/A22 and the combined/joint questions named above remain unanswered. No new provider call, alternate code, user-agent workaround or capture retry is authorized by this revision.

## Ian's questions

Ian authorized the bounded retry with the legacy client user agent. MMR and COVID-19 returned aggregate tables, and the PM approved explicit VAERS summary search. CVX remains dropped because no native WONDER mapping was proved. Influenza requires a separate code investigation. These rulings do not establish complete 0.9 vaccine coverage.

## Exact FAERS report execution

The get adapter accepts one report whose `safetyreportid` equals the requested digit-only ID. It rejects a wrong ID, multiple returned records and a 200 error body before ordinary cache admission. There is no version choice. Native drug roles remain on each original drug row; reaction arrays remain independent. Identifier pages preserve original spelling and order at the absolute drug index and offer no drug pivot. An absent identifier field returns absence; an explicitly empty array returns an empty complete page.

An initial checked get retains the report through the existing expiring source snapshot store. Generated section and continuation commands carry `--snapshot`. These reads require no provider request and cannot switch to a newer report during a walk. Source and MedDRA reaction admission run before reading a retained payload. A missing or expired snapshot returns a restart command. The optional request field is an additive compatible schema change; existing schema identities remain valid.

The bounded source capture `j12975` made exactly four ID-only GETs from landed `82bac28e85fb1b30b6abc41761903cbc86d7c73d`: 26940567, 26581534, 13946660 and the original F18 no-match control. It returned 200/200/200/404 without retries, redirects or fallback. Positive records were respectively version 1 with 2 drugs/1 reaction, version 1 with 12 drugs/4 reactions, and version 3 with 19 drugs/5 reactions. The publisher date was 2026-07-30; original retrieval occurred on 2026-10-25. The F18 string is outside the digit-only product grammar, so its 404 is source evidence only. A separate synthetic numeric-ID reply proves runtime no-match behavior. The capture archive is `data/biomcp-redux/0159-faers-get-native-proof/j12975-native-capture.tar.gz`, SHA-256 `f4bdb731a81a80384fa28d2c619425f80ff4dba34a0144815b5dc7215073511f`, with 37 independently verified manifest files.

Native product replay `j13004` used runtime revision `d6e94cdb3ca9665f1e60435579fc7398721f2ed9` and paired 1,004 CLI and MCP envelopes. Its local fixtures retained the three new exact-get records, the older F01 report witnesses and older F10 report 26749927 version 1. Older search records were explicitly selected into local get fixtures; this does not establish a fresh exact-get positive. Original source captures and local fixture deliveries remain separate. The run made zero provider requests, delivered nine initial local report fixtures and delivered zero further replies during retained paging. It walked all 19 drug rows, all 22 older reaction rows, all seven identifier kinds, the 2,087 package-NDC rows and the 841 SPL arrays without index gaps or duplicates.

The following are compact UTF-8 envelope maxima across the native replay cases. Comparable bytes exclude only the validated service outcome `use`, `terms` and `terms_url` fields. They retain component terms, query selectors, labels and report context. Every native case also has its own actual-plus-15-percent ceiling in the retained result record. These are local replay measurements; they do not measure provider speed.

| Output | Full bytes | Comparable bytes | Full ceiling | Comparable ceiling |
|---|---:|---:|---:|---:|
| Card | 1459 | 1344 | 1678 | 1546 |
| Drugs | 4120 | 4005 | 4738 | 4606 |
| Reactions | 1528 | 1413 | 1758 | 1625 |
| UNII | 760 | 645 | 874 | 742 |
| RxCUI | 948 | 833 | 1091 | 958 |
| Product NDC | 991 | 876 | 1140 | 1008 |
| Package NDC | 1006 | 891 | 1157 | 1025 |
| SPL ID | 1102 | 987 | 1268 | 1136 |
| SPL set ID | 1118 | 1003 | 1286 | 1154 |
| Application number | 1016 | 901 | 1169 | 1037 |

Token estimates use ceiling(bytes / 4). These ceilings apply only to the new get outputs and do not relax earlier search, help or shared tool-description ceilings.

The retained product archive is `data/biomcp-redux/0159-faers-get-product-proof/j13004-product-proof.tar.gz`, SHA-256 `8d1644b8698ce5d4905023bccb64e964f63e803cc38532492655d2732be72af7`, 4,572,717 bytes with 7,527 independently verified files. It includes raw CLI stdout/stderr and MCP frames, recorder files, original archive provenance, per-case byte/token results, category maxima and the two preserved local harness failures. The failed attempts concerned nested header selection and an omitted terminal `next`; neither made a provider request or changed product code.

Routine synthetic cases in `faers_get.rs` use their own `j13008` measured compact CLI/MCP maxima plus 15 percent. The shared source-terms validator performs the comparable deduction. The maxima include card 1215/1100 full/comparable bytes, drugs 1453/1338, reactions 1403/1288 and identifier defaults from 692/577 to 924/809. Missing identifier arrays remain separate from empty complete pages. Failed identity, disabled-source, absence and argument envelopes have separately measured bounds in the case code. These synthetic bounds do not replace the native measurements above.


## Shipped command and option ledger

This ledger records FAERS, MAUDE and the explicit VAERS summaries through CDC WONDER, with fixed zero-I/O migrations. CVX identity lookup and sync are dropped because no WONDER mapping was proved. The drug slice owns the `top_adverse_events` and `faers_query` card removals. Full same-question 0.9 execution remains owned by 0098.

<!-- parity-accounting -->
| 0.9 leaf command | Command outcome | Ported or renamed options | Dropped options | Deferred options | Undecided options | Ruling |
| --- | --- | --- | --- | --- | --- | --- |
| `search adverse-event` | Ported or renamed | `--reaction`, `--serious`, `--date-from`, `--date-to`, `--type`, `--device`, `--manufacturer`, `--product-code`, `--limit`, `--offset`, `--source` | `--drug`, `--outcome`, `--suspect-only`, `--sex`, `--age-min`, `--age-max`, `--reporter`, `--count`, `--classification` | — | — | Use literal TEXT with FAERS by default or explicit MAUDE. Receipt dates replace date aliases; native report seriousness and all six specific flags remain. MAUDE device text, manufacturer and native product problem use phrase matches; repeated product codes use exact OR matches. Device text is optional with a MAUDE filter. Drug recall routes use search drug with source fda-drug-recall, native classification, firm, status, reason and initiated-from/to dates; exact D-NNNN-YYYY recall numbers support get drug. Device recall routes use search diagnostic with source fda-device-recall; this covers every medical device. VAERS and CDC WONDER provide explicit MMR, COVID-19 and separately selected native influenza report summaries; CVX identity mapping remains dropped. |
| `get adverse-event` | Ported or renamed | — | — | — | — | Get one exact native report with explicit source and independent bounded sections. Historical versions remain unproved; guidance and unbounded all retire. |
| `drug adverse-events` | Dropped by ruling | — | `--reaction`, `--outcome`, `--serious`, `--date-from`, `--date-to`, `--suspect-only`, `--sex`, `--age-min`, `--age-max`, `--reporter`, `--count`, `--type`, `--limit`, `--offset` | — | — | Use search adverse-event TEXT. The duplicate helper and its FAERS-to-ClinicalTrials.gov fallback retire; no trial result substitutes for reports. |
<!-- /parity-accounting -->

The long-option denominator excludes positional text, aliases, sections and global formatting. The complete source-inspection table above supplies the individual alias and count-field inventory. `-d` and `--drug` point to literal TEXT; `-l` points to `--limit`; `--since` and `--until` point to first-receipt bounds. Outcome spellings and unsupported seriousness values return fixed guidance without I/O. Native report flags never replace reaction outcomes. Demographic, suspect-only, reporter and aggregation filters retire. Manufacturer, product-code and problem filters require MAUDE. Global `--json` and `-j` point to `--format json`; Markdown remains shared formatting.

| Old section or behavior | Shipped outcome and case owner |
| --- | --- |
| `reactions` | Independent native reaction rows and outcomes; `faers_get.rs`. |
| `outcomes` | Card report flags and each reaction's own outcome; fixed migration in `adverse_event_retirements.rs`. |
| `concomitant` | Complete `drugs` page with every original role; `faers_get.rs` and fixed migration. |
| `guidance`, `all` | Retired with named-section guidance; no generated clinical inference or unbounded fan-out. |
| Hidden drug card `top_adverse_events`, `faers_query` | Drug ledger owns removal; no duplicate adverse-event accounting row. |
| Recall routes and classification | Use `search diagnostic --source fda-device-recall` for every medical device; counts are recall records. |
| VAERS, CDC WONDER and vaccine summary/source outcomes | Ported for explicit MMR and COVID-19 independent tables; `vaers/` cases own the contract. Influenza remains unavailable pending a current code. |
| CVX identity lookup and sync | Dropped with fixed zero-I/O guidance to VAERS summaries; no proved WONDER mapping. |

## Question and case coverage limits

The harvested A01–A24 questions are a question bank, not twenty-four successful product executions. A01–A03, A05–A14 and A16–A17 have the source-report facts described above; only captured query expressions establish native selector positives. A04, A15, A18, A21 and A22 remain unanswered as asked. A19–A20 retain MAUDE native report/device/narrative facts without a causal claim. A23 concerns the limits of an exact empty query; A24 is an explicitly synthetic failure control.

Native F01/F20/F02/F03/F23/F13/F14/F15 product pairs and per-case ceilings live in j12931. F21 and the exact-get witnesses, the older F01/F10 retained report fixtures, all seven identifier pages and the long complete walks live in j13004. The D01/D05/D02/D04 thirteen product pairs and ceilings live in independently verified j13024 output. F04–F12 and F16–F17 remain bounded investigation captures unless explicitly used by those retained get cases; F19/F22 count queries remain investigation only. F18 is source absence evidence outside product ID grammar. D03 is an error-body adapter fixture, not an executed SCC CLI positive. I01 is source identity evidence, not a name-based drug join. V01–V03 do not supply vaccine product positives.

Routine executable cases are the adverse-event YAML files and native runner cases `faers_search.rs`, `faers_get.rs`, `maude.rs`, `maude_policy.rs`, `adverse_event_registry.rs`, `adverse_event_retirements.rs`, `adverse_event_failures.rs` and `vaers/`. Their synthetic bodies prove mechanics, source admission, independent rows, empty versus absent, literal boundaries, fixed migrations and failures. Native product archives remain outside Git. Raw terminology is never copied into routine fixtures. Every successful report envelope uses its owning measured full/comparable ceiling; retirement and argument envelopes use their separate error bounds. The external case definitions and archives remain separately identified from routine executions.


The earlier help measurement in j13144 uses generated source-owned section text and explicit death/hospitalization values. Search help measures 1663 compact bytes and get help 2242. Their PM-approved compact ceilings are 1913 and 2579, with stdout ceilings 1914 and 2580 including the newline. The PM approved this measured help growth in the ticket ruling of 2026-10-25. The MCP description derives options and source choices from the same schemas and stays within its existing ceiling. Whole-tree help measures 55079 stdout bytes; its standing actual-plus-fifteen-percent ceiling is 63341.

The two synthetic exact-get examples in j13131 measured 1068/953 and 850/735 full/comparable compact bytes. Their cases enforce comparable bytes with fifteen percent headroom through the shared validated source-terms deduction; component terms remain counted. The same real recorder supplied the synthetic exact-ID reply with four local example deliveries in separate environments and zero provider requests. The earlier failed setup used the wrong metadata limit in the invented body, then corrected it to the requested five. The literal-boundary case in j13144 measured argument errors at 143 bytes and offline literal queries at up to 327 full / 212 comparable bytes; its fixed ceilings are 165 and 377 / 244 respectively.


### Generated schema output

j13148 invoked each schema command and checked its parsed value against the generated artifact. CLI discovery prints the complete formatted schema; MCP receives the same parsed schema value. These are schema-document sizes, separate from clinical reply envelopes. No provider call runs. The existing generated-schema drift and compatibility contract owns these artifacts.

| Schema artifact | CLI bytes | Compact schema bytes |
| --- | ---: | ---: |
| adverse-event-search | 2972 | 2116 |
| adverse-event-get | 3171 | 2265 |
| adverse-event-page | 23165 | 14846 |
| adverse-event-faers-card | 27426 | 17492 |
| adverse-event-maude-card | 24279 | 15629 |
| adverse-event-drugs | 30234 | 19320 |
| adverse-event-reactions | 24326 | 15668 |
| adverse-event-identifiers | 30241 | 18855 |
| adverse-event-devices | 28054 | 17970 |
| adverse-event-narrative | 23870 | 15386 |

The case audit distinguishes source registry health contracts from report commands. Health remains a shared source-controls surface rather than an adverse-event report budget. Public type contracts validate schemas and native shapes; they are not claimed as CLI or MCP executions. The admitted report-command cases retain measured envelope bounds in their owning runners. Every new help example executes through both surfaces in the existing example contract. This audit adds no outside-repo case definition or new source recipe.


## VAERS summary contract

The explicit VAERS search accepts `MMR`, `COVID19`, the display name `COVID-19 vaccine`, or one qualified native influenza code, matched exactly without case. The published influenza code and label registry lives in `VAERS_INFLUENZA`. A broad `influenza`, `flu` or `FLU` selector fails before I/O and lists its native codes and labels. Each code selects its own tables; there is no aggregate influenza selector, vaccine alias bridge or automatic source selection. One report may list several vaccines, so counts from separate vaccine tables must not be summed. `seriousness` and `age` each select an independent native table. The default table contains reported symptoms. Rows retain native order and page locally through the retained snapshot. A missing or expired continuation returns restart guidance without contacting WONDER.

CDC WONDER declares `restricted` use under [CDC WONDER data use restrictions](https://wonder.cdc.gov/datause.html). Statistical use only and no re-identification apply. The source stays enabled by default; disabling the source or restricted group applies before network, reply cache, replay or retained snapshots. The registry owns its compatibility user agent; other sources keep the transport default. Its quota belongs to shared transport.

Each native row keeps its label, Events Reported and Percent cells as original attribute maps. The native measure codes and labels accompany each table. The source message report total remains separate from the native footer, which can count multiple symptom occurrences per report and exceed the message total. No row sum, denominator sharing, inferred row code or suppressed-cell reconstruction occurs. Each table preserves dataset vintage, applied response footnotes and evaluated caveats, and page messages. The dataset's unselected documentation catalog does not become applicable response notices.

The clinical reviewer approved this interpretation notice from the [CDC WONDER VAERS guide](https://wonder.cdc.gov/vaers.html) and [VAERS data guide](https://vaers.hhs.gov/data/dataguide.html): Counts are VAERS reports, not people. Reports do not show that a vaccine caused an event and cannot measure risk or compare vaccine safety. A report may mention several symptoms and several vaccines. Each table retains its own source total and percentages. Reported symptoms and reports classified as serious preserve source meaning; no diagnoses or unique people are inferred.

### VAERS synthetic command costs

The command case helper `vaers/budget.rs` enforces each observed envelope size plus fifteen percent. The local synthetic rows exercise separate totals and suppression; these measurements do not describe provider latency or native table costs. Full bytes retain registry terms. Comparable bytes remove only the verified registry use, terms and terms URL fields.

| Synthetic response | Full compact bytes | Comparable compact bytes |
| --- | ---: | ---: |
| Symptoms first page | 1993 | 1881 |
| Symptoms continuation | 1835 | 1723 |
| Seriousness | 1918 | 1806 |
| Age | 1919 | 1807 |
| 403 HTML | 294 | 182 |
| 429 | 298 | 186 |
| 500 | 297 | 185 |
| Timeout | 303 | 191 |
| Broken XML | 295 | 183 |
| Disabled | 295 | 183 |
| Disabled retained table | 373 | 261 |
| Expired snapshot | 224 | 224 |

Search help measured 2542 compact bytes and 2543 stdout bytes. `adverse-event/help.yaml` owns its approved ceiling. Exact report get help retains only FAERS and MAUDE and measured 2243 stdout bytes. The MCP command description measured 1920 bytes under its existing ceiling; snapshot tokens remain in detailed help and emitted continuations. Whole help and the Source enum retain their existing ceilings.

The fixed native geographic selection is `D8.V12` State / Territory, value `00`, with the native label “The United States/Territories/Unknown”. Each table exposes that variable and selected value. This is not a worldwide result or a vaccinated population denominator. There is no new geographic selector. The clinical reviewer confirmed the exact label and selection expansion from both retained native datasets.

The adapter decoder enforces `MAX_XML` before its native DOM parse and cache admission. Shared transport `max_body` independently bounds wire reads. The valid oversized synthetic table proves the decoder limit; an HTML denial or malformed table never becomes a zero-report answer.

The retained MMR symptom table has 4074 physical rows including the footer and 4073 pageable rows. Its native report-total message is 84,004 and its symptom footer is 269,356 / 320.65%. These remain separate source values, as do the independently returned seriousness and age totals.


### Native influenza selection

The published WONDER request form supplies the native influenza vaccine types in `VAERS_INFLUENZA`. The clinical reviewer checked their exact codes and labels against that form. `FLUX(SEASONAL)` retains the publisher's no-brand-name label; `FLUX(H1N1)` retains its unknown-manufacturer label. Those qualifications identify different native selections and cannot be dropped from a request. The implementation lists the registry when a caller asks for broad influenza results instead of combining its entries.

The bounded capture `j13562` returned HTTP 200 aggregate tables for all three independent groupings of every registry entry. It sent 42 requests with no retries or fallbacks. The original XML, request receipts and per-table source metadata remain outside Git in Yellow archive `data/biomcp-redux/0183-vaers-influenza-native-capture/2290-vaers-influenza-native-capture.tar.gz`, SHA-256 `9b39062a8446db31523db7312ee7cbb0e350ac4ad8a5b642d5640c691f253b99`. Its 429 members include 428 verified content hashes. This source capture proves table availability; native product measurements are recorded separately from synthetic command costs.

### VAERS retained native command costs

The independent retained proof at revision `037b321dc552391eafa002a4ebf9ddecf94c340f` checked 26 successful CLI/MCP pairs, two disabled-policy pairs and six external schema cases. It checked each table against its own original XML, including scope, native cells, source messages, applied notices and footer. Symptoms used representative first, second and terminal pages; the complete paging walk remains synthetic. The completion made no provider requests or new recorder deliveries. Local replay timings do not measure provider latency.

| Native response | MMR actual full/comparable | MMR ceilings | COVID19 actual full/comparable | COVID19 ceilings |
| --- | ---: | ---: | ---: | ---: |
| Default symptoms, five rows | 7977 / 7865 | 9174 / 9045 | 8650 / 8538 | 9948 / 9819 |
| Second symptom page, five rows | 8053 / 7941 | 9261 / 9133 | 8684 / 8572 | 9987 / 9858 |
| Terminal symptom page | 7722 / 7610 | 8881 / 8752 | 8276 / 8164 | 9518 / 9389 |
| Symptoms, explicit three rows | 7849 / 7737 | 9027 / 8898 | 8507 / 8395 | 9784 / 9655 |
| Symptoms, explicit one row | 7683 / 7571 | 8836 / 8707 | 8330 / 8218 | 9580 / 9451 |
| Default seriousness, two rows | 7996 / 7884 | 9196 / 9067 | 8637 / 8525 | 9933 / 9804 |
| Seriousness, explicit three-row limit | 8024 / 7912 | 9228 / 9099 | 8665 / 8553 | 9965 / 9836 |
| Seriousness, explicit one row | 8151 / 8039 | 9374 / 9245 | 8795 / 8683 | 10115 / 9986 |
| Default age, five rows | 7858 / 7746 | 9037 / 8908 | 8490 / 8378 | 9764 / 9635 |
| Second age page, five rows | 7941 / 7829 | 9133 / 9004 | 8587 / 8475 | 9876 / 9747 |
| Terminal age page, three rows | 7603 / 7491 | 8744 / 8615 | 8242 / 8130 | 9479 / 9350 |
| Age, explicit three rows | 7745 / 7633 | 8907 / 8778 | 8378 / 8266 | 9635 / 9506 |
| Age, explicit one row | 7591 / 7479 | 8730 / 8601 | 8224 / 8112 | 9458 / 9329 |

The approved default remains five rows. Each native external ceiling is its own measured compact actual plus fifteen percent, rounded up. These external bounds remain separate from the synthetic case ceilings above and from existing help, discovery and source-enum caps. Full bytes describe compact envelopes. Comparable bytes remove only the validated registry use, terms and terms URL fields, a 112-byte difference. Raw CLI stdout adds one newline byte; MCP frames include protocol wrapping. Explicit limits remain visible in the query, and a one-row seriousness page adds continuation metadata. Preserved native notices dominate response size: reducing the symptom page from five rows to one saves only 294 bytes for MMR and 320 bytes for COVID19.

The verified archive is `data/biomcp-redux/0182-vaers-product-proof/2291-vaers-offline-product-proof-037b321d.tar.gz` on Yellow, SHA-256 `f7b0e6d4a9661c8a80738d40d6908cdceb50f6e39e210eed84f008571cd2cd78`, with 287 exact members and 286 verified content hashes. Its measurements and raw CLI/MCP outputs remain outside Git.


### Native influenza command costs

The retained product replay `j13591` at revision `a0fa49078211027c880e45a4e239b008036d59be` recorded each of the 42 captured tables once through localhost, with zero provider requests or retries. It checked all 42 default CLI/MCP pairs and eight representative continuation pairs against the original native fields. The external Rust case runner validated all 50 envelopes across 42 cases with the current source-terms comparison. Disabled-policy and broad-selector checks preserved refusal before I/O.

The default remains five rows under the existing ruling. The section maxima below summarize the fourteen observed default commands per section; they do not bound unseen native values. Seriousness returned its two complete native rows. Each external case owns its measured full and comparable compact ceiling, calculated as its own actual plus fifteen percent, rounded up. These bounds do not raise any synthetic, help or discovery ceiling.

| Native influenza response | Full/comparable compact bytes | Full/comparable ceilings |
| --- | ---: | ---: |
| Default symptoms, observed maximum | 8067 / 7955 | 9278 / 9149 |
| Default seriousness, observed maximum | 8071 / 7959 | 9282 / 9153 |
| Default age, observed maximum | 8045 / 7933 | 9252 / 9123 |
| FLU(H1N1)-symptoms-second | 8087 / 7975 | 9301 / 9172 |
| FLU(H1N1)-symptoms-terminal | 7621 / 7509 | 8765 / 8636 |
| FLU(H1N1)-age-second | 7979 / 7867 | 9176 / 9048 |
| FLU(H1N1)-age-terminal | 7630 / 7518 | 8775 / 8646 |
| FLUX(H1N1)-symptoms-second | 8095 / 7983 | 9310 / 9181 |
| FLUX(H1N1)-symptoms-terminal | 7759 / 7647 | 8923 / 8795 |
| FLUX(H1N1)-age-second | 7984 / 7872 | 9182 / 9053 |
| FLUX(H1N1)-age-terminal | 7638 / 7526 | 8784 / 8655 |

Comparable sizes remove only the validated registry `use`, `terms` and `terms_url` fields. CLI stdout adds one newline byte; MCP frame measurements include protocol wrapping. All native notices, independent source totals, exact labels and qualified selectors remain present. Representative second and terminal pages cover two native selections; the complete walk remains covered by synthetic cases. No three-row or one-row native comparison ran for influenza. Local replay timings do not measure provider latency.

The archive on Yellow is `data/biomcp-redux/0183-vaers-influenza-product-proof/j13591-native-proof.tar.gz`, SHA-256 `6546dea01779d51757a46cb1461f9f04bb20aec38ab048eea4de2319cd85f8aa`. Its 648 members include 647 verified content hashes. The original per-case measurements, external caps and raw CLI/MCP outputs remain in that archive.
