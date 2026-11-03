# 0405 — Compare 1.0 with 0.4.6 on public agent benchmarks

Status: WITHDRAWN. Handed to the evals team (experiment 513), which owns comparisons; BioMCP builds tickets (Ian, 2026-11-03, relayed in m23337).

Milestone: 5

Depends on: 0293

## Outcome

The BioMCP tasks in MCP-Bench (https://github.com/Accenture/mcp-bench), and InfoMosaic-Bench's biomedical tasks if their code and data are public, are run against the vendored biomcp-python 0.4.6 server and against the 1.0 candidate. The report compares tool-call success, schema errors and answers, and lists every 0.4.6 tool or argument that agents trained on that surface would reach for and how 1.0 answers it. Every 1.0 regression is fixed before publication or cites the ruling that explains it.

- Why: Ian wants agents to learn and recommend BioMCP, and public benchmarks train and grade agents on the 0.4.6 tool surface (Jarvis m12744, 2026-10-28).
- Starts from: MCP-Bench's 8 BioMCP tasks first (2 single-server, 2 with Paper Search, 4 three-server), its vendored `mcp_servers/biomcp`, its rule-based checks, and the installed 1.0 candidate from 0293.
- Phase 1, no paid model calls: replay each task's 0.4.6 tool calls against 0.4.6 and the matching 1.0 command, mapping each call by hand, and record success, schema errors, data returned and tokens per reply. This phase also measures per-task token sizes for the cost estimate.
- Phase 2, agent runs on Ian's subscription (Ian, 2026-10-29): the agents and the judge run as subscription agents on Ian's plan, so no API keys and no per-call charges are used. Bound (release review C12, 2026-11-01): phase 1 gates the release; Ian approved the bound on 2026-11-01 (m18326): phase 2 runs MCP-Bench's BioMCP tasks and experiment 431's 59 dated cancer questions with BioMCP on 0.4.6 and on 1.0, plus 0490's 44 clinician questions twice on the candidate, about 400 agent runs plus judging, within the subscription's usage windows. cBioPortal's 146-question bank, MedBrowseComp, InfoMosaic-Bench and runs without BioMCP do not run before 1.0. It covers MCP-Bench, InfoMosaic-Bench, experiment 431's 59 dated cancer questions, cBioPortal's 146-question bank and MedBrowseComp's 605 questions, each run with and without BioMCP and against 0.4.6 and 1.0, counting the papers the agent actually read in its tool log. The same agent and judge run every arm, so the comparison between versions is fair; the scores are not the papers' leaderboard numbers, because the papers used their own harness and API models, and the report says so. Runs stay within the subscription's usage limits; anything that would need an API key or a paid plan change goes to Ian first.
- Rules: benchmark code and data stay outside this repository, because MCP-Bench's license is unconfirmed. Keys are inherited by name and never printed or recorded. The NCI source runs only with the existing key. Live calls stay within each source's published limits. The report keeps public IDs, commands and counts.
- Proof: a fresh reviewer checks the report; every regression has a landed fix or a cited ruling.
- Changes: none expected beyond fix tickets the comparison finds.
- Defers: publication, which needs Ian.
