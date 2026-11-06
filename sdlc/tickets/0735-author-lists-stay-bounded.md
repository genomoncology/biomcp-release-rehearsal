# 0735 — Author lists stay bounded

Status: OPEN.

Milestone: 5

## Outcome

Author search rows stay small when a paper has thousands of authors. A row shows a bounded list of names, says how many more there are, and tells the caller how to read the rest.

## Why

Review of 0722: nothing caps names per paper. One 2,900-author paper adds about 37 KB, so `--limit 3` gave 75 KB and `--limit 25` hit the deadline. The 0719 trial intervention rows set the pattern.

## Starts from

- Starts from: `crates/biomcp-query/src/author/literature.rs`.

## Keeps

- Keeps: the row order and the matched author always shown.

## Changes

- Changes: `crates/biomcp-query/src/author/`, `tests/cases/author/`.

## Proof

- Proof: a case with a very long author list shows the bound and the count, in JSON and Markdown, through the CLI.

## Defers

- Defers: nothing.

## Depends on

- Depends on: 0722.
