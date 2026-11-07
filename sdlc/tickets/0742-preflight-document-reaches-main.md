# 0742 — Preflight document reaches main

Status: COMPLETE.
Landed: e6b7a2dc37a1477bcfe77858c9dfc5d5e60abf4d

Milestone: 5

## Outcome

The reviewed release section document is on main, with rows for the build path and container changes, so the 0640 preflight can run from it.

## Why

`pm ticket land` closes a ticket, and 0640 stays open until its preflight has run. This ticket carries only the document. After it lands, record `pm ticket progress 0640 --landed SHA`.

## Starts from

- Starts from: branch `ticket/0640-main-section-preflight` at c47a070d0, merged with current main.

## Keeps

- Keeps: every section command and pass rule the 0640 review settled.

## Changes

- Changes: `release/qualification.md` and its docs tests.

## Proof

- Proof: `check_docs.py` and `release.test_docs` pass, with no pipe in the job.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.
