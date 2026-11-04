# 0490 — Run the clinician-worded questions on each release candidate

Status: WITHDRAWN. Handed to the evals team, which scores agent answers; failures come back as BioMCP tickets (Ian via Jarvis, m23337 and m23350).

Milestone: 5

Depends on: 0336

## Outcome

Each release candidate runs experiment 502's 44 round-2 questions, which start from a clinician's words rather than an ID, through an agent with BioMCP, and the report scores them against their answer keys and compares with the previous release candidate cut by 0336, running each question twice.

- Why: BioData's root cause analysis of experiment 502 (m15356, m15395), where an agent writing its own scripts beat BioMCP 0.9.3 on dated cancer questions; the analyst checked this against 1.0 at 19a748042.
- Starts from: BioData's experiment 502 (`questions-round2.jsonl` and its answer keys) and the agent run in 0405, which runs subscription agents on Ian's plan within the bound written in 0405.
- Rules: the questions and keys stay outside this public repository and are read from their private location; the agent under test never sees the keys; the run uses the installed candidate and live sources; a score below the last candidate's is reported with each lost question, and it does not block the release by itself; no paid API key.
- Proof: a run on the 1.0 candidate reports a score per question and the total; a planted wrong answer key lowers the score.
- Changes: a release step and its report script.
- Defers: nothing.
