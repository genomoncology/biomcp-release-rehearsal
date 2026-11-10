# Define FDA CDx row-date semantics before adding global filtering and sorting

Status: open.

Kind: idea
When: Before implementing the 1.1 diagnostic recent-arrivals feature

## Problem

Define FDA CDx row-date semantics before adding global filtering and sorting. Current table retains physical native order and dates embedded in intact device cells; persona reports only234/244 rows parseable. Inventory both schemas for multiple/missing/malformed/device-associated dates, define an honest row-date rule without inventing approval semantics. Then explicit whole-snapshot global sort/filter, original cells/identities, unparseable disclosure, source-index ties and cross-page/boundary/snapshot tests. Never sort individual returned pages.
