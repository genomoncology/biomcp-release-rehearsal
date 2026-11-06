# 0666 — Run one hosted release section against an explicit main commit

Status: COMPLETE.
Landed: 3b96575f9c16396681d2d31356e87170a4d2c457

Milestone: 5

## Outcome

The existing manual single-platform rehearsal can qualify an explicit immutable main commit before a candidate cut, building its product, case runners and fixtures from that same commit and recording the driver workflow's identity separately.

## Why

0640 needs independent real-host sections on main, but the existing recovery workflow and section helper are fixed to the superseded candidate.

## Proof

- Proof: focused workflow/helper tests reject malformed or mismatched revisions before building, bind product/runners/fixtures to the exact requested commit and tree, preserve one selected platform and publishing-off behavior, and retain completed producer evidence on an installed-gate failure. The corresponding real-host sections pass on the selected main commit under 0640 before the cut.

## Starts from

- Starts from: `.github/workflows/recover-section.yml`, `release/section.py` and their focused tests. Both fix the product revision to `1bb97e64f41634b19efc42b937b195e55a20ece4`; the helper also carries historical original-run and producer-reuse labels. The current gate builds runners from the driver tooling checkout, which fails provenance and schema expectations when that source differs from the product.

## Keeps

- Keeps: the existing workflow and launchers, manual rehearsal-only dispatch, approved rehearsal repository and non-release refs, pinned tools and permissions, no publishing or channel authentication, and completed-producer retention. The caller captures the selected full main commit from the canonical repository, and the workflow checks out and verifies that exact commit and tree. No fabricated build identity or weakened product assertions.

## Defers

- Defers: section order and the all-main preflight record belong to 0640; no new framework or workflow is needed.

## Changes

- Changes: `.github/workflows/recover-section.yml`, `release/section.py`, `release/test_section.py`, `release/test_workflow_contracts.py`; explicit selected-main inputs, same-source qualification and truthful provenance.
