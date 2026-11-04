# Replace the url crate with own parsing (release review dependency table)

Status: closed.
Declined: Own parsing would sit on the public-address security boundary; 0532 pins idna_adapter to drop the ICU crates with no own code; weighed 2026-11-01.

Kind: idea
When: If pinning idna_adapter stops removing the ICU crates

## Problem

Replace the url crate with own parsing (release review dependency table). About 20 fewer crates.
