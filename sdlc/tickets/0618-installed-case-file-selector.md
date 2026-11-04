# Installed cases are selected by their exact file

Status: OPEN.
Milestone: 5

## Outcome

The installed-case runner takes `--case-file <relative/path.yaml>`, resolves it against the target's real case catalog, requires exactly one match, and runs it through the existing `run_selected`; wrapper scripts stop inventing feature names.

- Why: the coordinator's RCA (m24607) found four selector failures across three tickets: 0603 j29053 (feature missing) and j29059 (`.yaml` missing), 0611 j29244 (a guessed path after 593 seconds), and 0596 j28568 (a custom trial with the wrong target). The Rust runner owns the canonical feature and relative path, and `scripts/trials.py` already lists and validates exact names, but wrappers restate them by hand.
- Starts from: `crates/biomcp/tests/runner/` (`run_selected`), `scripts/trials.py` and its tests, and the wrappers that build selectors.
- Rules: exact relative path including `.yaml`; no prefix or substring matching; zero or several matches fail before product builds or case execution, and before all builds when `--runner` is supplied, naming the nearest catalog entries (m24616). No YAML reparse, new registry or framework. Test infrastructure only; no product command changes. The custom trial target issue stays separate.
- Proof: tests show one exact file runs, a missing `.yaml` and an unknown path each fail fast with the catalog names, and one affected installed invocation from the RCA succeeds with `--case-file`.
- Changes: `crates/biomcp/tests/runner/**`, `scripts/trials.py`, `scripts/test_trials.py`; the selector option, the wrapper call sites that guessed names, and tests.
- Defers: nothing.
