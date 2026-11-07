# 0739 — Article rows keep titles on failed lookups

Status: COMPLETE.
Landed: 15ec46fcf3a8bb8c25700284fdb5bb9aec205886

Milestone: 5

## Outcome

When a drug name lookup fails, or storing a pool fails, `search article` rows still carry their PubMed titles.

## Why

Found while stacking 0717 and 0691. PubMed rows are built with `header: None`, and `page::fetched` never fills them in, so a name-source outage returns rows with no title. The pool-store failure path has the same gap. Status stays truthful, so it does not block the hosting batch. Read from the code, not yet reproduced.

## Starts from

- Starts from: `crates/biomcp-query/src/article/page.rs`, `crates/biomcp-query/src/article/mod.rs`.

## Keeps

- Keeps: failed lookups stated in the reply, and no pool stored for them.

## Changes

- Changes: `crates/biomcp-query/src/article/`, `tests/cases/article/`.

## Proof

- Proof: a case with a failed name lookup shows titles through the CLI.

## Defers

- Defers: nothing.

## Depends on

- Depends on: 0717, 0691.
