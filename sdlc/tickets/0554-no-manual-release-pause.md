# 0554 — Remove the manual approval pause from the release workflow

Status: OPEN.

Milestone: 5

## Outcome

The release workflow requires no environment reviewer and checks for none. A tag push runs qualification, publication, installs and promotion without a manual pause, and a rehearsal dispatch runs every job with each publish and channel authentication step a no-op.

- Why: Ian's ruling of 2026-11-01 (`decisions/2026-11-01-no-manual-release-pause.md`). The step "Require protected operator approval for a production release" (`.github/workflows/release.yml:70-76`) runs `release/workflow.py protection` (`release/workflow.py:48-56`), which fails unless `release-publication` has required reviewers, self-review prevention and no admin bypass. The private rehearsal repository cannot carry required reviewers on our plan, and Ian wants no pause in real releases either.
- Starts from: `.github/workflows/release.yml`, `release/workflow.py`, `release/test_workflow.py` and `docs/development.md:57-70`.
- Rules: delete the protection step, the `protection` command and its tests. Keep `environment: release-publication` on the publish and promotion jobs for Trusted Publishing and credentials. Rename the `approval_rehearsal` input to `rehearsal` and its summary lines so they no longer speak of approval. Every publication and channel authentication step still runs only on a tag push to the public repository. Keep every automated gate and the promotion rule. The docs say the release environment needs no reviewers and that Ian's approval governs pushing the tag.
- Proof: workflow tests show no protection command remains and that a rehearsal dispatch reaches every job with publication and authentication skipped; the existing release tests pass; `rg -i "required reviewer|prevent_self_review|approval_rehearsal"` finds nothing outside `sdlc/`.
- Changes: the release workflow, its helper and tests, and the development docs.
- Defers: none.
