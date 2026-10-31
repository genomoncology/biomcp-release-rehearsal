# 0484 — Keep only the runner trials a command line cannot express

Status: COMPLETE.

Milestone: 5

Depends on: 0456

## Outcome

The runner keeps about 150 Rust trials, for behavior a command-line call cannot show: MCP framing, HTTP and streaming, TLS, cache contention, the runner's own checks, and the few calls into internal functions that have a stated reason. The other trials go, and so does their helper code.

- Why: Ian's ruling in `decisions/2026-10-29-one-small-outside-in-suite.md`. About 1,320 Rust trials and 351 runner files with 74,000 lines sit beside the cases. About 1,170 of the trials launch the binary with an argv, so they repeat what a case does. 247 only check retired-command refusals, 130 of them for `study` alone.
- Starts from: `crates/biomcp/tests/runner/` and the trial registry in `cases.rs`.
- Rules: the batch with 0485 resets `coverage.floor` to the measured cut-suite value, 78.63 at j23454, as the suite decision says; 0511 sorts what the cut suite no longer reaches (m16827 ask 4). A trial stays only when no command-line case can show its behavior, and its name says which kind it is. A retired command keeps one refusal case in YAML. Helper files that no kept trial uses go. Add no new trial framework. Every shared Rust output ceiling that survives the cut moves to YAML, value for value, so output budgets live only in YAML (from 0431, m15432). After the cut, delete duplicate trial registrations and helpers, then add one small shared function that derives exact trial names from the existing target's `--list` output, keeps the target and its required working directory, refuses an empty selection before any expensive step, and runs that same selection; `--cases` stays a directory option, and no copied manifest or second runner is added (m16101).
- Proof: the trial count and runner line count are reported before and after. Each kept trial names the reason a case cannot replace it. 0483's check still passes. The reply diff is empty. A focused run given a group name, a tools-owned trial, a guessed qualified name or a name without its feature prefix refuses before building, as the four failed jobs in m16101 needed.
- Changes: the runner and the trial registry.
- Defers: nothing.
