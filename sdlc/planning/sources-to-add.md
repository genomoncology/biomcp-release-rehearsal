# Sources to add after 1.0

Ian, 2026-11-05: "I definitely want to keep a running list of sources to continue to add to BioMCP. These might be too technical, but potentially add them in."

This is a running list, not a plan. Nothing here starts before 1.0 or before Ian's evaluation of what BioMCP agent testing finds (`decisions/2026-11-05-hosting-validates-before-1-0.md`). Each source needs a licence check before hosted use, and a source we hold no licence for needs Ian's approval before anyone fetches it. Add to the list freely and record who asked and why.

## From the hosted evaluation of 2026-11-05

Ranked by how often the gap blocked a real question in that run (hosting m26488).

| Source | What it answers | Asked by |
|---|---|---|
| Regulatory event feed: FDA Novel Drug Approvals, Oncology Center of Excellence notices, Federal Register | What is newly approved for a disease, by date and indication | Hosted evaluation, top pick |
| Drug name normalization: RxNorm and NCI Thesaurus drug synonyms | Code, generic and brand names mapped to one concept across sources | Hosted evaluation, top pick |
| Orange Book and Purple Book | Patents, exclusivity, generics and biosimilars | Hosted evaluation |
| Trial results: ClinicalTrials.gov results section or AACT | Outcome measures and adverse-event tables, to compare trials | Hosted evaluation (BioMCP already reads posted results; check what remains) |
| GDC/TCGA open-access mutation frequencies | Mutation frequency by tumor type | Hosted evaluation (only the open-access tier; controlled TCGA data stays private) |
| ClinGen CNV and dosage, gnomAD structural variants | Copy-number and structural variants | Hosted evaluation |
| Health Canada Drug Product Database, UK MHRA, Japan PMDA | Approvals outside the US and EU | Hosted evaluation |
| CMS National and Local Coverage Determinations | Whether Medicare covers a test or drug | Hosted evaluation |
| bioRxiv and medRxiv direct | Preprints beyond Europe PMC's coverage | Hosted evaluation |

Licensing is hard for treatment guidelines (NICE, open ESMO material), ASCO and ESMO abstracts, and direct COSMIC access; they stay on the list for Ian's call.

## From a functional genomics research group

Relayed by Jarvis (m27130, m27221) from Ian's older notes.

| Source | What it answers |
|---|---|
| DepMap | Gene dependency and drug sensitivity by cell line |
| AlphaSync | Updated predicted structures |
| PDB | Experimental protein structures |
| AlphaFold DB | Predicted protein structures |
| AlphaMissense | Predicted missense pathogenicity |
| MaveDB | Deep mutational scanning sets |

## From the life sciences test of 2026-11-06

Candidates only. Rights are unchecked, and Ian decides after the evaluation.

- Author and institution data with commercial-safe terms, such as OpenAlex, for key opinion leader work if PubMed and Europe PMC fall short (0722).
- A drug name service such as RxNorm or the NCI Thesaurus, if names from current drug sources leave gaps (0717).
- Cell line mutation data, such as DepMap, if Cellosaurus does not carry it (0726).
