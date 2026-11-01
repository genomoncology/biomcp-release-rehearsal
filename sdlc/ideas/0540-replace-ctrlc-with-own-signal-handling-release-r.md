# Replace ctrlc with own signal handling (release review dependency table)

Status: closed.
Declined: Replacing it needs unsafe code, which the workspace denies, and a Windows handler; ctrlc is chosen and used under the dependency ruling; weighed 2026-11-01.

Kind: idea
When: If ctrlc becomes unmaintained

## Problem

Replace ctrlc with own signal handling (release review dependency table). Three fewer crates.
