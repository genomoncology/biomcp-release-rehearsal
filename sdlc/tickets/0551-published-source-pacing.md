# 0551 — Pace sources from their published policies

Status: OPEN.

Milestone: 5

Depends on: 0548

## Outcome

Each source's pacing in the quota registry cites that source's published usage policy, or says that none exists, and pacing rises where the policy allows. Each source's rate and in-flight count can be lowered or raised in `config.json` and by flag, but never above its published limit.

- Why: the hosted team's capacity review (m18618, Ian approved the request). `crates/biomcp-adapters/src/quotas.rs` paces most sources at 1 request a second with 1 in flight, and its comments say no published numeric limit was checked; only NCBI and openFDA cite one. ClinicalTrials.gov, Europe PMC and BioThings matter most for cancer care questions. The hosted team needs to lower a source fast if it complains.
- Starts from: `biomcp-adapters/src/quotas.rs`, `biomcp-fetch/src/quota.rs` and the settings shape from 0548.
- Rules: research every registry entry's published policy, starting with ClinicalTrials.gov, Europe PMC and BioThings, and cite the policy URL and date in the comment beside the entry. A source with a published limit gets that limit as its ceiling. A source with none keeps a cited conservative default as its ceiling. Overrides are one `config.json` map keyed by source name with rate and in-flight values, plus a matching flag; an unknown source, a zero or a value above the ceiling is refused at startup with a named error. Keyless and keyed rates stay separate. No reply changes.
- Proof: a test reads every registry entry and fails on one with neither a policy citation nor a recorded absence; tests set an override lower, at the ceiling and above it, and an unknown source; the docs list each source's default and ceiling.
- Changes: the quota registry, the settings file and flag, their tests and the configuration reference docs.
- Defers: merging identical in-flight keyless requests and measuring the shared cache connection at 32 or more calls in flight, after launch when traffic asks for it.
