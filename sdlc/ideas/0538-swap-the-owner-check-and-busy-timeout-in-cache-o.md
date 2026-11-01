# Swap the owner check and busy timeout in Cache::open_with_owners (release review A2)

Status: open.

Kind: idea
When: If a first-open SQLITE_BUSY failure is ever observed

## Problem

Swap the owner check and busy timeout in Cache::open_with_owners (release review A2). Concurrent first opens survive schema creation.
