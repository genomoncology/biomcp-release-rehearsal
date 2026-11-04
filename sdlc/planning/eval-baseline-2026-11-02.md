# Agent benchmark baseline, 2026-11-02

The full report is pm mail m21159 (eval runner, experiment 512). Ian approved fixing the general causes and bugs before 1.0 on 2026-11-03; tickets 0586 to 0590.

Build 27eadce, one run per question per arm, same model, BioMCP against curl and Python only. Accuracy 19/20 in both arms. Leaving out one outlier, BioMCP used about 17% more tokens; it used more error calls (25 against 13) and less time (954 s against 1055 s). Where BioMCP has the lookup and the agent uses it, it costs the same or less.

Ian's bar: BioMCP should save agents tokens. Fix general causes through help, skill, JSON and progressive disclosure, and never tune to the 20 questions.

## Defects, costliest cause first

1. Agents don't know a source is in BioMCP: descriptions don't name their source, and the skill names two sources. Fix: name the source in each description, add a covers line to top-level help, and put a one-line-per-source map in the skill.
2. Fixed cost of skill and help: per-command help runs 5 to 11k characters and gets re-read. Fix: help JSON keeps syntax and source names.
3. Bulky default output: the full-text index repeats the card, `next` never suggests `article text --find`, part titles repeat "Main text", and the envelope makes up about 16% of JSON with no field selection.
4. Bugs and errors: free-text label search counts matches but returns none; unsupported label sections don't list valid ones; `label` without a section names the wrong option; misleading usage errors (`help get clingen`, a regex dump for `--coding`, `regulatory` listed but unsupported); retired `search article -q/--query` gets no alias; bundles answer "syncing" on first read; approvals default to one submission.
5. Missing fields: label sections beyond five, application number on the label card, ClinVar submitter comment and variant name, homozygote counts and combined totals, indel resolution, rsID defaulting to GRCh37, ChEMBL max phase and first approval.
6. Missing sources: approval letter text, ClinGen CSpec and report text, Federal Register, posted trial results, GTEx, PubChem, bioRxiv, Gene2Phenotype, DailyMed SPL.

See also `coverage-gaps-2026-11-02.md`.
