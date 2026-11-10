# 0294 — Append this repository's history to the public BioMCP repository

Status: OPEN.

Milestone: 5

Depends on: 0788
Depends on: 0797
Depends on: 0798

## Outcome

After Ian approves, the public BioMCP main gains one merge commit whose parents are its previous main and the approved 1.0 candidate and whose files equal that candidate, so the old history, tags, stars, issues and releases stay and this repository's history follows them.

- Why: Ian ruled on 2026-10-27 to append, not replace (`2026-10-27-append-history-to-biomcp.md`), because the public repository has earlier releases.
- Starts from: that decision, the settled candidate and its actual runtime/artifact qualification, the prepared history-append branch, and freshly read public repository settings. Refresh the preparation after the November 10 fixes and manual checks settle; do not reuse earlier ancestry as the new candidate.
- Rules: nothing is pushed to the public repository before Ian's explicit approval of this step. A rehearsal on a scratch clone shows the merge, that old tags resolve and that the merge tree equals the candidate before the approval request. The release operator performs the push without force.
- Release order (W31, `records/2026-11-06-pre-release-review.md`): one Ian approval covers the append, tag and publication. Prepare the merge locally with current public main first and the approved candidate second. Push only `v1.0.0` on that merge; the ordinary workflow qualifies and publishes without a manual pause. Fast-forward public main to the merge only after publication, public-channel installs and promotion pass. A failed tag run leaves public main unchanged.
- Release session: tag, publication and public-main update occur in one operator session. The docs site is prepared before the cut, then lands and publishes after successful publication/main advancement in the same session under its approval. Preserve the public files and migration guidance already in the candidate.
- Account preflight (W8): this ticket owns the read-only check of the release-publication environment, PyPI Trusted Publisher and channel permissions. Existing environment approval is scoped to that completed change; new account changes and public publication still need their concrete approval. Never inspect or print secret values.
- Publication prerequisites: the existing AWS-first validation/source-assessment ruling remains in force after the November 10 issue hunt and accepted fixes. The blind comparison moved to evals and is not a release dependency.
- Old history findings (m20294, 2026-11-02): the history scan of public rehearsal run 41441571941 found 44 findings in 18 paths reachable only from the old 0.9 `main` and `v0.9.2` (Yellow `biomcp-redux-evidence/0294/public-rehearsal-41441571941/scope-diagnosis.json`). That history is already public. Before the append, review each finding without printing values, bring any finding that is a live credential to the PM for Ian without changing it, and record the outcome; the merged history then scans clean or with reviewed ledger rows. Reviewed 2026-11-02 (m20351, PM accepted): all 44 are noncredentials (37 public identifiers, 4 controlled test inputs, 2 artifact checksums, 1 documented example), classification on Yellow `biomcp-redux-evidence/0294/public-rehearsal-41441571941/legacy-finding-classification-20261102.json` (SHA256 8d24ebd6…). That snapshot need not equal the canonical public BioMCP main, so the append still scans its real merged ancestry.
- Proof: after the push, the public repository shows both histories, v0.9.3 and older tags resolve, stars and releases are intact, and the merge tree equals the approved candidate.
- Changes: preparation records and this ticket; after approval, the public merge/tag and publication sequence.
- Defers: nothing.
