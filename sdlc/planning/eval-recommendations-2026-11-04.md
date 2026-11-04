# BioMCP recommendations from the evals team

2026-11-04. From the evals team's experiment comparing agents with and without BioMCP. This copy is self-contained: work from it, not from the evals team's folders, and send questions through Jarvis. Questions are named by type only, and no benchmark question text, case ID, subject or key appears here.

How to read it:
- Ratios are against a cold agent: the same model with bash, curl and python3, and no BioMCP.
- Identical rounds of the same setup have differed by up to about 30%. So only changes larger than that count as **shown**. Smaller ones are marked **suggestive**.
- The evals team's fixes were tuned on five questions, so evidence from those five is marked **tuned**.

## Shown

### 1. Accept names, or say exactly what to run instead
- **The finding:** on BioMCP main 2ffbeab2, `get drug` takes only an exact source ID plus `--source`. It does no name resolution.
- **Witness:** `biomcp get drug imatinib approvals` exits 2 with `usage.arguments`. The error lists ten valid sources, and its only next step is `help get drug`. Nothing says to run `search drug` first, or which source fits a name.
- **Evidence:**
  - With an older command map in the prompt, 58 of 69 `get drug` calls on main hit this error across one arm of 20 questions at 2 runs each.
  - On a drug-regulatory question (tuned), median tokens were: cold agent 346k; BioMCP 1.0 316k; the evals team's patched version with name matching across EMA and openFDA 82k; main with a map 161k; main with its own help only 255k.
- **Recommendation:** either resolve brand and generic names across sources, or make the usage error give the exact next command for the input. State the ambiguity rule: which record wins when a name matches several.

### 2. The fix bundle beats the cold agent on held-back questions
- **The bundle:**
  - name resolution;
  - any variant ID form, including gnomAD-style indels;
  - drug approvals newest first, capped at 10 rows;
  - a cold source sync that gives up after 60 seconds;
  - an empty-result note when the source's own answer is "absent".
- **Evidence:**
  - On ten questions it was not tuned on: 0.71× the cold agent, or 0.78× counting one run that answered but crashed in cleanup. It lost fewer fields (6 against 10).
  - On the five tuned questions: about 0.60×.
- **The limit:** main 2ffbeab2 with a map was 0.64× on the same ten. So on held-back questions, today's main with a map is as good as the bundle. The items above are not shown one by one, only as a bundle, and against an older base.

## Suggestive (below the noise level, worth BioMCP's judgment)

### 3. Put a short command map where agents read first
- **Evidence:** the same main binary, on 15 questions, used 0.89× the cold agent with its own help only, and 0.70× with a one-page map in the prompt. That is a 21% change, under the noise level.
- **What our map adds:** main's `biomcp help` already lists commands, sections and sources. Ours puts it in the agent's prompt, pairs each section with its source, and adds a few trap lines. The agents made about as many help calls as data calls without it.
- **Recommendation:** put the map in the skill or the MCP instructions, not one help call away.

### 4. MCP shape
- **Evidence:**
  - Over MCP in pi, a single tool that takes CLI arguments cost less than 14 typed tools (0.83× and 0.92× in two pairs) and lost fewer fields. The typed tools' descriptions cost about 546k tokens to read.
  - A pi prompt line saying "scripts can call `tools.bash`" cut single-tool MCP by 21%. pi shows only one line of an MCP server's instructions, so this is a hint for whoever writes the agent's prompt, not something BioMCP can fix alone.
  - pi's tool search found the typed tools with no map. Every run searched, but that arm still cost 1.38× the cold agent.
  - With the bash line, MCP still cost about 1.3–1.5× the CLI.

## Gaps (claims about coverage, not token savings)

Found while writing new 2026 questions, against 1.0 with the patched fixes. **Not yet confirmed on main 2ffbeab2:** main's help lists `fda-pma`, `fda-510k`, `fda-cdx`, `who-pq` and a trial `outcomes` section, so some may already be closed. The evals team will follow up with each gap rechecked on main and a witness command; treat this list as provisional until then.
- posted trial results: outcome measures, adverse-event tables and the results-posted date;
- full label sections, search by indication, and labels only on DailyMed or as PDFs;
- FDA approval letters, device approvals and device safety summaries;
- a device adverse-event search with a working date filter, and drug adverse-event search across several names at once;
- gnomAD population, homozygote and joint totals;
- rows from a cBioPortal study, not only finding the study;
- a GWAS Catalog study found from its paper, a pathway search by gene symbol, and a cell-line search by disease;
- CIViC evidence searched by drug and evidence level;
- WHO lists filtered by year.

## Tried, and not shown to help
- **Per-command help cards** showed no gain over the map.
- **A help front page on its own,** without the source names, was not shown.
- **Compact replies** were inconclusive.

## Appendix: the one-page map used in the prompt

It was written for BioMCP 1.0 at 27eadce. Some command shapes differ on main 2ffbeab2, notably `get drug`. It is offered as a model of the shape, not as text to paste.

```markdown
# BioMCP map
`biomcp search <entity> <text>` finds; `biomcp get <entity> <id> [section...]` reads; `biomcp help <verb> <entity>` for flags. Sources in ( ).
- article (PubMed, Europe PMC, PMC): citation-evidence; citations (iCite); pdf; recommendations; references; supplement; text; get [annotations (PubTator3)|fulltext|supplements|tldr (Semantic Scholar)|citations|indexing|pdf]; search
- author: papers [details]; get; search semantic-scholar
- cell-line: assay-count; get [names|identity|xrefs|variants|chembl]; search
- adverse-event (FAERS, MAUDE, VAERS): get [drugs|reactions|unii|rxcui|product-ndc|package-ndc|spl-id|spl-set-id|application-number|devices|narrative]; search [symptoms|seriousness|age]
- biomarker: get nci; search nci
- diagnostic (GTR, WHO IVD, FDA PMA/510k/CDx): get [genes|conditions|methods|regulatory|supplements]; search
- disease (OLS, MyDisease, Monarch, SEER): get [synonyms|xrefs|hierarchy|genes|phenotypes|incidence|survival]; search
- drug: get [xrefs (ChEMBL)|targets (ChEMBL)|indications|civic (CIViC)|label (openFDA SPL)|approvals (Drugs@FDA)|regulatory (EMA EPAR)|interactions (DDInter)]; search
- gene: get [protein (UniProt)|civic|clingen (ClinGen)|gencc (GenCC, includes Gene2Phenotype)|constraint (gnomAD)|expression (HPA)|pathways (Reactome)|oncokb]; search
- gwas (GWAS Catalog): get [loci|associations|ancestries]; search
- pathway (Reactome, WikiPathways): get [summary|genes|events]; events; genes; search
- pgx: get [recommendations|guidelines|diplotype (CPIC)|annotations (ClinPGx; --gene, --drug)]; search cpic; get --annotation-id PA… (ClinPGx clinical)
- phenotype (HPO, Monarch): get [synonyms|xrefs|hierarchy|diseases|genes]; search
- protein (UniProt): get [features|domains|isoforms|complexes|structures|function|sequence]; complex-members; complexes; domains; features; isoforms; sequence; structures; search
- study (cBioPortal): get [profiles|sample-lists|files]; search
- trial (ClinicalTrials.gov, NCI): get [eligibility|outcomes|arms|locations|contacts|references|documents|diseases|summary]; search; document
- variant: get [clinvar (ClinVar)|erepo (ClinGen eRepo)|population (gnomAD via MyVariant)|gwas|civic|alphagenome|oncokb|hotspots|predict|conservation|cosmic]; search myvariant; assertions; normalize --source car (ClinGen Allele Registry); population (gnomAD); recurrence cbioportal
- organization: search nci
```
