# 0686 — Markdown help lists every flag the installed proof expects

Status: OPEN.

Milestone: 5

## Outcome

`help <command> --format md` on an installed build lists every flag of that command, and the installed proof `schema-discovery/rejects-bad-cases` (the nested `markdown-pair.yaml` case) passes on every native platform. If the proof's expectation is stale rather than the help wrong, the proof is corrected to the shipped help and says why.

## Why

The hosted Mac ARM rehearsal of main 757ace1f8 (0666, run 42139916605) passed 35 installed proofs and failed one: `help author --format md` was expected to show `--format`, and also author papers and `--disable-sources` (`crates/biomcp/tests/runner/proof.rs:155-164`). The assertion did not keep the actual Markdown, so it is not yet known whether the help or the expectation is wrong. The help descriptions changed recently (0653). This must pass on main before the cut (`decisions/2026-11-05-release-finds-no-bugs.md`).

## Proof

- Proof: first capture the actual Markdown from that build (or an identical local build of 757ace1f8) and compare it with the help source. Make the assertion keep the actual text on failure. Fix whichever side is wrong at its owner, with a fresh review, and never weaken the proof to pass. The case passes in the routine gate and on one native platform rerun of the installed proof.

## Starts from

- Starts from: `crates/biomcp/tests/runner/proof.rs:155-164`, the nested `markdown-pair.yaml` case, the help Markdown renderer, and the retained run on Yellow under `0666/hosted-42139916605`.

## Keeps

- Keeps: the proof's strength and JSON help output.

## Defers

- Defers: nothing.

## Changes

- Changes: the help Markdown renderer or the proof case, whichever is wrong, and the assertion's failure output.
