# 0514 — Keep the live check's standard output to its JSON report

Status: COMPLETE.
Landed: 82b871733e0d5262ac2e726fcd633b5408e67a24

Milestone: 5

## Outcome

`cargo test -p biomcp --test tools -- --live-check --details` writes only its JSON report to standard output, so the nightly live check parses it. Trial diagnostics go to standard error.

- Why: nightly j24031 at 27b8e7ab8 (m17560) ran 84 live cases with exit 0, but `crates/biomcp/tests/runner/research.rs` line 96 wrote four `research ... compact_bytes=` lines to standard output ahead of the report, so `scripts/nightly_live_check.py` refused the report as invalid JSON. The night before parsed cleanly.
- Starts from: `research.rs` and every other runner `stdout` write reachable in live-check mode.
- Rules: the report is the only standard output in live-check mode; the nightly script keeps refusing a report that does not parse, and never strips leading text.
- Proof: a live-check run over recorded replies whose standard output parses as one JSON value, failing on the base build; the next scheduled nightly parses and classifies.
- Changes: the runner trials that write to standard output.
- Defers: nothing.
