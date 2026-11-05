# 0294 — Append this repository's history to the public BioMCP repository

Status: OPEN.

Milestone: 5

Depends on: 0293, 0633

## Outcome

After Ian approves, the public BioMCP main gains one merge commit whose parents are its previous main and the approved 1.0 candidate and whose files equal that candidate, so the old history, tags, stars, issues and releases stay and this repository's history follows them.

- Why: Ian ruled on 2026-10-27 to append, not replace (`2026-10-27-append-history-to-biomcp.md`), because the public repository has earlier releases.
- Starts from: that decision, the `release/1.0` candidate in 0293, and the public repository's settings.
- Rules: nothing is pushed to the public repository before Ian's explicit approval of this step. A rehearsal on a scratch clone shows the merge, that old tags resolve and that the merge tree equals the candidate before the approval request. The release operator performs the push without force.
- Release order (PM ruling 2026-10-28, pre-release review `sdlc/records/2026-10-28-pre-release-review.md`): one Ian approval covers the append, the tag and publication. Then the append, then the `v1.0.0` tag on the public merge commit, then the release workflow on that tag, then the operators publish from that run. This ticket owns the tag step.
- Release review C4 (2026-11-01): the append, the tag and publication run in one operator session, because the plugin marketplace serves the repository root and switches plugin users at the merge. The public files `CITATION.cff`, `.zenodo.json`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` and a 1.0 `CHANGELOG.md` are in the candidate through 0534. Publication runs through 0533's gated jobs.
- Old history findings (m20294, 2026-11-02): the history scan of public rehearsal run 41441571941 found 44 findings in 18 paths reachable only from the old 0.9 `main` and `v0.9.2` (Yellow `biomcp-redux-evidence/0294/public-rehearsal-41441571941/scope-diagnosis.json`). That history is already public. Before the append, review each finding without printing values, bring any finding that is a live credential to the PM for Ian without changing it, and record the outcome; the merged history then scans clean or with reviewed ledger rows. Reviewed 2026-11-02 (m20351, PM accepted): all 44 are noncredentials (37 public identifiers, 4 controlled test inputs, 2 artifact checksums, 1 documented example), classification on Yellow `biomcp-redux-evidence/0294/public-rehearsal-41441571941/legacy-finding-classification-20261102.json` (SHA256 8d24ebd6…). That snapshot need not equal the canonical public BioMCP main, so the append still scans its real merged ancestry.
- Proof: after the push, the public repository shows both histories, v0.9.3 and older tags resolve, stars and releases are intact, and the merge tree equals the approved candidate.
- Changes: none in this repository.
- Defers: nothing.
