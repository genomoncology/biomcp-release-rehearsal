# 0671 — Gene name replay declares its live-check eligibility

Status: OPEN.

Milestone: 5

## Outcome

The gene full-name replay declares its intended live-check eligibility using the existing case format. Installed `--live-check --list` succeeds with the complete case inventory, while this synthetic replay remains an offline test.

## Why

Hosted selected-main run 42095109380 built its package successfully, then failed `live-source-checks/offline-eligibility` because `tests/cases/gene/names-single.yaml`, added by the gene-name change, was the only case without a `live_check` declaration. The installed tool proof reported 35 passes, one failure and one intentional ignore. Raw proof is on Yellow at `data/biomcp-redux-evidence/0666/hosted-42095109380`.

## Proof

- Proof: reproduce the failing installed `--live-check --list`, then pass it and the existing gene-name replay after declaring eligibility. Run the routine gate. Another hosted supplier run follows from landed main; it is not a condition for landing this fixture fix.

## Starts from

- Starts from: `tests/cases/gene/names-single.yaml`; the existing declaration format in adjacent replay cases; `crates/biomcp/tests/runner/live.rs` and the installed offline-eligibility proof. Selected main was `dea3f1389c15ae602de59bcaeb6176144e9a695f`; the producer and runners were correctly bound to that revision.

## Keeps

- Keeps: the strict inventory completeness check, synthetic gene fixtures, gene-search behavior, hosted revision selection and provider admission. No live provider request is needed.

## Defers

- Defers: no other behavior belongs to this fix. Final installed platform qualification remains with the existing qualification tickets.

## Changes

- Changes: `tests/cases/gene/names-single.yaml` and an existing focused test recipe only if needed.
