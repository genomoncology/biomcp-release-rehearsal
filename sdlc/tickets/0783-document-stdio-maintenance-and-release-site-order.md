# 0783 — document-stdio-maintenance-and-release-site-order

Status: COMPLETE.
Landed: c8c74f37145d50f73c609e48bdcc7aa7342fe875

Milestone: 5

## Outcome

The README states that ordinary stdio MCP permits explicit synchronization and store cleaning. The docs-site ticket has one consistent instruction: refresh before the cut, then land immediately after successful publication/public-main advancement in the same release session.

## Why

Ian's November 9 review found an omitted stdio capability and contradictory site timing that could lead an operator to publish stale docs.

## Proof

- Compare the wording with the actual transport policy and the November 6 ruling; run the existing documentation check. No new behavior test is needed for prose-only changes.

## Starts from

- README MCP section, `crates/biomcp/src/command/mcp_policy.rs`, the existing MCP boundary tests, ticket 0401 and the prepared release order. The PM has reconciled 0401 in this branch; complete the README sentence and review both for consistency.

## Keeps

- HTTP restrictions, bounded explicit maintenance, the installer copy/check, site squashing, 0401's publication dependency and Ian's approval of site publication. Do not change runtime policy or imply the site must expose unavailable 1.0 binaries before the tag succeeds.

## Defers

- The actual site refresh/binding and landing remain in 0401 after the final runtime fixes select a new source.

## Changes

- README and the reconciled 0401 ticket; PM release ordering and post-release milestone metadata accompany the batch.
