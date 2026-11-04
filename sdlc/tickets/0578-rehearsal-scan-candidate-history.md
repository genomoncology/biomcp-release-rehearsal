# Release history scan covers the candidate's own history

Status: COMPLETE.
Landed: f7b14460e206158eb3edce133115a4600dc4e5ea
Milestone: 5

## Outcome

In any repository the release workflow runs in, the history secrets scan and ledger cover the complete ancestry of the candidate commit and nothing else, so refs that already exist in the target repository, such as the public rehearsal repository's old 0.9 `main` and `v0.9.2`, do not enter a candidate's scan.

- Why: public rehearsal run 41441571941 at 372102490 failed the history scan. `actions/checkout` with `fetch-depth: 0` (`.github/workflows/release.yml:37-40`) fetched every old head and tag, and `release/secrets.py:95` scans `--all HEAD`: 11,901 commits and 149,590 objects instead of the candidate's 5,660 commits and 86,540 objects. All 44 findings in 18 paths are reachable only from the unchanged old 0.9 refs, none from the candidate (Yellow `biomcp-redux-evidence/0294/public-rehearsal-41441571941/scope-diagnosis.json`, SHA256 e81c4ca4…). This proves scope only; it does not show those values are harmless.
- Starts from: the checkout steps in `release.yml` that feed the scan, `release/secrets.py` scope, the 0555 ledger binding.
- Rules: limit the checkout to the candidate's complete ancestry, for example a full fetch of only the candidate ref with `--no-tags`, and prove the fetched ref and object counts equal the candidate's. Keep the scanner's `--all` default, its ledger and archive guards, and every remote ref unchanged. Disclose, suppress or waive no finding. Changing the scanner's default instead would need its own ruling.
- Proof: a workflow or fixture test that fails when an unrelated ref enters the scan; the next public rehearsal's scan reports the candidate's commit and object counts and passes.
- Changes: the release workflow's checkout for the scan, and its tests.
- Defers: the 44 findings in the old 0.9 history go to the public history append ticket (0294), which merges that history and must review them before the append.
