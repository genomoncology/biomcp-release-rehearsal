# 0743 — Silent error drops stay visible

Status: COMPLETE.
Landed: f636524bbb3b3ef6b3fbd5d2875fec06d143d0a5

Milestone: 5

## Outcome

A failed optional value written as `match` with `Err(_) => None` is caught by the same ratchet as `.ok()`, and the existing sites either report a problem in the reply or carry a stated reason.

## Why

The 0733 review found the optional-value ratchet misses this shape, with seven other sites in the code. A dropped error can make a partial answer read as complete.

## Starts from

- Starts from: the optional-value ratchet and the seven sites it misses.

## Keeps

- Keeps: replies that are correct today, and the ratchet's existing list.

## Changes

- Changes: the ratchet, the sites it names, and their cases.

## Proof

- Proof: the ratchet refuses a new site in a test tree, and each changed site has a case showing its problem or reason.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.
