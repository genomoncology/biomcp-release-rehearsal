# 0487 — Fall back to a broader sponsor or site search when the exact phrase finds nothing

Status: COMPLETE.
Landed: 87fca64387247ed8a76a7c47f1efad2f7d5c05c3

Milestone: 5

## Outcome

When `search trial --sponsor` or `--facility` finds no trial for the exact phrase, BioMCP runs ClinicalTrials.gov's broader sponsor or location search (`query.spons`, `query.locn`) and says in the reply that it did. Captures on 2026-10-29 showed that ClinicalTrials.gov's broader search also returns zero for "MD Anderson", so BioMCP reports that zero honestly and adds no alias of its own (m15663).

- Why: BioData's root cause analysis of experiment 502 (m15356, m15395), where an agent writing its own scripts beat BioMCP 0.9.3 on dated cancer questions; the analyst checked this against 1.0 at 19a748042.
- Starts from: the ClinicalTrials.gov adapter in `crates/biomcp-adapters/src/ctgov`, the trial query type, the existing next-step hints and `skills/SKILL.md`.
- Rules: the fallback runs only when the exact phrase returns zero, runs once, and the reply names both requests; no fuzzy matching of BioMCP's own; trial hints name `--sponsor` alongside `--sponsor-type`.
- Proof: a case over recorded replies where the exact sponsor phrase returns zero and the broader search returns a trial, with the fallback named, fails on the base build; the positive pair is found by a bounded capture recipe and verified against the live provider; a recorded case keeps "MD Anderson" returning zero with both requests named; the same for a facility; an exact phrase that finds trials makes one request.
- Changes: the ctgov adapter, the trial query type, hints, the skill, cases and baseline.
- Defers: nothing.
