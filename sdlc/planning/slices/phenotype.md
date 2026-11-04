# Phenotype slice investigation and proposed design

This investigation starts from `85e6ba864c3f13ec50432a58670ec2af13e12857`. It proposes an entity interface for human phenotype terms and their source associations. It changes no product code. The disease slice already keeps Monarch disease-to-phenotype annotations. This slice must reuse those rows and their component terms rather than create a competing disease annotation model.

## Harvested questions and accepted source answers

The questions below adapt explicit source features and source workflows into user questions. They are not patient cases or quotations of questions submitted by those authors. A term lookup answers terminology. An association lookup answers what its publisher asserts. Neither establishes a diagnosis, causation, penetrance or an individual risk. Clinical acceptance of the question bank is separate from native transport measurements.

Sources: [NF1 GeneReviews](https://www.ncbi.nlm.nih.gov/books/NBK1109/), [PTEN GeneReviews](https://www.ncbi.nlm.nih.gov/books/NBK1488/), [VHL GeneReviews](https://www.ncbi.nlm.nih.gov/books/NBK1463/), [Birt-Hogg-Dube GeneReviews](https://www.ncbi.nlm.nih.gov/books/NBK1522/), [HPO 2024, PMID 37953324](https://pubmed.ncbi.nlm.nih.gov/37953324/), and [Monarch 2024, PMID 38000386](https://pubmed.ncbi.nlm.nih.gov/38000386/).

| Case | Source-derived question | Source answer to retain | Source |
| --- | --- | --- | --- |
| P01 | Which HPO terms describe cafe-au-lait macules? | Publisher term candidates, their exact IDs and definitions; NF1 source names the feature. No syndrome inference. | NF1 |
| P02 | What does neurofibroma mean in HPO? | Exact returned term and definition, with synonyms as a separate disclosure layer. | NF1 |
| P03 | Which term describes plexiform neurofibroma? | Exact term candidates; retain a more specific term separately from its ancestors. | NF1 |
| P04 | Which term describes axillary freckling? | Source term candidates and exact IDs. A missing literal hit is not an absent clinical feature. | NF1 |
| P05 | What does macrocephaly mean? | Source definition, not a locally chosen head-circumference threshold. | PTEN |
| P06 | Which HPO term describes gastrointestinal polyps? | Source candidates; no automatic equivalence between generic and hamartomatous polyps. | PTEN |
| P07 | Which HPO term describes lipoma? | Source term identity and definition. | PTEN |
| P08 | What term describes developmental delay? | Source candidates and definition; no automatic substitution of intellectual disability. | PTEN; HPO |
| P09 | What term describes retinal hemangioblastoma? | The literal Monarch search is empty; HPO returns candidates, including retinal hemorrhage. No accepted equivalent term is established. | VHL |
| P10 | Which term describes renal cysts? | Source candidates and exact IDs; no assumed subtype. | VHL |
| P11 | What does pheochromocytoma mean? | Source definition and term identifier. | VHL |
| P12 | Which term describes pancreatic cysts? | Exact source term and definition. | VHL |
| P13 | What term describes fibrofolliculoma? | Source identity and definition. | Birt-Hogg-Dube |
| P14 | Which term describes pulmonary cysts? | Source candidates and definitions. | Birt-Hogg-Dube |
| P15 | What does spontaneous pneumothorax mean? | Source definition; no inference that any observed pneumothorax is spontaneous. | Birt-Hogg-Dube |
| P16 | What does HP:0001250 mean? | Exact HPO concept identity, definition and native synonyms where supplied. | HPO; legacy documented example |
| P17 | Which native disease candidates resemble HP:0001250 and HP:0001263? | Native similarity ordering and score semantics only. A score is not evidence that every requested feature is annotated to the candidate. | Monarch; legacy documented example |
| P18 | What phenotype annotations does von Hippel-Lindau syndrome have? | Existing disease section, actual edge endpoints, provenance, evidence and qualifiers. No newly synthesized syndrome summary. | VHL; kept disease design D14 |
| P19 | What phenotype annotations does Li-Fraumeni syndrome have? | Existing disease section, association rows and native count. A source association does not establish a patient diagnosis. | Kept disease design D13; Monarch |
| P20 | What phenotype annotations does multiple endocrine neoplasia type 2 have? | Existing disease section preserves subtype edges and closure proof. A parent request must not relabel a subtype edge. | Kept disease design D18; Monarch |
| P21 | Which phenotype annotations are connected to the human VHL gene? | Source gene-to-phenotype associations, actual subject/object IDs, predicates and provenance, if the public tier returns them. | VHL; Monarch |
| P22 | Which phenotype annotations are connected to the human PTEN gene? | Same source association contract; no reverse inference from a disease name. | PTEN; Monarch |
| P23 | Which phenotype annotations are connected to the human NF1 gene? | Same source association contract; preserve many rows for one term when provenance differs. | NF1; Monarch |
| P24 | Does an unknown term or an unavailable source mean no matching syndrome exists? | No. An unknown exact term is absent; a successful empty search is empty; a source failure is failed or partial. These states make no clinical absence claim. | HPO/Monarch workflow; recorded failure case required |

### Implemented question mapping

The clinical reviewer accepted this mapping against the cited GeneReviews, HPO and Monarch sources above and the accepted native identity, complete-card and association reviews below. No new clinical interpretation changes a question's boundary. The result is nineteen answered questions, four partial questions and one unanswered question.

| Question | Status | Owning proof and limit |
| --- | --- | --- |
| P01 | Answered | HP:0000957; native search 02 and card 18. |
| P02 | Answered | HP:0001067; native search 03, card 19 and synonym sections. |
| P03 | Answered | HP:0009732; native search 04 and card 20. |
| P04 | Answered | HP:0000997; native search 05 and card 21. |
| P05 | Answered | HP:0000256; native search 06, card 22 and complete definition/comment. |
| P06 | Answered | Native search 07 and cards 23/24 retain HP:0200008 and HP:0004390 as separate candidates. |
| P07 | Answered | HP:0012032; native search 08 and card 25. |
| P08 | Answered | Native search 09 and card 26 retain HP:0001263 with its complete age-limited definition and comment. |
| P09 | Partial | Native search 10 proves Monarch's successful empty result. The retained HPO candidates establish no retinal hemangioblastoma equivalent. |
| P10 | Answered | Native search 11 and cards 27/28 retain HP:0005562 and HP:0000107 with distinct multiplicity. |
| P11 | Answered | HP:0002666; native search 12 and card 29. Obsolete HP:0001676 remains separate in witness 40. |
| P12 | Answered | HP:0001737; native search 13 and card 30 retain the pseudocyst comment. |
| P13 | Answered | HP:0030436; native search 14 and card 31. |
| P14 | Answered | Native search 15 and cards 32/33 retain HP:0005948 and HP:0032445 with their complete comments. |
| P15 | Answered | HP:0002108; native search 16 and card 34. |
| P16 | Answered | Exact HP:0001250 cards 00/01 and native synonym sections keep independent source answers. |
| P17 | Unanswered | Similarity is deferred by the PM ruling. Single-term association sections do not answer pair similarity. |
| P18 | Answered | Existing get disease MONDO:0008667 phenotypes; retained disease native proof preserves closure scope. |
| P19 | Answered | Existing get disease MONDO:0018875 phenotypes; retained disease native proof preserves source rows. |
| P20 | Answered | Existing get disease MONDO:0019003 phenotypes; retain the disease route's 100 closure-expanded rows separately from the investigated 48 direct rows. |
| P21 | Partial | Retained HGNC:12687 gene-keyed mapper proof preserves VHL rows, original NCBIGene:7428 and total 58. No gene-keyed product query route exists. |
| P22 | Partial | Retained HGNC:9588 gene-keyed mapper proof preserves PTEN rows, original NCBIGene:5728 and total 85. No gene-keyed product query route exists. |
| P23 | Partial | Retained HGNC:7765 gene-keyed mapper proof preserves NF1 rows, original NCBIGene:4763 and total 129; PMID:8317503 stays on its second row. No gene-keyed product query route exists. |
| P24 | Answered | Retained witnesses 35/36 plus states.rs and faults.rs distinguish successful empty, exact absence and failure. Empty and exact absence share status absent; source state/reason and result presence distinguish them. |

The numbered search/card witnesses belong to `crates/biomcp/tests/runner/phenotype/native.rs` and the unchanged term archive recorded below. `section_native.rs` owns the retained text/link sections. The [disease case map](disease.md#executed-harvested-case-map) and `crates/biomcp/tests/runner/disease_monarch/native.rs` own P18–P20. `crates/biomcp-adapters/src/monarch/native_contracts.rs` owns the P21–P23 retained mapper check; it does not supply a new query. The [PM phenotype ruling](../../decisions/2026-10-26-protein-pathway-study-rulings.md#phenotype) owns P17's deferral.

## Investigation boundaries

Native captures run through ticket jobs. The first bounded metadata pass names six requests, a single attempt, no redirects or fallback, a per-response byte bound and a stop on rate limiting or server errors. The raw capture remains outside Git. No 0.9 binary has been executed by this investigation. Its output budgets remain unmeasured unless verified containment can supply the exact native request identities.

The disease slice retired scored symptom-to-diagnosis ranking. A new source-native similarity operation therefore needs an explicit PM ruling after source semantics and the harvested research question are reviewed. Term discovery must not silently choose the first HPO candidate and feed it into a ranker.

## What would delight the agent

1. **What does it ask first?** "What is this feature called?" A literal term search returns HPO candidates, not diseases. An agent selects an exact HP identifier before following any association.
2. **What is the smallest useful header?** Exact ID, publisher label, source, native match metadata and native collection count. The complete definition belongs on the default term card.
3. **Which details come next?** Source synonyms with their relationship classes, cross-references, hierarchy and source disease/gene association pages. The explicit disease section already answers a known syndrome's annotations.
4. **Which identities must survive?** Exact HP IDs for terms, MONDO IDs for disease concepts and returned HGNC/NCBIGene IDs for human genes. Original endpoints and provenance remain on every association. No symbol or disease-name join is permitted.
5. **How do source disagreements appear?** HPO's flat synonym list and Monarch's classified synonym groups remain separate source answers. For HP:0001250 the HPO response lists Epilepsy as a synonym while Monarch marks it as related, not exact. No local clinical equivalence follows.
6. **What lets the agent stop?** An exact term definition with source attribution answers a terminology question alone. A successful empty search answers only that literal source query. An association page answers only the captured publisher scope.
7. **What makes the answer trustworthy?** Publisher URL, retrieval time, ontology release when actually supplied, original assertion provenance, native predicate, evidence, negation and qualifier presence. A retrieval date is never invented as an ontology version.
8. **What does similarity add?** It answers P17's explicitly requested research similarity. It must be opt-in and source-native if approved. It cannot replace ontology search, choose a diagnosis or claim that every input term appears on every returned disease.

## Proposed types, schema and commands

The proposal separates term discovery, exact term identity and associations. The consumer layer owns checked requests, cards, pages and typed actions. The neutral layer reuses `Curie`, source concepts, attribution and the disease slice's association record. A new identifier wrapper accepts the published HP identifier grammar; a MONDO or gene identifier is a usage error for an exact phenotype card. No patient, clinical score, symptom parser or diagnosis type is introduced.

`PhenotypeHeader` holds HP ID and native label plus optional source match score, matched field and match type. Scores retain the publisher's meaning. `PhenotypeCard` holds requested ID, actual returned ID, label and complete native definition; optional comment and source concept metadata remain available as described below. `PhenotypeSynonym` retains a publisher relationship kind when provided; an unclassified HPO synonym must not acquire Monarch's classification. `PhenotypeAssociationPage` reuses native row identity, actual subject/object and original endpoints, predicate, evidence/publications, provenance, negation, frequency/onset/sex and other source qualifiers, and source totals. A gene association remains an association, not a causal clinical statement. Omitted, null and empty collections follow the established source-presence rules; no nonempty text invariant may silently discard a native empty definition or comment.

| Proposed operation | Default answer | Additional disclosure and cases |
| --- | --- | --- |
| `search phenotype TEXT --source monarch --limit N --offset N` | Literal source search restricted at the provider to `namespace=HP` and phenotype category; default five headers in native order. | P01–P16, P24. Validate the native limit, offset, total and every returned namespace. Retain native match metadata. An unrelated namespace is a source problem, never a local reclassification. |
| `get phenotype HP:ID --source hpo` | HPO term label and complete definition, native comment, attribution and published identity. | P01–P16. Direct HPO supplies the source comment and publication references absent from the earlier disease annotations. No follow-up metadata request is implicit. |
| `get phenotype HP:ID --source monarch` | Independent Monarch term card, complete description and native deprecation fact when supplied. | P16. No merged card or automatic fallback. Keep source hierarchy and association counts distinct from HPO descendant counts. |
| `get phenotype HP:ID synonyms --source hpo\|monarch` | Bounded native synonym rows. | P02, P05, P16; preserve Monarch exact/broad/narrow/related distinctions and HPO's unclassified list. |
| `get phenotype HP:ID xrefs --source hpo\|monarch` | Bounded native cross-reference rows and actual predicates where supplied. | P05, P16. Cross-reference presence alone proves no equivalence. |
| `get phenotype HP:ID hierarchy --source monarch` | Native direct parents/children with exact IDs and labels. | P03, P16. Descendant totals are explicitly publisher counts, not patient prevalence. No local graph traversal. |
| `get phenotype HP:ID diseases --source monarch` | Source disease-to-phenotype rows with requested HP beside actual endpoints, default one row. | P17–P20; `--section-limit`, `--section-offset`; preserve source order and native count. Exact/direct mode requires positive native proof. |
| `get phenotype HP:ID genes --source monarch` | Source human-gene-to-phenotype rows, default one row. | P21–P23; provider-side human taxon restriction must be positively proved before inclusion. No model-organism or ortholog expansion. |
| Existing `get disease MONDO:ID phenotypes --source monarch` | The already implemented disease annotation page. | P18–P20 reuse its command, schema and qualifiers. This slice adds no second route to synthesize syndrome annotations. |
| Proposed optional `search disease --phenotype HP:ID` or separate explicit similarity mode | Source-native disease candidates only if PM approves their exact semantics. | P17. Naming and scope remain a product decision; the ordinary term-search positional argument must never silently switch to ranking. |

Requests use the established closed source and section choices. Schemas are generated from types and versioned. Source/entity applicability errors return useful usage recovery before I/O. A mixed disabled component is skipped before cache or network access, using the existing Monarch admission rules. No disabled upstream payload is fetched and filtered afterward.

Typed next actions follow architecture 6.10. Each selected term header gets one exact get action, bounded by the checked page size, followed by native paging and at most three other actions. Each distinct linked disease/gene in an association page gets one exact pivot in native order. No action promotes an HGNC symbol to identity, relabels an NCBIGene ID or assumes every synonym is equivalent. An exact existing gene command must accept the returned identifier before its action is advertised. A failed source supplies a fresh action; a disabled source supplies no workaround provider.

## Legacy accounting

The inspected 0.9 source revision is `4050fdc7dbdf4d713d94cf02a39a147b863665be`. Its `src/cli/phenotype/mod.rs`, `dispatch.rs`, `src/entities/disease/search.rs`, `src/sources/hpo.rs`, `src/sources/monarch.rs` and user guide establish the following static inventory. Static inspection proves behavior ownership and request construction, not live output quality or token cost.

| 0.9 command, option or section | 1.0 disposition | Reason and acceptance case |
| --- | --- | --- |
| `search phenotype TERMS` with space/comma HP ID sets | Deferred similarity; retire the old ID-set spelling. | The PM retained the disease ranking retirement. P17 remains unanswered; `retired.yaml` checks fixed recovery to literal term search and explicit associations. |
| One symptom phrase or comma-separated phrases | `search phenotype TEXT --source monarch` sends literal text. | P01–P16; `native.rs` checks exact candidate order and IDs. No candidate becomes an equivalent term by rank. |
| Automatic HPO resolution before similarity | Dropped. | Explicit candidate selection replaces hidden resolution. No symptom intent detector; `retired.yaml` checks ID sets and unsupported HPO search. |
| `-l`, `--limit` | Literal term search retains bounded `--limit`; the old similarity allocation retires. | `examples.yaml` proves the explicit limit; `states.rs` proves a short native page; `admission.yaml` proves invalid bounds. |
| `--offset` | Literal term search retains native `--offset`. | Native P02/P16 middle and terminal rows; no similarity window is advertised. |
| Fixed first 50 similarity rows, local disease deduplication and slicing | Deferred with similarity. | No local disease ranking, deduplication or fabricated total. P17. |
| `resolved_query` phrase, HPO ID and label | Dropped automatic resolution; retain literal `/query/text` and exact result IDs. | `mod.rs::search` and native P01–P16 preserve input and publisher candidates independently. |
| Native semantic score and ordering | Deferred similarity; term search retains its independent native match fields. | `native.rs` checks `score`, `matched_field`, `match_type` and order without reinterpretation. |
| Per-term supported/not_supported/indeterminate/unavailable enrichment | Dropped automatic similarity enrichment. | Explicit `get phenotype HP:ID diseases` and `genes` preserve only the publisher's assertion. P17, P21–P24. |
| Automatic first fully supported disease next step | Dropped. | Exact candidate pivots retain source order; none selects a diagnosis. `mod.rs::search` and `associations.rs`. |
| `list phenotype` | Renamed to `help phenotype`. | `retired.yaml` checks exact recovery; `help.yaml` checks source choices, paging, sections and two case-derived examples per command. |
| `discover` phenotype suggestion | Dropped. | Use explicit `search phenotype TEXT`; automatic HPO resolution and symptom intent detection stay absent. Shared discovery retirement retains its existing owning cases. |
| `get phenotype` card | New: exact independent HPO or Monarch card. | `examples.yaml`, complete native P01–P16 cards; no automatic source fallback. |
| `synonyms`, `xrefs` | New: HPO or Monarch text/link sections. | `sections.rs` and retained `section_native.rs` preserve source classes, field presence and executable local pages. Positive Monarch xrefs were captured in 0208, so this section is kept. |
| `hierarchy` | New: Monarch direct parent and child links. | Retained Seizure pages and `sections.rs`; no locally inferred graph traversal. |
| `diseases`, `genes` | New: direct Monarch association pages with checked original endpoints. | `associations.rs`, six retained reverse pages; human taxon is checked before admission. Existing disease phenotype sections retain closure scope. |
| `--section-limit`, `--section-offset`, `--snapshot` | New bounded section controls. | Text/link pages retain snapshots; association pages reject snapshots and retain native offsets. `admission.yaml`, `sections.rs`, `associations.rs`. |
| CLI JSON/Markdown and MCP invocation | Retained through the shared request and envelope. | `help.yaml`, `mod.rs::help`, executable example ownership and measured named CLI/MCP phases. Shared global flags retain their owning cases. |
| HPO term search | Deferred. | No proved native paging contract; `retired.yaml` preserves explicit Monarch recovery. |
| Translations | Deferred. | No harvested multilingual question requires them. |
| HPO ontology/annotation bundles and Monarch graph bundles | Deferred. | Bounded entity access answers the accepted questions; no whole-bundle download is justified. |

The counted ledger records the shipped literal term search under the reused spelling. The earlier disease ruling still drops symptom-to-disease ranking and automatic HPO resolution. The old similarity option slots remain dropped rather than being relabeled as the new term-search contract.

<!-- parity-accounting -->
| 0.9 leaf command | Command outcome | Ported or renamed options | Dropped options | Deferred options | Undecided options | Ruling |
| --- | --- | --- | --- | --- | --- | --- |
| `search phenotype` | Ported or renamed | — | `--limit`, `--offset` | — | — | Literal Monarch term discovery ships under this spelling. Drop symptom-to-disease ranking and automatic HPO resolution; literal Monarch term discovery and explicit source associations have independent contracts. Similarity stays deferred. |
<!-- /parity-accounting -->

The complete closed public-issue inventory returned 57 issues. No issue title or body contained phenotype, Monarch or HPO. The investigation must preserve the inventory and filter as external evidence; this means zero entity-specific closed issues were found, not that general transport, cache or MCP issues cease to apply.

## Clinical review of question boundaries

The clinical reviewer checked GeneReviews source sections: NF1 Suggestive Findings (revision 2025-04-03); PTEN Summary, Clinical Description and Suggestive Findings (update 2026-07-28); VHL Summary (revision 2025-05-01); and Birt-Hogg-Dube Summary/Clinical Description (update 2024-12-05). These support the features used in P01–P15. They establish retrieval questions, not literal HPO equivalence or diagnostic sufficiency. NF1 and Legius syndrome can share pigmentary findings; a shared term does not select a diagnosis. Exact native HP IDs and source association interpretation require the capture review below.

## Native candidate findings

Captures occurred on 2026-10-26 UTC, which was still 2026-10-25 in New York. Job `j14204` captured the API contract and first term, search and similarity replies. Job `j14210` captured both source searches for P01–P16. Both used one attempt, no response cache, no redirects and no fallback on the M5. Measurements describe the machine and network at capture time; no quiet-machine speed comparison or warm-cache measurement is claimed.

| Case | Native answer or candidates | HPO returned / native total | Monarch returned / native total |
| --- | --- | --- | --- |
| P01 | `HP:0000957` Cafe-au-lait spot is second in both lists. The first candidate is `HP:0005605` Large cafe-au-lait macules with irregular margins. No automatic selection. | 9 / 9 | 3 / 3 |
| P02 | `HP:0001067` Neurofibroma | 10 / 15 | 5 / 16 |
| P03 | `HP:0009732` Plexiform neurofibroma | 10 / 16 | 1 / 1 |
| P04 | `HP:0000997` Axillary freckling | 10 / 15 | 1 / 1 |
| P05 | `HP:0000256` Macrocephaly | 5 / 5 | 5 / 8 |
| P06 | HPO first `HP:0200008` Intestinal polyposis; Monarch first `HP:0004784` Juvenile gastrointestinal polyposis. Both carry `HP:0004390` Hamartomatous polyposis. None becomes equivalent to the input by rank. | 10 / 50 | 3 / 3 |
| P07 | `HP:0012032` Lipoma | 10 / 25 | 5 / 26 |
| P08 | Both first return `HP:0001263` Global developmental delay. The broader phrase is not silently replaced with this term. | 10 / 100 | 5 / 17 |
| P09 | Monarch's literal search is empty. HPO first returns `HP:0000573` Retinal hemorrhage, which is not retinal hemangioblastoma. No candidate is selected as an equivalent. | 10 / 100 | 0 / 0 |
| P10 | HPO first `HP:0005562` Multiple renal cysts; Monarch first `HP:0000107` Renal cyst. Preserve singular versus multiple. | 10 / 100 | 5 / 12 |
| P11 | `HP:0002666` Pheochromocytoma. Monarch also returns `HP:0001676` with an obsolete label; that row needs a deprecation case. | 4 / 4 | 5 / 11 |
| P12 | `HP:0001737` Pancreatic cysts | 10 / 72 | 2 / 2 |
| P13 | `HP:0030436` Fibrofolliculoma | 1 / 1 | 1 / 1 |
| P14 | HPO first `HP:0005948` Multiple pulmonary cysts; Monarch first `HP:0032445` Pulmonary cyst. Preserve multiplicity. | 10 / 100 | 5 / 8 |
| P15 | `HP:0002108` Spontaneous pneumothorax | 10 / 32 | 2 / 2 |
| P16 | `HP:0001250` Seizure | 10 / 100 | 5 / 355 |

These are publisher search-match totals with different matching rules. They are not counts of synonymous terms or clinically equivalent features. HPO's response supplies `totalCount` and up to ten candidates in these observations; this investigation has not established an HPO server paging contract. Monarch's captured contract supports explicit namespace, limit and offset. The unfiltered first Monarch search included MP and UPHENO concepts; provider-side `namespace=HP` is therefore required for this human phenotype surface.

The clinical reviewer accepted the exact reported native identities and the explicit selection boundaries. It specifically rejected equating retinal hemorrhage with retinal hemangioblastoma and warned against interpreting the subtype and multiplicity differences above as synonyms. These counterexamples become acceptance cases, not extra aliases.

The first association request in `j14210` accidentally included an empty `subject_taxon=` selector for a disease subject. It returned HTTP 500, and the recipe stopped before later requests. This is evidence of a malformed investigation request and the stop rule. It is not an accepted source outage measurement or a successful disease association measurement. Successful searches are not repeated to repair that recipe.

### Exact cards and association identity

The authorized continuation `j14227` completed all 34 planned requests without another server error. It captured seventeen exact HPO candidate cards, six requested disease/gene association pages, two reverse term-association pages, three empty/absent witnesses and six release, terms and issue-inventory documents. No successful search was repeated. Across the three native jobs there were 73 delivered requests, including the invalid-selector 500. No product binary or 0.9 binary ran.

All seventeen cards returned the requested exact HP identity. Seizure's exact card from the metadata pass is reused. The clinical reviewer read all eighteen retained cards and accepted the bank with these rules: preserve the complete Global developmental delay definition and its age restriction and comment; preserve Macrocephaly's source measurement definition and distinction from relative macrocephaly; preserve the Pancreatic cysts comment distinguishing pseudocysts; and preserve the pulmonary cyst comments and multiplicity distinctions. HPO's flat synonyms include Neurofibromatosis for Neurofibroma and Epilepsy for Seizure. They are source-published synonyms, not permission to make those clinical equivalences. No term definition or comment is shortened to meet a budget.

| Query scope | Native total | First two exact returned endpoints | Provenance and qualifiers |
| --- | --- | --- | --- |
| P18 `MONDO:0008667`, `direct=true` | 60 | `HP:0031207`, `HP:0002894` | Actual subject is von Hippel-Lindau disease; original `OMIM:193300`; HPO disease annotations; ECO:0000304 / ECO:0000501. |
| P19 `MONDO:0018875`, `direct=true` | 48 | `HP:0002894`, `HP:0003002` | Actual subject is Li-Fraumeni syndrome; original `OMIM:151623`; ECO:0000501. |
| P20 `MONDO:0019003`, `direct=true` | 48 | `HP:0002315`, `HP:0000787` | Actual subject is MEN2; original `Orphanet:653`; frequency IDs HP:0040282 / HP:0040283 and ECO:0000304 remain on their own rows. |
| P21 human `HGNC:12687` | 58 | `HP:0006880`, `HP:0002668` | VHL; original `NCBIGene:7428`; source subject taxon NCBITaxon:9606. |
| P22 human `HGNC:9588` | 85 | `HP:0000337`, `HP:0000160` | PTEN; original `NCBIGene:5728`; source subject taxon NCBITaxon:9606. |
| P23 human `HGNC:7765` | 129 | `HP:0000006`, `HP:0011343` | NF1; original `NCBIGene:4763`; source subject taxon NCBITaxon:9606; PMID:8317503 belongs only to the second captured row. |
| Reverse `HP:0001250`, diseases, `direct=true` | 2,561 | `MONDO:0014379`, `MONDO:0012596` | Exact object HP:0001250. HPO disease annotations preserve original Orphanet identities, evidence and frequency IDs. |
| Reverse `HP:0001250`, human genes, `direct=true` | 1,409 | `HGNC:9910`, `HGNC:29529` | RBMX / TBL1XR1; originals NCBIGene:27316 / NCBIGene:79718. Exact object and human subject taxon are supplied. |

These totals count association records in the stated native request scope. They are not unique gene/disease/patient counts. The `direct=true` request establishes the publisher's graph behavior and exact endpoints; it establishes no direct molecular causation or clinical directness. It also explains why MEN2's direct count differs from the existing disease section's closure-expanded count. That section retains its accepted behavior; this proposal must name its own scope explicitly rather than change the older section.

The gene rows carry `biolink:has_phenotype`, `infores:hpo-annotations` and `hpoa_gene_to_phenotype_edges`. Null evidence or negation remains unavailable, not no evidence or false negation. Native explicit false remains explicit false. The clinical reviewer accepted these source-bound interpretations. No qualifier, publication or claim transfers between neighboring rows or between a subtype and parent.

The continuation also captured two successful zero-result searches for the literal unknown string and an exact HP:9999999 HTTP 404. Failure acceptance must additionally use recorded 429, 500, timeout and broken-body cases. The invalid-selector 500 is not substituted for a representative provider outage.

### Source fields, terms and bundles

| Source/tier | Native public fields proved | Kept behavior and omissions |
| --- | --- | --- |
| HPO JAX term API | `id`, `name`, `definition`, `comment`, `descendantCount`, flat `synonyms`, `xrefs`, `publicationReferences`, `translations` with language/name/definition/status. Empty strings, empty arrays and null translation collections occur. | Keep exact identity, complete definition/comment, references and explicit synonym/xref sections. Do not infer ontology release or synonym relationship type. Defer translations because no harvested multilingual question needs them. Descendant count is source metadata, not a fabricated hierarchy. |
| HPO JAX search | `terms` with term metadata and `totalCount`. | Investigated and retained as comparison evidence. Prefer Monarch's proved native paging and HP namespace for product discovery; do not invent HPO paging from undocumented parameters. |
| Monarch search | Limit, offset, total, ordered items; native ID/name/category, description, source synonym classifications, score/matched field/match type, facets and optional resolved scope. | Fixed human phenotype namespace; compact headers retain native match facts. Facets and other ontology namespaces are not needed by this bank. Never locally rerank or reinterpret matching. |
| Monarch entity | Exact ID, description, classified synonyms, native deprecation, source/xrefs, association counts and hierarchy. Many source fields are explicitly null. | Independent source card and named sections. Association counts distinguish direct/expanded scopes. Do not merge with HPO's different descendant count: Seizure has HPO346 and Monarch347 in the captured replies. |
| Monarch association | Exact and original endpoints, source categories/taxa, predicates, provenance, evidence/publications, null or populated qualifiers, native closures, links, counts and page metadata. | Reuse the existing neutral association model and preserve clinically relevant qualifier presence. Add source taxon to that shared model if it is missing; do not create a second association representation. A human restriction must be sent and verified at the provider. |
| Monarch similarity | Native ordered disease rows and score; subject/object term sets, best matches, subsumers, component similarity details and metrics. | Only P17 needs this analytic result. Defer implementation until PM rules. No local scoring or automatic direct-support enrichment. |

HPO's [publisher license](https://human-phenotype-ontology.github.io/license.html) was captured successfully and has the same body hash as the prior disease investigation's reviewed grant. The declaration remains `attribution`, with acknowledgement/citation, displayed dates or versions, preserved content and logical relationships, and product acknowledgement. API retrieval time is labeled as retrieval time; absent ontology version is reported as missing metadata. It is never filled from the date of an unrelated GitHub release. HPO source and component disables and the existing `attribution` group apply before network or cache access. No key is required.

Monarch's [licensing page](https://monarch-app.monarchinitiative.org/Licensing/) recommends CC0 or CC BY for project data while leaving per-repository licenses authoritative. Keep the existing `attribution` source declaration and HPO/Orphanet component rules. HPO-published annotations that cite OMIM identities do not become a separately licensed OMIM payload. Separately supplied upstream content requires independent terms review. An explicit disabled mixed component stops the request before I/O unless a captured native exclusion can prove safe narrowing. The restricted group alone does not disable these previously reviewed HPO annotations. No key or restored key name is relevant to either kept source.

HPO release metadata reports `v2026-09-01`, published 2026-09-03, and the preceding `v2026-06-23`. This proves those releases and dates, not a fixed monthly cadence or the version currently served by JAX or Monarch. The newer release lists hp.obo at 10,863,613 bytes, hp.json at 21,915,519, genes_to_phenotype.txt at 20,821,704, phenotype.hpoa at 35,816,037 and phenotype_to_genes.txt at 67,246,520. These are publisher asset sizes; no asset download or local index build occurred. The repository README supplies publisher ontology download pointers. The Monarch release endpoint returned only a mutable `latest` URL, not a pinned version or graph size. Its complete bundle cost and refresh cadence remain unmeasured.

Keep live bounded entity access. Defer HPO ontology/annotation bundles and the whole Monarch graph because the harvested questions already have bounded source routes. This avoids downloading a whole graph to answer one exact term or association. A later offline/bulk question can justify explicit sync through the existing bundle framework. No external bioinformatics tool is downloaded, installed or run.

## Progressive disclosure and measured cost

Term search defaults to five compact headers. Exact HPO cards keep the complete definition, comment and publication references together because source comments prevent incorrect equivalence in this bank. Synonyms and cross-references are explicit five-row sections. Monarch hierarchy is explicit and defaults to five parent/child links, retaining the source relation and native order. Disease/gene association pages default to one complete row because qualifier and closure payloads dominate cost. Every section uses `--section-limit` and `--section-offset`; explicit limits retain their checked values. No field or complete comment is clipped to meet a budget.

The formatter in `j14238` measured proposed projections before the command envelope and typed actions existed. Those projections are not acceptance ceilings. The table now uses retained implemented CLI/MCP envelope measurements from the term, section and association proofs below. Comparable bytes remove exactly `use`, `terms` and `terms_url` from validated source outcomes. Bytes divided by four and rounded up are a token proxy, not tokenizer output. Fixed case ceilings add fifteen percent to the matching measured baseline.

| Implemented layer and retained scope | Full bytes, largest retained envelope | Comparable bytes | Proxy tokens | Comparable ceiling | 0.9 comparison |
| --- | ---: | ---: | ---: | ---: | --- |
| Monarch term search, P01–P16, five-header default including actions | 2,203 | 2,072 | 518 | 2,383 | Unmeasured: contained comparison belongs to 0098. |
| Exact HPO cards, eighteen retained identities with whole text and section actions | 2,032 | 1,868 | 467 | 2,149 | Unavailable: 0.9 has no phenotype get command. |
| Exact Monarch Seizure card with section actions | 2,081 | 1,949 | 488 | 2,242 | Unavailable: 0.9 has no phenotype get command. |
| HPO synonyms, first five rows | 1,327 | 1,163 | 291 | 1,338 | Unavailable: no 0.9 section. |
| HPO xrefs, first five rows | 586 | 422 | 106 | 486 | Unavailable: no 0.9 section. |
| Monarch Seizure hierarchy, first five links | 1,905 | 1,773 | 444 | 2,039 | Unavailable: no 0.9 section. |
| HP:0001250 direct disease/gene associations, one-row pages | 3,311 | 3,180 | 795 | 3,657 | Unavailable: no 0.9 reverse-association section. |
| HP:0001250 direct disease/gene associations, two-row pages | 5,539 | 5,408 | 1,352 | 6,220 | Unavailable: no 0.9 reverse-association section. |
| Similarity | Deferred | — | — | — | Unmeasured: old ranking is retired; retained native replies are investigation evidence only. |

These retained native measurements describe their original product revisions and exact scopes. They are not new command executions at the accounting revision. Association outputs predate the null/empty evidence-presence fix. The current synthetic phase measurements below include that fix and the complete present envelopes. Neither the complete native hierarchy payload nor the earlier source-row projection supplies the current product allocation. Original native archives and their measurements remain unchanged.

The native HPO search requests took 0.135–0.391 seconds, median 0.142, and returned 602–6,553 bytes. The HP-filtered Monarch searches took 0.135–0.622 seconds, median 0.160, and returned 176–8,178 bytes. The unfiltered initial Monarch search was 6,300 bytes and included other ontology namespaces. The initial similarity request took 3.979 seconds and returned 36,803 raw bytes for five rows. Its details explain why similarity must be an explicit expensive disclosure layer if kept. These are serial uncached HTTP observations on a shared M5, not warm product latency or quiet-machine speed baselines. No warm-cache timing is established. No per-card follow-up requests are hidden inside these request counts.

Every 0.9 same-question byte/token measurement is unmeasured here because verified containment was not established and the binary was not run. The earlier disease investigation's bounded 0.9 outputs do not contain qualified phenotype/gene section comparisons. Static inspection shows 0.9 incurs term resolution, similarity and a direct-support enrichment query; it does not establish this bank's live facts or comparative completeness. A future contained baseline can reuse retained exact requests; it cannot treat a rewritten request as identical to the literal term search.

## Acceptance cases by command and option

| Surface | Positive story and contrasting case |
| --- | --- |
| Literal `search phenotype TEXT` | P02/P05 exact label, P01/P06/P08/P10/P14 ambiguous subtype or multiplicity, P09 unmatched term, P24 successful empty. Assert native ordering, returned HP namespace and match metadata. |
| Search `--source monarch` and default source | Same checked request and source; disabled source before cache/network; unsupported source is usage error rather than fallback. |
| Search `--limit` / `--offset` | Native P02/P16 first and offset-five pages are captured. P02 offset fifteen is a final partial page with one of sixteen total rows. Preserve exact next arguments; a terminal empty response remains a synthetic acceptance case. Zero, overflow and out-of-range inputs fail before I/O. |
| Exact get and `--source hpo` | All eighteen exact identities; P24 unknown exact term; mismatched requested/returned identity and malformed/null identifier are recorded failures. HP grammar and wrong namespace fail before I/O. |
| Exact get `--source monarch` | Seizure exact identity; source-specific synonym/deprecation behavior. The exact HP:0001676 card supplies `deprecated=true` and no replacement. Preserve the native deprecation fact; offer no replacement action. |
| `synonyms` / `xrefs` | P02/P05/P16 source arrays; preserve unclassified versus classified synonyms. The exact Monarch HP:0000256 card supplies `xref=["Fyler:4335"]`, proving a positive Monarch xref section; Seizure null remains unavailable. Empty lists remain distinct from unavailable fields. |
| `hierarchy` | Seizure's captured one parent/twelve children, native order, source relation and exact IDs. First/next/terminal local pages remain bound to the retained entity revision; no invented graph traversal. |
| `diseases` / `genes` | Reverse Seizure pages and P18–P23 exact endpoints; gene human taxon and original identity; two rows with different provenance must not collapse. Native limit-one offset-one disease and human-gene pages are captured. The mouse taxon request returns an empty page, so wrong-taxon response rejection uses an explicitly synthetic reply. |
| Section `--section-limit` / `--section-offset` | Omitted one-row association default versus explicit two-row limit, five-row text/link defaults, exact page action and terminal empty. Numeric bounds before I/O; no hidden fetch of all pages. |
| Existing disease section | Reuse P18–P20 source facts without changing its closure-expanded contract. Qualifiers never transfer between subtype and parent or rows. |
| `--fresh` / source policy | Shared fresh/cache contracts, disabled component before reads, stale source metadata, source failure versus successful empty. Preserve allowed sibling source results without automatic fallback. |
| CLI / main MCP tool / read-only wrappers | Shared checked request, schema and source facts. Typed next arguments round-trip unchanged; wrappers retain canonical publisher links and omit uncitable rows using the existing wrapper contract. |
| Error and cost cases | Recorded 429, 500, timeout, malformed/truncated body, wrong identity/taxon, null qualifiers, explicit false negation, missing version, measured output ceilings and comments retained whole. An investigation recipe defect is not relabeled a provider outage. |

The HPO term card has no native deprecation/version field in this capture. Source metadata cannot establish the absence of an obsolete status or a release binding. The admission captures below prove middle paging, a final partial page, exact native deprecation and a positive Monarch xref. They supply no obsolete replacement or native wrong-taxon response to reject. A source-specific section is offered only after its actual public fields are captured.

## Ian's questions

The PM reviews these product choices and escalates only a ruling conflict or material expansion that needs Ian. The investigation asks Ian no clinical or configuration question.

1. **Keep term identity and explicit association retrieval?** Recommend the bounded term search/exact HPO card and reuse of the existing Monarch association model. They answer the bank without first-hit equivalence or patient ranking. No paid service, key, large bundle or external tool is needed.
2. **Keep source-native similarity as a separate explicit research operation?** P17 supplies a sourced question, but the earlier disease slice retired symptom-to-diagnosis ranking. Recommend deferring it from the first implementation until the PM records its naming and scope. Native score and best-match details cost materially more than term identity; no diagnostic promise or default enrichment is justified.
3. **Keep the proposed disclosure allocations?** Recommend five term headers and one complete association row by default, with whole term definitions/comments and exact source qualifiers preserved. Actual product envelopes still need measured ceilings and source-specific positive captures before their schemas are accepted.

No release, external publication, paid commitment or licensed upstream data access is requested by this design.

## Evidence and known limits

The verified evidence archive is on Yellow at `data/biomcp-redux/0192-phenotype-investigation/j14204-j14210-j14227-j14238-source-evidence.tar.gz`, SHA-256 `d837797bdfc94a19ba99074ecd37546f6b9864b1850007c4de218cc9a129b4f8`, 265,022 bytes. Independent inspection verified the exact 179-member inventory, 178 content hashes and all 73 native response receipts before scratch cleanup. Yellow independently verified the archive and every member hash after transfer. It contains raw bodies, headers, request inventories, job logs, continuation and formatter scripts, proposed views and measurements. The initial inline recipes remain in the original pm job request records; their emitted URLs, timings and responses are archived. The nested unchanged first-pass archive has SHA-256 `3747a1cbf5d08946f67d30ec8cf547db2793a8102bac34e0a812e9a51f1c741f`.

### Phenotype admission captures

Job `j14350` made eight keyless requests from landed commit `6e20aa069bbb981f9c9c6f0da3fd4ba31e556ceb`. Each fixed plan ran through `make source-capture` with the retained contract's exact selectors, a fifteen-second deadline, a four-MiB body bound, one-second pacing, no retries and no redirects. The ninth possible request was skipped after the first cross-reference card supplied a non-null field. No successful search was repeated. All replies were HTTP 200; no credential, cookie, bundle or product command was used.

| Witness | Native result | Capability unlocked or deferred |
| --- | --- | --- |
| P02 neurofibroma, HP namespace and phenotype category, limit 5 / offset 5 | Proven: five HP rows, native total 16; ordered IDs HP:0034765, HP:0100698, HP:0034762, HP:0009732, HP:0009735. | Native middle-page search is admitted with unchanged order and match metadata. |
| P16 seizure, same selectors, limit 5 / offset 5 | Proven: five HP rows, native total 355; ordered IDs HP:0020219, HP:0031951, HP:0032792, HP:0032794, HP:0032807. | The second term's middle-page search is admitted. |
| P02 neurofibroma, same selectors, limit 5 / offset 15 | Proven: one final row, HP:0009595, native total 16. | A final partial page is admitted. No next step follows this page. Terminal empty remains an explicitly synthetic case; no extra page was requested. |
| Exact Monarch HP:0001676 | Proven: requested ID, phenotype category and `deprecated=true`; no replacement identifier or relationship is supplied. | Native deprecation is admitted for the Monarch card. Replacement actions remain deferred. |
| Exact Monarch HP:0000256 | Proven: requested ID and `xref=["Fyler:4335"]`. HP:0001067 was not requested because the first card was positive. | A native Monarch xref section is admitted, preserving the publisher's reference without inventing an external URL. |
| Reverse HP:0001250 diseases, `direct=true`, limit 1 / offset 1 | Proven: total 2,561; actual subject MONDO:0012596 and object HP:0001250. | Disease association page steps are admitted with original endpoints and row provenance preserved. |
| Reverse HP:0001250 genes, `direct=true`, human subject taxon, limit 1 / offset 1 | Proven: total 1,409; actual subject HGNC:29529, object HP:0001250 and subject taxon NCBITaxon:9606. | Human gene association page steps are admitted with the explicit provider selector and checked row taxon. |
| Same gene request with subject taxon NCBITaxon:10090 | Empty: HTTP 200, total zero and no rows. | The requested mouse taxon scope is empty. This does not prove rejection of a returned wrong-taxon row. That rejection case remains explicitly synthetic. |

Each row above is backed by the archive `data/biomcp-redux/0208-phenotype-admission-captures/2291-phenotype-admission-captures-verified.tar.gz` on Yellow, SHA-256 `4340c3f959015ae77b0328156a2f635d58107ccda7a61e45248f623ae8ff3428`. It retains the complete exact-URL fixed plan, conditional controller, individual helper plans, raw bodies and headers, receipts with status/bytes/hash/timing, the aggregate ledger and nested helper archives. Independent inspection verified all 71 outer members and 70 hashes, plus all eight nested helper archives with six members and five hashes each. Yellow independently verified the outer inventory and hashes after transfer, before scratch cleanup.

All eight helper archives passed their own verification. The first outer packaging attempt in `j14350` failed verification because macOS tar added AppleDouble members. Offline job `j14354` repacked the retained bytes with `COPYFILE_DISABLE=1` and the existing helper's archive verifier. It made no provider requests and changed no source result. These captures establish source facts only; they supply no inferred clinical answer.

The archived closed-issue search is complete at 57 returned of 57 total, `incomplete_results=false`; the entity/source filter has no matches. The issue response body has SHA-256 `6a425c084987cbf99b48924b318c63b669ffb580987a951a06e9dbd4c651c8f6`. It preserves the full public inventory rather than a selected issue summary.

The implemented scope follows the PM phenotype ruling. The explicit deferrals and retirements above remain part of that scope. Remaining source limitations are explicit: no warm product timing or contained 0.9 execution; no pinned API ontology version; no native terminal empty search page or wrong-taxon returned-row rejection witness; no obsolete replacement field; and no whole-graph bundle download/size measurement. The existing captured fields can guide types, but final type acceptance must check actual absence, multiplicity and scope against the preserved bodies. These limits do not justify inventing a source fact or another automatic live pass.

## Implemented term command measurements

The term commands retain literal Monarch text, native candidate order and complete independent source cards. Each search header offers the exact default HPO get action. Cards list their source's native sections and do not advertise handlers that are not implemented. The positive Monarch synonym witness retains exact and related classes independently from HPO's unclassified list; this card implementation only uses field presence to list the section.

Jobs `j14607` and `j14614` replayed forty-one unchanged native replies through CLI and MCP. The first job proved thirty-five rows. Its next row exposed a request-scope mismatch in the old unknown-text witness: that request omitted the phenotype category selector. The ticket-authorized single keyless request in `j14614` supplied the exact missing selector set with no retry, a fifteen-second deadline and a four-MiB bound. It returned a successful empty page. The six unreached cases then passed. The local replay recipe failure was not a publisher outage. No successful source request or completed native case was repeated.

Complete definitions, comments, references, counts and available source sections remain in these outputs. Their actual envelopes include checked queries, attribution, component terms and typed candidate actions that the proposed formatter did not fully model. The measured ceilings below replace the proposed term-command allocations under the standing measured-size-plus-fifteen-percent rule. Comparable bytes remove exactly `use`, `terms` and `terms_url` from each source outcome. They remove no other fields. The native replay check binds each retained case to its fixed `native_ceiling` entry.

| Retained case | Full bytes | Comparable bytes | Comparable ceiling |
| --- | --- | --- | --- |
| `00-hpo-term.raw` | 1178 | 1015 | 1168 |
| `01-monarch-term.raw` | 1749 | 1618 | 1861 |
| `02-P01-monarch-search.raw` | 1600 | 1469 | 1690 |
| `03-P02-monarch-search.raw` | 2118 | 1987 | 2286 |
| `04-P03-monarch-search.raw` | 1075 | 944 | 1086 |
| `05-P04-monarch-search.raw` | 1067 | 936 | 1077 |
| `06-P05-monarch-search.raw` | 2121 | 1990 | 2289 |
| `07-P06-monarch-search.raw` | 1597 | 1466 | 1686 |
| `08-P07-monarch-search.raw` | 2088 | 1957 | 2251 |
| `09-P08-monarch-search.raw` | 2203 | 2072 | 2383 |
| `10-P09-monarch-search.raw` | 776 | 645 | 742 |
| `11-P10-monarch-search.raw` | 2110 | 1979 | 2276 |
| `12-P11-monarch-search.raw` | 2190 | 2059 | 2368 |
| `13-P12-monarch-search.raw` | 1311 | 1180 | 1357 |
| `14-P13-monarch-search.raw` | 1064 | 933 | 1073 |
| `15-P14-monarch-search.raw` | 2145 | 2014 | 2317 |
| `16-P15-monarch-search.raw` | 1336 | 1205 | 1386 |
| `17-P16-monarch-search.raw` | 2086 | 1955 | 2249 |
| `18-card-HP_0000957.raw` | 1243 | 1080 | 1242 |
| `19-card-HP_0001067.raw` | 894 | 731 | 841 |
| `20-card-HP_0009732.raw` | 839 | 676 | 778 |
| `21-card-HP_0000997.raw` | 896 | 733 | 843 |
| `22-card-HP_0000256.raw` | 1725 | 1562 | 1797 |
| `23-card-HP_0200008.raw` | 746 | 583 | 671 |
| `24-card-HP_0004390.raw` | 1068 | 905 | 1041 |
| `25-card-HP_0012032.raw` | 796 | 633 | 728 |
| `26-card-HP_0001263.raw` | 1453 | 1290 | 1484 |
| `27-card-HP_0005562.raw` | 737 | 574 | 661 |
| `28-card-HP_0000107.raw` | 720 | 557 | 641 |
| `29-card-HP_0002666.raw` | 1013 | 850 | 978 |
| `30-card-HP_0001737.raw` | 877 | 714 | 822 |
| `31-card-HP_0030436.raw` | 1082 | 919 | 1057 |
| `32-card-HP_0005948.raw` | 1059 | 896 | 1031 |
| `33-card-HP_0032445.raw` | 1443 | 1280 | 1472 |
| `34-card-HP_0002108.raw` | 769 | 606 | 697 |
| `35-P24-monarch-empty.raw` | 767 | 636 | 732 |
| `36-P24-hpo-absent.raw` | 339 | 176 | 203 |
| `37-P02-middle.raw` | 2085 | 1954 | 2248 |
| `38-P16-middle.raw` | 2050 | 1919 | 2207 |
| `39-P02-terminal.raw` | 1035 | 904 | 1040 |
| `40-obsolete.raw` | 1043 | 912 | 1049 |

The generated search help measures 844 comparable bytes and uses a 971-byte ceiling. Exact get help measures 1,455 bytes and uses a 1,674-byte ceiling. Their Markdown views measure 776 and 1,266 bytes and retain every checked argument. Whole-tree help, including the accepted study and pathway command registrations, measures 66,338 compact bytes; the existing whole-help case uses its standing measured-size-plus-fifteen-percent ceiling of 76,290 bytes. The CLI and MCP help cases and whole-tree executable examples passed in `j14605`.

The complete native evidence archive is on Yellow at `data/biomcp-redux/0217-phenotype-commands/1792992427-remaining-package.tar.gz`, SHA-256 `74b34e6090b6f3a3f636a933a5b4977ecc44b112052736f0e42b7e98b942c31e`. It retains the original and missing-request plans, unchanged bodies, hashes, receipts, complete CLI/MCP product outputs and logs. `paths.json` reverses the flat archive names to their original collection and path. Independent offline job `j14619` verified the archive hash, all 143 exact members, all 142 content hashes and all forty-one CLI/MCP result files after transfer. Raw source text remains outside Git.

## Retained text and link sections

Text and link pages use the existing immutable source snapshot store. The request's `snapshot` selector binds the exact source and HP identifier to the complete admitted reply, its retrieval date and source outcomes. Cards publish only implemented sections backed by a present source field. A page preserves the selector, source, section and explicit limit. A missing or foreign selector offers a restart rather than another provider request. The existing store expiry governs retained replies.

HPO synonym rows have no relationship class. Monarch rows preserve their native class and array order. The captured Monarch cross reference supplies no predicate, so none is inferred. Direct hierarchy links retain only their original identifier, name and parent or child collection. Source-null sections remain unavailable; a source-empty array remains a present collection.

## Section and card action measurements

The retained CLI and MCP run validates forty-nine section witnesses and twenty revised cards. The canonical measured baselines live in `tests/phenotype-section-budgets.yaml`; both full CLI envelopes and comparable replies use ceilings computed from those baselines plus fifteen percent. The table shows comparable MCP bytes. CLI output adds its newline. These values include whole row text, native references and executable continuation arguments. No provider request ran.

| Case | Source and term | Section or card | Offset / limit | Measured bytes | Ceiling bytes | Ceiling tokens |
| --- | --- | --- | --- | ---: | ---: | ---: |
| `card-HP_0000107-hpo` | hpo HP:0000107 | card | — | 863 | 993 | 249 |
| `card-HP_0000256-hpo` | hpo HP:0000256 | card | — | 1868 | 2149 | 538 |
| `card-HP_0000957-hpo` | hpo HP:0000957 | card | — | 1386 | 1594 | 399 |
| `card-HP_0000997-hpo` | hpo HP:0000997 | card | — | 1039 | 1195 | 299 |
| `card-HP_0001067-hpo` | hpo HP:0001067 | card | — | 1037 | 1193 | 299 |
| `card-HP_0001250-hpo` | hpo HP:0001250 | card | — | 1321 | 1520 | 380 |
| `card-HP_0001250-monarch` | monarch HP:0001250 | card | — | 1949 | 2242 | 561 |
| `card-HP_0001263-hpo` | hpo HP:0001263 | card | — | 1596 | 1836 | 459 |
| `card-HP_0001676-monarch` | monarch HP:0001676 | card | — | 1072 | 1233 | 309 |
| `card-HP_0001737-hpo` | hpo HP:0001737 | card | — | 1020 | 1173 | 294 |
| `card-HP_0002108-hpo` | hpo HP:0002108 | card | — | 912 | 1049 | 263 |
| `card-HP_0002666-hpo` | hpo HP:0002666 | card | — | 1156 | 1330 | 333 |
| `card-HP_0004390-hpo` | hpo HP:0004390 | card | — | 1211 | 1393 | 349 |
| `card-HP_0005562-hpo` | hpo HP:0005562 | card | — | 880 | 1012 | 253 |
| `card-HP_0005948-hpo` | hpo HP:0005948 | card | — | 1202 | 1383 | 346 |
| `card-HP_0009732-hpo` | hpo HP:0009732 | card | — | 982 | 1130 | 283 |
| `card-HP_0012032-hpo` | hpo HP:0012032 | card | — | 939 | 1080 | 270 |
| `card-HP_0030436-hpo` | hpo HP:0030436 | card | — | 1225 | 1409 | 353 |
| `card-HP_0032445-hpo` | hpo HP:0032445 | card | — | 1586 | 1824 | 456 |
| `card-HP_0200008-hpo` | hpo HP:0200008 | card | — | 889 | 1023 | 256 |
| `section-00` | hpo HP:0001250 | synonyms | 0 / 5 | 688 | 792 | 198 |
| `section-01` | hpo HP:0001250 | xrefs | 0 / 5 | 297 | 342 | 86 |
| `section-02` | monarch HP:0001250 | synonyms | 0 / 5 | 1573 | 1809 | 453 |
| `section-03` | monarch HP:0001250 | xrefs | 0 / 5 | 203 | 234 | 59 |
| `section-04` | monarch HP:0001250 | hierarchy | 0 / 5 | 1773 | 2039 | 510 |
| `section-05` | monarch HP:0001250 | hierarchy | 5 / 5 | 1772 | 2038 | 510 |
| `section-06` | monarch HP:0001250 | hierarchy | 10 / 5 | 1221 | 1405 | 352 |
| `section-07` | monarch HP:0001250 | hierarchy | 0 / 2 | 1253 | 1441 | 361 |
| `section-08` | hpo HP:0000957 | synonyms | 0 / 5 | 1163 | 1338 | 335 |
| `section-09` | hpo HP:0000957 | xrefs | 0 / 5 | 297 | 342 | 86 |
| `section-10` | hpo HP:0001067 | synonyms | 0 / 5 | 708 | 815 | 204 |
| `section-11` | hpo HP:0001067 | xrefs | 0 / 5 | 422 | 486 | 122 |
| `section-12` | hpo HP:0009732 | synonyms | 0 / 5 | 303 | 349 | 88 |
| `section-13` | hpo HP:0009732 | xrefs | 0 / 5 | 297 | 342 | 86 |
| `section-14` | hpo HP:0000997 | synonyms | 0 / 5 | 303 | 349 | 88 |
| `section-15` | hpo HP:0000997 | xrefs | 0 / 5 | 297 | 342 | 86 |
| `section-16` | hpo HP:0000256 | synonyms | 0 / 5 | 1131 | 1301 | 326 |
| `section-17` | hpo HP:0000256 | xrefs | 0 / 5 | 422 | 486 | 122 |
| `section-18` | hpo HP:0200008 | synonyms | 0 / 5 | 740 | 851 | 213 |
| `section-19` | hpo HP:0200008 | xrefs | 0 / 5 | 297 | 342 | 86 |
| `section-20` | hpo HP:0004390 | synonyms | 0 / 5 | 738 | 849 | 213 |
| `section-21` | hpo HP:0004390 | xrefs | 0 / 5 | 297 | 342 | 86 |
| `section-22` | hpo HP:0012032 | synonyms | 0 / 5 | 571 | 657 | 165 |
| `section-23` | hpo HP:0012032 | xrefs | 0 / 5 | 550 | 633 | 159 |
| `section-24` | hpo HP:0001263 | synonyms | 0 / 5 | 1199 | 1379 | 345 |
| `section-25` | hpo HP:0001263 | xrefs | 0 / 5 | 297 | 342 | 86 |
| `section-26` | hpo HP:0005562 | synonyms | 0 / 5 | 442 | 509 | 128 |
| `section-27` | hpo HP:0005562 | xrefs | 0 / 5 | 297 | 342 | 86 |
| `section-28` | hpo HP:0000107 | synonyms | 0 / 5 | 829 | 954 | 239 |
| `section-29` | hpo HP:0000107 | xrefs | 0 / 5 | 297 | 342 | 86 |
| `section-30` | hpo HP:0002666 | synonyms | 0 / 5 | 573 | 659 | 165 |
| `section-31` | hpo HP:0002666 | xrefs | 0 / 5 | 422 | 486 | 122 |
| `section-32` | hpo HP:0001737 | synonyms | 0 / 5 | 711 | 818 | 205 |
| `section-33` | hpo HP:0001737 | xrefs | 0 / 5 | 297 | 342 | 86 |
| `section-34` | hpo HP:0030436 | synonyms | 0 / 5 | 438 | 504 | 126 |
| `section-35` | hpo HP:0030436 | xrefs | 0 / 5 | 297 | 342 | 86 |
| `section-36` | hpo HP:0005948 | synonyms | 0 / 5 | 576 | 663 | 166 |
| `section-37` | hpo HP:0005948 | xrefs | 0 / 5 | 297 | 342 | 86 |
| `section-38` | hpo HP:0032445 | synonyms | 0 / 5 | 430 | 495 | 124 |
| `section-39` | hpo HP:0032445 | xrefs | 0 / 5 | 297 | 342 | 86 |
| `section-40` | hpo HP:0002108 | synonyms | 0 / 5 | 447 | 515 | 129 |
| `section-41` | hpo HP:0002108 | xrefs | 0 / 5 | 297 | 342 | 86 |
| `section-42` | monarch HP:0001676 | synonyms | 0 / 5 | 209 | 241 | 61 |
| `section-43` | monarch HP:0001676 | xrefs | 0 / 5 | 203 | 234 | 59 |
| `section-44` | monarch HP:0001676 | hierarchy | 0 / 5 | 670 | 771 | 193 |
| `section-45` | monarch HP:0001676 | hierarchy | 5 / 5 | 670 | 771 | 193 |
| `section-46` | monarch HP:0001676 | hierarchy | 10 / 5 | 672 | 773 | 194 |
| `section-47` | monarch HP:0001676 | hierarchy | 0 / 2 | 670 | 771 | 193 |
| `section-48` | monarch HP:0000256 | xrefs | 0 / 5 | 790 | 909 | 228 |

The native section proof archive is on Yellow at `data/biomcp-redux/0220-phenotype-sections/native-section-proof.tar.gz`, SHA-256 `57c1ea2b054dcc0dc1f79882f8c4bf25fe31981a69b2dec6a94f8558b6369c1a`. It retains the verified source-body plan, unchanged bodies, CLI/MCP recordings, forty-nine section results, twenty revised cards, measured baselines and scoped ceiling proof. The producer revisions and source archive hashes are recorded in `proof.json`. Independent verification checked the exact 180-member inventory and all 179 content hashes on both machines. No provider request ran.

### Direct reverse association proof

`get phenotype HP:0001250 diseases --source monarch` and `genes` preserve Monarch's direct graph scope, native association totals and source row order. They reuse the disease association record and mapper. Exact HP object and human gene taxon checks run before cache admission. The existing closure-expanded disease route stays unchanged. Native gene identifiers supply pivots only where the existing gene command accepts them. Source-local rows and totals supply no causal clinical assertion.

Retained j14227 limit-two first pages and 0208 limit-one next pages supply four exact requests. Job j14844 captured only the two missing limit-one first pages with a fifteen-second deadline, four-MiB bound, one-second admission spacing and zero retries. Their verified Yellow archive is `data/biomcp-redux/0240-phenotype-associations/default-pages.tar.gz`, SHA-256 `75f3e393e53d7b96f27d47612dc82750ff5cc03ef17b5452c5084a262f7ae061`. The archive contains nine exact members and eight content hashes. No other provider request ran.

The unchanged P21 to P23 gene-keyed bodies exercise the existing private row mapper through its declared retained-source contract. They preserve human taxa, original NCBIGene identities and NF1's second-row publication. They add no gene-keyed route. The reverse source commands run through CLI and MCP. Each pair returns identical JSON. The limits below are measured comparable bytes plus fifteen percent, rounded up; token limits divide those ceilings by four and round up.

| HP:0001250 section | Limit / offset | CLI and MCP full bytes | Comparable bytes | Measured tokens | Byte ceiling | Token ceiling |
| --- | --- | --- | --- | --- | --- | --- |
| diseases | 1 / 0 | 3311 | 3180 | 795 | 3657 | 915 |
| diseases | 1 / 1 | 3086 | 2955 | 739 | 3399 | 850 |
| diseases | 2 / 0 | 5539 | 5408 | 1352 | 6220 | 1555 |
| genes | 1 / 0 | 2704 | 2573 | 644 | 2959 | 740 |
| genes | 1 / 1 | 2860 | 2729 | 683 | 3139 | 785 |
| genes | 2 / 0 | 4715 | 4584 | 1146 | 5272 | 1318 |

Job j14910 verified all six native command pairs and their original source identities, evidence, publications, qualifier IDs and labels. The verified product archive is `data/biomcp-redux/0240-phenotype-associations/product-738c589063c7970b04ff8106de1848339e247947.tar.gz`, SHA-256 `a2205d41d2cdbd47d6139d679942345b2ad9f270d5535ebc749db7854781e7b3`. It retains the checked source plan, unchanged bodies, original-date replay recordings, complete product results and measurements. Fixed ceiling and exact native next-step checks reuse those complete results without another command or source request.

## Accounting case ownership and current envelope limits

The synthetic attribution uses every ordinary phenotype case and leaves both opt-in native collections ignored. `tests/phenotype-command-budgets.yaml` records full-envelope measurements by named command phase and actual surface. CLI measurements include the newline; MCP measurements count compact structured JSON. Markdown help has separate verb-specific measured limits. Each check multiplies its fixed baseline by 115 percent and rounds up. Each loop variant names its source, fault, kind, attempt, section or explicit-limit choice in the fixed key. Ordinary YAML steps keep their own measured limits and do not share an argv budget table. The YAML `max_bytes` checks separately retain the existing validated source-terms subtraction and include the CLI newline. The runtime never grants headroom from the output it is checking.

| Owning case | Behavior and measured phases |
| --- | --- |
| `tests/cases/phenotype/examples.yaml` | Two parsed search examples and two parsed exact get examples, from their actual case run steps; CLI/MCP equality. |
| `tests/cases/phenotype/help.yaml` | Search/get source choices, all paging arguments, five sections and two examples each, through CLI/MCP help. |
| `tests/cases/phenotype/admission.yaml` | Invalid source, identifier, section and bounds; unsupported snapshot combinations fail before I/O. |
| `tests/cases/phenotype/retired.yaml` | Help rename, ID-set retirement, short limit spelling and deferred HPO search recovery. |
| `runner/phenotype/mod.rs` | Literal native paging, independent complete cards, cache admission, disabled components, timeout and Markdown help; phases name each invocation and recording. |
| `runner/phenotype/states.rs` | P24 empty/absent/failure distinctions, short limit and cold/warm disabled behavior. |
| `runner/phenotype/faults.rs` | Seven command forms across 429, 500, timeout, malformed JSON and source error bodies; five identity/taxon rejections and rejected-cache checks. |
| `runner/phenotype/sections.rs` | First/terminal/explicit-limit text pages, classified Monarch synonyms, hierarchy, positive xrefs, empty/unavailable sections, retained refresh and policy checks. |
| `runner/phenotype/section_failures.rs` | Text section 429, 500, malformed reply, wrong identity and recorded timeout through CLI/MCP. |
| `runner/phenotype/associations.rs` | Exact human gene scope, native pages, malformed endpoints/taxa, failed cache admission, source policy, card actions and recorded timeout. |
| Outside-repo native term proof | All forty-one retained term/search witnesses, including P01–P16, P24, middle/terminal and obsolete cases, owned by `native.rs`. |
| Outside-repo native section proof | Forty-nine source text/link witnesses and twenty revised cards, owned by `section_native.rs`. |
| Outside-repo native association proof | Six exact reverse command pairs, owned by `association_native.rs`; gene-keyed P21–P23 rows remain the declared retained mapper contract. |
| Shared generated help and MCP discovery | `gene/help.yaml`, `help_examples.rs` and `protocol.rs` own whole-tree help, case-derived executable examples and the complete bounded MCP description. |

Runner paths in the table are relative to `crates/biomcp/tests/`. Every design case P01–P24 maps to an owning command case or an explicitly retained outside-repo witness above. Deferred similarity, translations and bundles remain investigation evidence and do not become accepted command cases. Every 0.9 command timing and output size stays unmeasured until the contained comparison in 0098. New exact get and section forms have no 0.9 counterpart.

The initial attribution in `j15009` retained 423 synthetic command/surface observations across 137 grouped phase/surface keys. These grouped observations are historical measurement input, not the final variant limits. Sixty-six cases passed; the help case stopped on an invalid scalar assertion. The scoped `j15010` continuation corrected that assertion and measured all four help surfaces without repeating successful source cases. Search help measures 985 CLI bytes and 984 MCP bytes; get help measures 1,917 CLI bytes and 1,916 MCP bytes. The PM approved their fixed YAML `max_bytes` limits, 1,133 and 2,205, from the full CLI envelopes. The actual combined MCP description measures 2,782 bytes and retains the existing approved ceiling in `protocol.rs`. No phenotype content was removed to fit it. The current complete whole-tree help measures 80,734 CLI bytes. Earlier help and native tables above retain their original revision basis.

The final Rust inventory contains 240 distinct owning-case, command-phase and actual-surface identities. Fifty-three unambiguous measurements were preserved from the earlier observations, including the separately labeled search/get Markdown help sizes. Bounded synthetic job `j15018` ran only the fifty-one Rust cases with missing attribution and supplied exactly the remaining 187 identities. It passed every case. No grouped maximum became an individual variant's baseline. The temporary collector is removed; every unknown phase now fails and every known phase uses only its own fixed measurement plus fifteen percent. The twenty-eight ordinary YAML steps retain their individual measured `max_bytes` and `max_tokens` rather than a second argv budget table.

Scoped `j15014` passed the complete MCP discovery check, whole-tree help, case-derived executable examples and all twenty-five generated-schema checks. The final schema descriptions preserve the same nested request shapes and source applicability as the owning types. No provider command or opt-in native replay ran for this accounting work; the native archives and historical measurements above remain unchanged.
