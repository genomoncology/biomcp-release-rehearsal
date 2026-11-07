# 0745 — One Python version for tools

Status: COMPLETE.
Landed: f636524bbb3b3ef6b3fbd5d2875fec06d143d0a5

Milestone: 5

## Outcome

Release tooling and development use one Python version, 3.13, and `scripts/check_front_map.py` fails with a stated reason rather than a bare assert.

## Why

`release/toolchain.json` says 3.12 while `sdlc/pm.json` and 36 files say 3.13. The split has cost three failed jobs. The 0694 ruling left this out of that ticket.

## Starts from

- Starts from: `release/toolchain.json`, `sdlc/pm.json`, and the 0694 amendment that named 3.13.

## Keeps

- Keeps: the product, and the pinned release toolchain's other versions.

## Changes

- Changes: `release/toolchain.json`, the files that name a Python version, and `scripts/check_front_map.py`.

## Proof

- Proof: `release.test_docs` and the toolchain tests pass, one search shows a single version, and a broken front map prints its reason.

## Defers

- Defers: bare `python3` in the Makefile stays for 1.0, because the ticket gate pins Python 3.13 first on PATH.

## Depends on

- Depends on: nothing.
