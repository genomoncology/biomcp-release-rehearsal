# 0489 — Add three short clinical flows to the skill

Status: COMPLETE.
Landed: 9714f89d6eae0b1e212a48be330afd13f8b2afb0

Milestone: 5

Depends on: 0479, 0481, 0486, 0487, 0492

## Outcome

`skills/SKILL.md` gains three short flows: gene plus coding change to ClinVar, drug brand to its label section, and drugs plus sponsor to a trial search. It stays about its current 3.4 KB.

- Why: BioData's root cause analysis of experiment 502 (m15356, m15395), where an agent writing its own scripts beat BioMCP 0.9.3 on dated cancer questions; the analyst checked this against 1.0 at 19a748042.
- Starts from: `skills/SKILL.md` and the commands from 0479, 0481, 0486 and 0487.
- Rules: the skill stays under 4 KB, because experiment 502 found that the 0.9.3 skill, at 16.5 KB and re-read every turn, caused 54 to 66 percent of BioMCP's token gap; each flow is a few lines of real commands; no prose a command's help already says.
- Proof: a check fails when the skill exceeds 4 KB; each flow's commands run as cases.
- Changes: the skill, its size check and cases.
- Defers: nothing.
