# 0804 — Clarify which runner owns routine command trials

Status: COMPLETE.
Landed: 5d1d1355e54c2daf52126ef9ec69015e75a70a8e

Milestone: 5

## Outcome

Existing test guidance and selector hints clearly direct YAML cases and routine Rust CLI/MCP trials, including protocol and pool tests, to the cases runner. They reserve tools --proofs for its separate tooling-proof catalog and direct users to copy names from the selected runner's --list.

## Why

Wrong-catalog setup failures repeated during qualification and source-pool regression work. Read-only root-cause review found that scripts/trials.py and native hints ambiguously direct custom trials to tools --proofs, although routine Rust trials are registered in case_main.rs. Existing zero-selection guards already reject these mistakes correctly.

## Proof

- Inspect the updated help/hints against actual cases/tooling registrations and run existing focused helper checks appropriate to the changed wording. Preserve zero-selection rejection and both catalogs. No new product build or mutation campaign is required for this wording change.

## Starts from

- scripts/trials.py, crates/biomcp/tests/runner/case_main.rs and existing native selector-error hints. Existing retained-artifact --runner/--executable mode checks the selected catalog before preparing the product.

## Keeps

- Existing catalogs, CLI selectors, validation, execution, zero-selection errors and runtime/schema behavior. No new selector, framework, guard or dependency. Preserve earlier setup failures as setup failures.

## Defers

- Product qualification stays in 0788 and new source-pool regression work stays in 0803. This ticket changes guidance only and can share the routine landing gate with the regression ticket.

## Changes

- Existing trials helper guidance, matching native selector hints and directly affected existing guidance assertions if any.
