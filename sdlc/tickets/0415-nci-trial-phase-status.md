# 0415 — Filter NCI trials by phase and status

Status: COMPLETE.
Landed: 0184a6fffeaad67f1bb3c43dc15992c32e877fa8

Milestone: 5

Depends on: 0411

## Outcome

`search trial --source nci` accepts `--phase` and `--status`, mapped exactly to NCI's documented `phase` and `current_trial_status` values, which help lists. Values outside NCI's published set refuse before any request.

- Why: `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. MCP-Bench's biomarker task asks for open phase 2 or 3 trials, and NCI refuses both filters today.
- Starts from: `crates/biomcp-types/src/query/trial.rs`, `crates/biomcp-types/src/response/nci_phase.rs` and NCI's documented values.
- Rules: follow `sdlc/planning/architecture.md` and the shared source rules in `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. NCI returns zero, not an error, for a wrong field name, so every NCI proof includes a positive case that a wrong build would fail. Use NCI's published value list only, as 0361 requires; never relabel ClinicalTrials.gov values as NCI values.
- Proof: cases for a phase, a status and both, each asserting at least one trial, and a refused value; bounded keyed captures.
- Changes: the files above, `crates/biomcp-adapters/src/nci`, help and cases.
- Defers: other NCI filters.
