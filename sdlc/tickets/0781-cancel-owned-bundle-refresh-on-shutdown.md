# 0781 — cancel-owned-bundle-refresh-on-shutdown

Status: OPEN.

Milestone: 5

## Outcome

Dropping the server-owned bundle refresher cancels its in-flight download and joins its workers promptly. It does not wait out the ordinary 120-second sync budget. Ending an individual caller still does not cancel a refresh owned by the server.

## Why

The current refresh clone clears cancellation and its owner joins workers on shutdown, making a normal server exit appear hung during a download.

## Proof

- Hold a real loopback bundle response after the background refresh starts, drop the public refresher owner, and prove cancellation interrupts the request and workers are joined before the response is released. Use events rather than a short timing-only assertion. Demonstrate the regression before the fix.
- Preserve existing stale-serving/single-flight behavior and the distinction between caller cancellation and owner shutdown.

## Starts from

- `crates/biomcp-fetch/src/bundle/refresh.rs`, `BundleRefresh`, existing `Cancellation::cancel` socket shutdown and public Fetch bundle integration fixtures. Both MCP transports opt into the same refresh owner.

## Keeps

- Server ownership, ordinary quotas, store/publisher lock, atomic publication, shared deadlines, independent caller lifetimes and error reporting. Signal cancellation and shut down blocking owner connections before joining; a flag alone must not leave a blocked wire read running.

## Defers

- No broad shutdown refactor or new worker framework.

## Changes

- Fetch background refresh ownership/cancellation and its owning public API regression fixture.
