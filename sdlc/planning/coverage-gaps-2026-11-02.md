# Coverage gaps from the benchmark lookup map

Source: an outside eval runner mapped BioMCP main at b1946c816 against the lookups that 20 benchmark questions need, with no question text or answers (2026-11-02). Of 97 lookups, 57 are fully covered, 32 reach the source but miss a field, filter or document, and 8 need a source BioMCP does not reach. None of these is a ticket. Ian's plan: run a baseline on the 1.0 release candidate first, prioritize these gaps from that run, then compare with an agent writing its own scripts. This does not change 1.0's scope.


These gaps come from mapping a set of benchmark lookup chains against BioMCP `main` at b1946c816. Each gap is written as a general capability an agent would need, not a fix for one question.

- **Question labels.** Labels such as D1 or T4 stand in for benchmark questions, and their key is kept outside this file.
- **Hop verdicts.** A "partial" hop means BioMCP reaches the source, but not the field, filter, section or document needed. A "not covered" hop means BioMCP doesn't reach the source.

Ordered by how many questions each source affects.

## Summary

| Rank | Source | Questions | Recommendation |
|---|---|---|---|
| 1 | FDA drug label (openFDA label, DailyMed) | 9 | Add every label section, an indication search, and DailyMed for fresh labels |
| 2 | ClinicalTrials.gov | 7 | Add a posted-results section; add design and identification fields to the card |
| 3 | ClinGen gene–disease validity | 4 | Add report evidence text; add filters across the bundle |
| 4 | ChEMBL | 4 | Add a target lookup, mechanisms by target, and molecule phase and approval fields |
| 5 | Drugs@FDA and FDA regulatory documents | 4 | Add date and submission filters; read approval letter text |
| 6 | EMA | 3 | Add active-substance search and assessment documents |
| 7 | Federal Register | 2 | Add, scoped to FDA documents |
| 8 | ClinGen Criteria Specification Registry (CSpec) | 2 | Add |
| 9 | cBioPortal | 2 | Add clinical data, patient counts and gene-level mutation rows |
| 10 | gnomAD | 2 | Add homozygote, hemizygote and rsID fields |
| 11 | GTEx | 2 | Add expression and eQTL sections |
| 12 | GWAS Catalog | 1 | Add study search |
| 13 | PubMed | 1 | Add comment and reply links |
| 14 | bioRxiv / medRxiv API | 1 | Add a preprint version section |
| 15 | PubChem | 1 | Add identity lookup only |
| 16 | ClinPGx | 1 | Add label annotation detail and a drug filter |
| 17 | Gene2Phenotype | 1 | Don't add yet; carry GenCC's report link instead |
| 18 | Article full text (back matter, publisher-only text) | 1 | Add back matter; leave publisher-only text alone |

## 1. FDA drug label sections, indication search and fresh labels

- **Source:** FDA Structured Product Labeling, through openFDA drug label (https://api.fda.gov/drug/label.json), which BioMCP already reads, and DailyMed web services v2 (https://dailymed.nlm.nih.gov/dailymed/services/v2/), which it doesn't.
- **Credential:** none. openFDA takes an optional key for higher rate limits, and DailyMed needs no key.
- **Entity and fields:** drug labels. BioMCP exposes five headings today: boxed warning, dosage and administration, drug interactions, indications and usage, and warnings and cautions. Missing are:
  - clinical studies
  - mechanism of action and clinical pharmacology
  - pharmacogenomics
  - recent major changes
  - adverse reactions
  - use in specific populations
  - contraindications
  - description
- **Needs:**
  - **Sections:** mechanism of action, clinical studies (including the NCT IDs and results labels cite), pharmacogenomics, and recent major changes. Needed by D1, D2, D3, D4, D5, T1, T2, T3 and L6 (9 questions).
  - **Search by indication text:** "which labels are indicated for disease X". Needed by D1, T2 and T3 (3).
  - **Fresh labels:** labels from the last few weeks that openFDA hasn't indexed yet, which DailyMed already serves. Needed by D1 (1).
- **Scope:** fits. Labels are core to cancer care, rare disease and other medical questions.
- **Recommendation:** add.
  - Expose every SPL section openFDA publishes, under its native field name.
  - Add an `--indication` text filter to `search drug --source openfda-label`.
  - Add DailyMed as a second label source for the same `label` section, so the newest labels and full section text can be reached.
  - Reason: section-level label reading is the most common need in this set. It is a generic gap for anyone asking what a label says about efficacy, mechanism or pharmacogenomics.

## 2. ClinicalTrials.gov posted results and design fields

- **Source:** ClinicalTrials.gov API v2 (https://clinicaltrials.gov/api/v2/studies/{NCT}), already read by BioMCP.
- **Credential:** none.
- **Entity and fields:** trials.
  - Today BioMCP maps the protocol only: card, eligibility, registered outcomes, arms, sites, contacts, references and documents.
  - It never reads `resultsSection`: participant flow, baseline, outcome measures with their values and statistical analyses, and adverse events (serious and other, by term and group, with numbers at risk).
  - The card also lacks the acronym, org study ID, secondary IDs, and design details: allocation, masking, primary purpose and enrollment.
- **Needs:**
  - **Posted outcome values and adverse event tables:** needed by T1, T2, T3, T6 and L2 (5).
  - **Design and identification fields,** to match a trial by acronym or sponsor protocol code, or a design described elsewhere: needed by D5 and D6 (2).
- **Scope:** fits.
- **Recommendation:** add.
  - Add a `results` section to `get trial` with independent pages for outcome measures (with analyses) and adverse events.
  - Add the acronym, org study ID, allocation, masking and has-results flag to the trial card.
  - Reason: posted results are public, structured and stable once posted. Today an agent must leave BioMCP for raw `curl` to read them.

## 3. ClinGen gene–disease validity: report text and cross-gene filters

- **Source:** ClinGen gene-disease validity (https://search.clinicalgenome.org/kb/gene-validity/download, already read) and the per-curation report pages linked from its ONLINE REPORT column.
- **Credential:** none.
- **Entity and fields:** gene–disease curations.
  - BioMCP has the export columns: gene, disease, MONDO, mode of inheritance, classification, date, panel and report URL.
  - It lacks the report content: the evidence summary, scored publications, proband and variant counts, and case-control studies scored and not scored.
- **Needs:**
  - **Report evidence summary text:** needed by G2, G4, G5 and G6 (4).
  - **Filter across all curations** by panel, classification, date range or disease (today reads are keyed by gene only): needed by G2 (1).
- **Scope:** fits (rare disease and hereditary cancer).
- **Recommendation:** add.
  - Add a `report` section that reads the curation's evidence summary and scored evidence from the linked report. Check whether ClinGen offers a structured export of the same report before choosing to parse HTML.
  - Add panel, classification, date and disease filters over the already synced bundle. They are cheap, because the bundle is local.

## 4. ChEMBL targets, mechanisms by target, and molecule development fields

- **Source:** ChEMBL REST (https://www.ebi.ac.uk/chembl/api/data/), already read for molecules, mechanisms by molecule and indications.
- **Credential:** none.
- **Entity and fields:**
  - **Target records:** target ChEMBL ID, type, organism, and components with UniProt accessions.
  - **Mechanisms filtered by target.**
  - **Molecule fields:** `max_phase`, `first_approval` and availability type. Today only per-indication `max_phase_for_ind` and Open Targets `maxClinicalStage` are exposed.
- **Needs:**
  - **Target ID from a protein or gene:** needed by R1, R2 and R4.
  - **Molecules with a mechanism against a target, with approval phase and year:** needed by R2.
  - **Molecule highest phase:** needed by R1, R4 and R6.
  - Together these touch 4 questions.
- **Scope:** fits (biopharma research from public data). This is lookup of existing records, not design.
- **Recommendation:** add.
  - Add `max_phase` and `first_approval` to the ChEMBL drug card.
  - Add a ChEMBL target lookup by UniProt accession or gene, perhaps as a `chembl` section of `get protein` or `get gene`.
  - Add a section listing mechanisms against that target.
  - Reason: "which drugs act on this target, and how far have they got" is a basic target question, and today it can only be answered from the drug side.

## 5. Drugs@FDA discovery filters and FDA approval documents

- **Source:** openFDA Drugs@FDA (https://api.fda.gov/drug/drugsfda.json, already read) and the FDA documents its `application_docs` link to (approval letters and review PDFs on accessdata.fda.gov).
- **Credential:** none.
- **Entity and fields:** applications and submissions.
  - BioMCP already reads every submission's type, number, status date, class and document URLs.
  - It lacks search by submission type, class or date window (search requires a name), and it doesn't read the content of approval letters.
  - Letters hold the approved change, postmarketing requirements and commitments with timetables, accelerated approval conditions, and voucher numbers.
- **Needs:**
  - **Letter content:** which supplement approved which indication, plus postmarketing requirements. Needed by D5, T1 and T6 (3).
  - **Approvals in a date window or of a submission type:** needed by D1 (1).
- **Scope:** fits.
- **Recommendation:** add.
  - Add `--submission-type`, `--submission-class` and an approval date range to `search drug --source drugsfda`.
  - Add a section that reads the text of a listed FDA approval letter. BioMCP downloads PDFs today but never parses them, so this needs a bounded PDF text reader for FDA-hosted documents.
  - Reason: letters are the primary record of what each supplement approved.

## 6. EMA active-substance search and assessment documents

- **Source:** EMA website data in JSON (https://www.ema.europa.eu/en/about-us/about-website/download-website-data-json-data-format). BioMCP reads the medicines report; EMA also publishes document and referral reports, and EPAR, refusal and withdrawal question-and-answer pages.
- **Credential:** none.
- **Entity and fields:** EU medicines and applications.
  - BioMCP has status, opinion status, every named date and indication.
  - It lacks search by active substance or INN (only `name_of_medicine` words are indexed).
  - It also lacks the committee's stated grounds for refusal, withdrawal or revocation, which sit in EPAR, refusal or withdrawal summaries and referral outcomes.
- **Needs:**
  - **Grounds stated by the regulator:** needed by D2, D3 and D4 (3).
  - **Every application for one active substance:** needed by D3 (1).
- **Scope:** fits.
- **Recommendation:** add.
  - Index the active-substance and INN columns the medicines report already carries.
  - Add a `documents` section listing the EPAR and summary documents for a product.
  - Add a reader for the regulator's summary of grounds, or the referral outcome, where EMA publishes it as text.

## 7. Federal Register

- **Source:** Federal Register API (https://www.federalregister.gov/api/v1/documents.json), from the US Office of the Federal Register.
- **Credential:** none. The content is public domain.
- **Entity and fields:** published notices and rules, with document number, publication date, agency, docket IDs, type, title, abstract and full text (`raw_text_url`, `body_html_url`).
- **Need:** FDA notices that withdraw or propose to withdraw an approval, with their dates, dockets, application numbers and narrative history. Needed by D2 and D6 (2).
- **Scope:** fits. FDA regulatory actions on drugs are medical questions.
- **Recommendation:** add.
  - Scope it to Food and Drug Administration documents, either as a `regulatory` source for drugs (search by drug name or application number) or as its own search.
  - Reason: withdrawals and other formal FDA actions appear first, and sometimes only, in the Federal Register.

## 8. ClinGen Criteria Specification Registry (CSpec)

- **Source:** ClinGen CSpec (https://cspec.genome.network/cspec/api/svis and `/cspec/api/SequenceVariantInterpretation/id/{GNxxx}`).
- **Credential:** none.
- **Entity and fields:** each variant curation expert panel's gene-specific ACMG/AMP specification, with:
  - CSpec ID, panel and affiliation, gene, and disease with MONDO and mode of inheritance
  - version and release date, and status
  - each criterion's code, strength, applicability and rule text (frequency thresholds, predictor cut-offs, hotspot lists)
- **Need:** the current specification a panel applies to a gene, and the exact thresholds in its criteria. Needed by G1 and L5 (2).
- **Scope:** fits (germline variant interpretation for rare disease and hereditary cancer).
- **Recommendation:** add.
  - Add as a `cspec` section of `get gene`, plus `get` by CSpec ID.
  - Reason: anyone reading an expert panel's ClinVar classification needs the rule set it was made under.

## 9. cBioPortal clinical data and gene-level mutation rows

- **Source:** cBioPortal public API (https://www.cbioportal.org/api), already read for the study catalog and exact-allele recurrence.
- **Credential:** none for public studies.
- **Entity and fields:** study cohorts. Missing are:
  - patient counts (the card's patient count is always empty)
  - sample-level and patient-level clinical attributes, such as detailed cancer type and recorded treatments
  - gene-level mutation rows with protein change and patient ID; today rows carry only sample ID and genomic allele, one exact allele per call
- **Needs:** cohort composition, mutation calls in a gene with protein changes, and the joins between samples, patients and clinical attributes. Needed by L6 and T4 (2).
- **Scope:** fits (cancer research from public data).
- **Recommendation:** add.
  - Add `clinical` (sample and patient attributes) and `mutations` (by gene, with protein change, sample and patient IDs) sections to `get study`, paged as native rows.
  - Fill patient counts from the source.
  - This keeps BioMCP's rule that it computes no frequencies: it returns rows, and the agent counts.

## 10. gnomAD homozygote, hemizygote and rsID fields

- **Source:** gnomAD GraphQL (https://gnomad.broadinstitute.org/api), already read.
- **Credential:** none.
- **Entity and fields:** variants. BioMCP's query requests AC, AN, filters and populations per exome and genome. It omits:
  - homozygote and hemizygote counts
  - rsIDs
  - the joint exome-plus-genome fields that gnomAD v4 publishes
- **Need:** homozygote counts, which matter for recessive disease, and combined counts. Needed by L1 and G3 (2).
- **Scope:** fits.
- **Recommendation:** add these fields to the existing population section. Reason: for recessive conditions the homozygote count is often the deciding population fact.

## 11. GTEx

- **Source:** GTEx Portal API v2 (https://gtexportal.org/api/v2/).
- **Credential:** none for summary data.
- **Entity and fields:**
  - **Genes:** median expression per tissue, by dataset release.
  - **Variants:** single-tissue eQTLs and sQTLs, with gene, tissue, effect size (NES), p-value and dataset.
- **Need:** a gene's tissue expression profile in a named GTEx release, and whether a variant is an eQTL in which tissue and in which direction. Needed by R4 and T5 (2).
- **Scope:** fits (life sciences research, and interpreting GWAS hits).
- **Recommendation:** add.
  - Add a `gtex` expression source for `get gene ... expression` and an `eqtl` section of `get variant`.
  - Reason: GTEx is the standard public answer to "is this variant regulatory, and where".

## 12. GWAS Catalog study search

- **Source:** GWAS Catalog REST v2 (https://www.ebi.ac.uk/gwas/rest/api/v2/studies), already read for associations and single study cards.
- **Credential:** none.
- **Entity and fields:** studies by PubMed ID, trait or ancestry, with accession, trait IDs, sample sizes and ancestries.
- **Need:** every study curated from a paper, without paging every association row to collect accessions. Needed by R5 (1).
- **Scope:** fits.
- **Recommendation:** add `search gwas-study --pubmed/--trait-id`, or a studies mode on `search gwas`. Reason: papers often yield several studies, and today the agent must page through all their associations.

## 13. PubMed comment and reply links

- **Source:** PubMed E-utilities efetch (already read), CommentsCorrections element.
- **Credential:** none (optional NCBI key).
- **Entity and fields:** articles. BioMCP maps retraction, erratum and expression-of-concern links. It drops these relation types:
  - Comment in / on
  - Reply in / to (if present)
  - Update in / of
  - Republished in / from
- **Need:** letters, replies and updates linked to a paper. Needed by G2 (1).
- **Scope:** fits.
- **Recommendation:** add the remaining CommentsCorrections relation types to the existing corrections field. It is a small change with wide use for following debate around a paper.

## 14. bioRxiv / medRxiv API

- **Source:** https://api.biorxiv.org (details and publisher endpoints, by DOI).
- **Credential:** none.
- **Entity and fields:** preprints, with every version and its date, category, license, and the DOI and journal of the published version.
- **Need:** a preprint's latest version and its published counterpart. Needed by R3 (1).
- **Scope:** fits.
- **Recommendation:** add as a `versions` section of `get article` for bioRxiv and medRxiv DOIs. Reason: Europe PMC finds preprints, but BioMCP's preprint card carries no version history or published link.

## 15. PubChem

- **Source:** PubChem PUG REST (https://pubchem.ncbi.nlm.nih.gov/rest/pug/).
- **Credential:** none.
- **Entity and fields:** compound identity, with CID, synonyms, molecular formula, InChIKey and cross-references.
- **Need:** a drug's PubChem identity fields. Needed by R6 (1).
- **Scope:** fits as identity lookup by name or ID. It must stay away from structure search, similarity search or property-driven discovery, which edge toward molecule design.
- **Recommendation:** add as a drug identity source limited to name and CID lookup and identity fields. Reason: PubChem IDs are the common cross-reference for small molecules, and MyChem and ChEMBL in BioMCP don't expose them.

## 16. ClinPGx label annotation detail

- **Source:** ClinPGx API (https://api.clinpgx.org/v1/data/labelAnnotation), already read for gene-keyed reference lists.
- **Credential:** none.
- **Entity and fields:** FDA, EMA and other regulator label annotations. BioMCP lists only ID and name, by gene. It lacks:
  - the genetic testing level
  - the label source and agency
  - the related chemicals
  - the annotation text
  - a filter by drug
- **Need:** the testing level ClinPGx assigns to a drug–gene label annotation. Needed by D3 (1).
- **Scope:** fits (pharmacogenomics).
- **Recommendation:** add a label annotation detail read and a chemical filter. Reason: the testing level is the main thing users want from those annotations.

## 17. Gene2Phenotype

- **Source:** Gene2Phenotype API (https://www.ebi.ac.uk/gene2phenotype/api/).
- **Credential:** none.
- **Entity and fields:** locus–genotype–disease records, with record ID, confidence, allelic requirement, mechanism, panels and publications.
- **Need:** the curated record and confidence for a gene–disease pair. Needed by G3 (1).
- **Scope:** fits.
- **Recommendation:** don't add as a separate source yet. Gene2Phenotype submits to GenCC, which BioMCP already reads. Map GenCC's per-submission public report link instead, if the export carries one. Revisit if allelic requirement or mechanism fields become a recurring need.

## 18. Article full text: back matter and publisher-only text

- **Source:** PMC Open Access JATS (already read) and publisher open-access pages.
- **Credential:** none.
- **Entity and fields:** articles.
  - **Back matter:** BioMCP reads the JATS `<body>` with tables. It skips `<back>`, where data availability statements, accession lists, funding and notes often sit.
  - **Publisher-only articles:** open-access articles that aren't in PMC can be saved as a PDF, but BioMCP never reads them.
- **Needs:**
  - **Full text of a recent open-access article not yet in PMC:** needed by R1 (1).
  - **Back matter:** a risk for any hop that reads a data statement (none failed outright here).
- **Scope:** fits.
- **Recommendation:**
  - Add back matter to `article text` (low cost, generic value).
  - Don't add publisher HTML scraping. It is fragile, and terms vary by publisher.

## Findability problems (no new source needed)

These need no new source. They are reachable today but easy to miss through help, skills and progressive disclosure:

- **Bundle syncs.** EMA, GenCC and cBioPortal reads fail until the agent syncs a local bundle, while the skill says only HGNC and ClinGen sync themselves. Auto-sync under the same byte cap, or name each bundle in the skill.
- **Search syntax.** Europe PMC and PubMed accept their native field syntax through the free-text argument, but help calls it "words". Say so, and add a preprint-only filter.
- **Code-name lookup.** ChEMBL code names need `--name-selector synonym`. A miss on the preferred name could suggest the synonym search.
- **Criteria codes.** Expert panel criteria codes sit in free-text SCV comments. Mention this in the variant skill.
- **gnomAD totals.** Exome and genome counts come separately. Help should say there is no combined total.
- **Drug target genes.** The easiest route to a drug's target gene is `get drug ... targets --source opentargets`. The label skill could point there.

