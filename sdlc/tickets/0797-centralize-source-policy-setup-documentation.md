# 0797 — Centralize source policy setup documentation

Status: COMPLETE.
Landed: 62fb202e781d0355b9210e63ce377071d0fb2d64

Milestone: 5

## Outcome

Configuration documentation is the complete home for local source exclusions and hosted policy, while the README keeps a prominent first-use action and link without duplicating the full policy.

## Why

The recorded single-policy stance conflicts with the current README/configuration split; deleting the README block alone would lose the only local commercial-exclusion example. Evidence: `records/2026-11-10-product-review.md`, F7.

## Starts from

- `README.md`, `docs/configuration.md`, `sdlc/decisions/2026-11-06-review-stances.md` and the prepared docs-site branch. `scripts/check_docs.py` currently parses README's first JSON block as the commercial setup and compares its replay fixtures; move that existing invariant to the canonical local-policy example using a stable anchor (configuration has other JSON blocks). `scripts/test_docs.py` owns the checker regression. Compare actual configuration/source exclusion code; current prose is not shown to be false.

## Proof

- One new-user walkthrough reaches the commercial exclusion JSON, local configuration location, combining CLI/config exclusions and the distinct hosted defaults from the README link. `make docs-check` verifies the documented examples and existing invariants, not general Markdown links; the walkthrough checks the actual policy link. Altering the canonical exclusions example must still make the existing checker fail. Run existing site checks if site inputs change. No new product runtime test is needed.

## Keeps

- Current open-by-default source policy and publisher terms, conspicuous commercial setup guidance, usable first-screen instructions, and the prepared site's release-session landing order.

## Changes

- Changes: `README.md`, `docs/configuration.md`, `scripts/check_docs.py`, `scripts/test_docs.py`.

## Defers

- No source-policy change. The docs-site ticket integrates the accepted text into its prepared branch.
