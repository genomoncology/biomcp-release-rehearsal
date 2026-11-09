# Retry the owned cache WAL transition within its shared budget

Status: COMPLETE.
Landed: 9e6b8d5ae84089b4249884cc445dc2c728d3e36d

Milestone: 5

## Outcome

A transient SQLite Busy during the WAL transition is retried inside the existing shared cache-open deadline, and a lock held beyond that deadline returns Busy without changing foreign-owned files.

## Why

Installed cold check j36136 reproduced the journal-mode failure previously fixed by 0639: one of eight first opens returned Busy before its budget expired.

## Starts from

- Starts from: `crates/biomcp-fetch/src/cache.rs`, its existing contention tests, the installed first-open helper, and `records/2026-11-05-cache-first-open-busy.md`. The 0748 sidecar-lock change removed 0639's explicit WAL retry; SQLite can still decline its own busy handler.

## Keeps

- Keeps: ownership validation before mutation, one shared five-second open deadline, cache schemas/namespaces, BioData pin, and immediate propagation of non-Busy errors. Never retry initialization or ownership transactions as a whole.

## Proof

- Proof: a real held SQLite writer causes one WAL attempt to return Busy; after that observed Busy releases the writer, the same open deadline allows WAL to succeed. A held writer cannot reset or outlive the deadline. Existing cache ownership/process tests and the installed eight-process first-open check pass.

## Changes

- Changes: WAL retry boundary and its existing private contention tests; one bounded root-cause record.

## Evidence

- Evidence: `records/2026-11-09-cache-wal-retry-regression.md` records the repeated cause and the retained installed witness.

## Defers

- Defers: nothing.
