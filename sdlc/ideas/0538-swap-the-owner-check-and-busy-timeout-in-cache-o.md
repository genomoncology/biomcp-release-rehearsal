# Swap the owner check and busy timeout in Cache::open_with_owners (release review A2)

Status: closed.
Declined: Claim wrong: rusqlite 0.40.2 sets a 5 s busy timeout on every open and check_owners is a read-only deferred transaction, so it already waits; weighed 2026-11-01.

Kind: idea
When: If a first-open SQLITE_BUSY failure is ever observed

## Problem

Swap the owner check and busy timeout in Cache::open_with_owners (release review A2). Concurrent first opens survive schema creation.
