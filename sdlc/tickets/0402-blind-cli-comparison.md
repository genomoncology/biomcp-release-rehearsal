# 0402 — Compare 0.9.3 and 1.0 blind before publishing

Status: WITHDRAWN. Handed to the evals team (experiment 513), which owns comparisons; BioMCP builds tickets (Ian, 2026-11-03, relayed in m23337).

Milestone: 5

Depends on: 0459

## Outcome

A fresh agent with no project knowledge runs about 20 tasks with the released 0.9.3 CLI and a 1.0 CLI built locally from main in an empty folder. The report scores each task on both versions for ease of use, tokens spent, chaining, `jq` use, access to the underlying data and answer correctness. Every task where 1.0 does worse is fixed before publication or cites the ruling that explains the difference.

- Why: `decisions/2026-10-28-blind-cli-comparison-before-publishing.md`. Ian, 2026-11-02: run it locally with the CLI before any publishing work, because it will find bugs and ideas; so it runs now on main, and its fixes land before the release candidate is cut.
- Starts from: the 0.9.3 release binary, a release build of main after the BioData switch lands, with its commit recorded, the earlier outside question sets and 0.9.3 comparison evidence.
- Runner (Ian, 2026-11-02): neither the BioMCP PM nor the BioMCP build team or its coordinator drives this comparison. An outside party that Jarvis assigns runs it; BioMCP supplies only the binaries on request and fixes what the findings show.
- Rules: a clinical reviewer chooses the tasks to match real cancer care and biomedical research questions across the entity families, including multi-step and `jq` tasks; tasks are fixed before either binary runs. The agent sees only the two binaries, their own help and the task list. Both versions use the same keys by name, which the agent never prints or records. Correctness is judged against the clinical reviewer's answers, which stay out of Git and away from the agent under test. Live calls stay within each source's published limits. No licensed content goes into the repository; the report keeps public IDs, commands and counts.
- Rubric (PM ruling 2026-10-28): the scoring also uses MCP-Bench's six judged dimensions, so results compare with that public benchmark: task fulfillment, grounding of each claim in tool output, tool choice, argument accuracy, chaining one result into the next call, and call efficiency. The task set includes fuzzy multi-entity briefings like MCP-Bench's: recent papers and preprints in a date window, variant frequency, recruiting trials with sponsors and sites in a named city, trial adverse-event rates, label and approval facts, FAERS safety and device events for test kits, and a pharmacogenomic variant.
- Proof: a fresh reviewer checks the report; every 1.0 regression has a fix ticket on the release candidate's Depends line or a cited ruling. After those fixes land, the regressed tasks run again on the new main.
- Changes: none expected beyond fix tickets the comparison finds.
- Defers: publication, which needs Ian.
