# 0462 — Check BioMCP against cBioPortal's agent question bank

Status: WITHDRAWN. Handed to the evals team, which scores agent answers; failures come back as BioMCP tickets (Ian via Jarvis, m23337 and m23350).

Milestone: 5

Depends on: 0293

## Outcome

Each of the 62 numeric questions in cBioPortal's agent question bank (github.com/cBioPortal/cbioportal-mcp-qa, Apache-2.0) is mapped by hand to 1.0 commands, run against the installed candidate, and compared with the graded answer. The report lists questions answered exactly, answered with a different but explained number, and not answerable, with the capability each unanswerable question needs. cBioPortal's own agent passed 82% of these on 2026-10-15.

- Why: the direct test of whether BioMCP covers what cBioPortal does (m13050, 2026-10-28).
- Starts from: the question bank at a pinned commit and the cBioPortal source in 1.0.
- Rules: free; no model calls. The question bank stays outside the repository. A gap becomes a ticket or a recorded deferral. An agent run over all 146 questions is a paid phase that waits for Ian's cost approval with the other paid runs.
- Proof: a fresh reviewer checks the mapping and every claimed exact answer.
- Changes: none beyond fix tickets.
- Defers: the paid agent run.
