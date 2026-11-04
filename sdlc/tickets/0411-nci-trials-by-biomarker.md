# 0411 — Find NCI trials that list a biomarker

Status: COMPLETE.
Landed: ebc8d576cfe3a861cd265bbc67effe5168cbf83b

Milestone: 5

Depends on: 0410, 0409

## Outcome

`biomcp search trial --source nci --biomarker-code <NCIt code>` returns NCI trials that list that exact concept, and `--biomarker-use inclusion` or `exclusion` narrows by NCI's native criterion. Each trial header reports the native criterion and `inclusion_indicator` of the listed entry with that code. A trial whose entry with that code does not carry the requested use keeps its place with a problem, because NCI does not document that two biomarker parameters bind one entry. A biomarker header names this search as its next step.

- Why: `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. It finds trials from NCI's structured biomarker listings instead of eligibility text.
- Starts from: 0410, `crates/biomcp-types/src/query/trial.rs`, `crates/biomcp-adapters/src/nci` and NCI's trial biomarker fields.
- Rules: follow `sdlc/planning/architecture.md` and the shared source rules in `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. NCI returns zero, not an error, for a wrong field name, so every NCI proof includes a positive case that a wrong build would fail. Capture first whether the public tier returns trial `biomarkers`. A clinical reviewer approves help wording that says NCI lists the concept in inclusion or exclusion criteria; it never says a trial requires positivity or that a patient is eligible. The literal `--variant` text search stays separate.
- Proof: a case for a known code asserts at least one trial and a total equal to the 0410 count for that code and criterion; a case covers a trial with the code in exclusion and another concept in inclusion; bounded keyed captures.
- Changes: the files above, the biomarker next step, help and cases.
- Defers: searching descendant concepts.
