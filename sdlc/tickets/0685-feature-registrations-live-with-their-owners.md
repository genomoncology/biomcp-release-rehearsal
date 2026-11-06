# 0685 — Feature registrations live with their owning modules

Status: COMPLETE.
Landed: 6688f220eb64e9979d84c4f1f25d796c06cf7768

Milestone: 5

## Outcome

The protocol test registry holds only protocol-local checks. Typed-tool and held-proxy feature registrations move into the existing modules that own those features, each through its own `trials()` function, as hosted registration did. Two independent additions no longer push one shared function past its size limit after they merge.

## Why

Two native gates failed the same way (j30799 and j31514; about 500 seconds wasted, no product defect; coordinator RCA m27760). Feature names and closures sit in one shared, size-limited registry, so independent additions overflow it only after merge. Moving only the tail out earlier left the cause.

## Proof

- Proof: the moved trials keep their names, order, closures and target exclusions. The list of registered trials from `cases` and `tools` is identical before and after, byte for byte. The structural ratchets pass with no limit change. Routine gate.

## Starts from

- Starts from: the protocol trials registry in `crates/biomcp/tests/runner/`, the current lander's move of hosted registration into `hosted_surface::trials()`, and the per-module `trials()` pattern already used across the runner.

## Keeps

- Keeps: every trial and its behavior. No framework, new check or size-limit change.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp/tests/runner/`; test code only.
