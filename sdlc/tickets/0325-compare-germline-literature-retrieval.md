# 0325 — Record experiment 497's germline literature retrieval comparison

Status: COMPLETE. Experiment 497 evaluated; findings filed as 0326 to 0329; PDF reading on the later list.

Milestone: 5

## Outcome

This ticket is the record of experiment 497, which Ian asked for on 2026-10-27 and a session outside the team ran. No developer lane works this ticket. Findings are private evidence and stay out of public text.

- Why: Ian asked for a comparison of BioMCP 1.0's public germline literature retrieval with direct NCBI and web search, with issues and output token measurements (Jarvis m9423 and m9494).
- Starts from: the experiment 497 folder, BioMCP 1.0 at 485bebd8, jobs j17053 to j17086 on the build machine, and raw evidence on the evidence store under 0325.
- Result: on six independently verified reference papers, BioMCP's Europe PMC search found 6 of 6, a PubMed alias query found 5 of 6 through BioMCP and through direct NCBI alike, and targeted web search found 6 of 6 after diagnostics. Current-name PubMed found 1 of 6, because four older papers use a historical protein name. BioMCP's supplement reader found a variant row that body search missed, and its body search found a paper whose abstract omits the variant. The 20-variant literal sweep matched direct NCBI's first page in 17 of 20 cases. The BioMCP workload returned about 103,000 tokens over 68 commands against about 439,000 raw tokens for direct NCBI.
- PM evaluation 2026-10-27: real 1.0 defects or gaps became 0326 (spreadsheet first row mislabeled as headers), 0327 (next steps suggest disabled sources and repeat config exclusions), 0328 (dropped variant words not shown first, and no recovery steps for LitVar absence or unsearchable bodies) and 0329 (help does not say which sources the commercial recipe turns off). PDF appendix reading goes to the later list in milestones.md. Union of sources and historical names go to the agent guide after the docs branch lands. Web search query quality and attribution are outside BioMCP.
- Proof: this record, the four tickets and the evidence location.
- Defers: the fixes, in their own tickets.
