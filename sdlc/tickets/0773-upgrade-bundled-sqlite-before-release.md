# 0773 — Upgrade bundled SQLite before release

Status: COMPLETE.
Landed: f35affe8ef0a899c117438e10c579899278dcd37

Milestone: 5

## Outcome

Bundled SQLite no longer fails RUSTSEC-2026-0459, while cache and store behavior remain unchanged.

## Why

The combined landing gate j36068 stopped on vulnerable libsqlite3-sys 0.38.2 through rusqlite 0.40.2.

## Proof

- Proof: cargo deny passes without an advisory exception, and focused cache, bundle, store and concurrent-access checks pass.

## Starts from

- Starts from: workspace rusqlite dependency in Cargo.toml, Cargo.lock, and gate j36068.

## Keeps

- Keeps: bundled SQLite, cache and store formats and behavior, and the advisory gate without an allowlist.

## Defers

- Defers: nothing.

## Changes

- Changes: workspace rusqlite constraint, Cargo.lock, and only required API compatibility fixes.
