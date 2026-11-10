# 0792 — review-remaining-maintainability-usability-and-accuracy

Status: COMPLETE.
Landed: 4e9dc1131b2610ce36404db7506bc2cbc78fe9f8

Milestone: 5

## Outcome

A pinned review of core and hosting yields an independently weighed findings register, an executable test playbook and ready fix tickets, ordered by harm and implementation overlap.

## Why

Ian asked on November 10 to prioritize finding remaining maintainability, usability and accuracy problems over release timing or AWS publication; see `decisions/2026-11-10-prioritize-remaining-product-issues.md`.

## Proof

- Every finding names its source snapshot, evidence, uncertainty, smallest fix and owning ticket or explicit disposition. Tested behavior includes exact commands, artifacts and results; unrun cases remain marked unrun. A fresh weigher checks findings before the PM accepts them. The playbook gives another agent the inputs and assertions needed to execute each remaining check.

## Starts from

- Core `a84ee9721f7fb486755341286ae5512232ba9799`, hosting `a14558a99e408c1d89ac134cde75ecc9e1350063`, the current source/transport/store/MCP boundaries, existing review decisions and open tickets. The successful replacement rehearsal and failed nightly live-source run remain evidence for their original sources.

## Keeps

- Read-only product review, isolated public-keyless probes, source licensing and credential containment. No deployment, account modification or publication. No repeated full qualification merely for review. Review records distinguish proven behavior from hypotheses.

## Defers

- Implementations belong to the resulting tickets. Existing final qualification, hosted validation and publication tasks resume after accepted fixes settle; their earlier evidence is preserved.

## Changes

- Changes: `sdlc/records/2026-11-10-product-review.md`, `sdlc/planning/remaining-issues-playbook.md`, `sdlc/plan.md`, `sdlc/planning/build-plan.md`, `sdlc/decisions/2026-11-10-prioritize-remaining-product-issues.md`, `sdlc/tickets/`.

## Evidence

- [Findings and source evidence](../records/2026-11-10-product-review.md), [test playbook](../planning/remaining-issues-playbook.md), and [implementation order](../planning/build-plan.md). Fresh source weighing and combined ticket design review were completed; corrections cover legacy pools, concurrency, configuration readers, failure diagnostics and the docs checker.
- Core fix tickets 0793–0798, existing cache tickets 0785–0786, and hosting signing-key ticket 0091 carry the accepted work. The release-order and planning-record corrections are included here.
- M5 artifact probe j36638 establishes seven CLI invocations and sequenced stdio help/error behavior. Original nightly failure j36636 and Yellow setup failure j36637 remain separate. The findings record names the receiver-verified Yellow packet and precise limits.
- Review changes are confined to planning, tickets, decisions and records. PM lint and document file-link checks validate the handoff; no product gate is needed for these inert edits.
