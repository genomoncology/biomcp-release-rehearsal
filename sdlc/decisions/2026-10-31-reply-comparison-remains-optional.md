# Keep optional reply comparison

Decided 2026-10-31 within the architecture rewrite's delegated choice for the restructure's ruling 8.

Keep `scripts/reply_diff.py` as an optional diagnostic for captured base and head executions. Keep it outside the routine gate. Its bounded difference report and explicit runtime bindings help inspect refactors without creating another comparison framework.

The script does not capture every operation in every trial, and its runtime bindings do not cover every wire, path, clock or pool representation. A difference or tool error remains that result. Review must account for complete records and all public values through the captured data and the affected behavior checks. Unknown values cannot become masks. A zero exit establishes agreement only for the compared captured executions under the implemented substitutions.

`scripts/test_reply_diff.py` checks supported comparisons and their refusals. The implementation and its calibration tests own the substitutions. `docs/development.md` describes capture and review. This decision does not waive public-value equality or settle any pending refactor's proof.
