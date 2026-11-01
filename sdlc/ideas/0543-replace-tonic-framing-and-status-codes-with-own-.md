# Replace tonic framing and status codes with own code (release review dependency table)

Status: closed.
Declined: 0532's feature trim removes the unused channel stack; tonic is chosen and used, and own gRPC framing adds risk for little gain; weighed 2026-11-01.

Kind: idea
When: If the feature trim in 0532 leaves tonic's tree large

## Problem

Replace tonic framing and status codes with own code (release review dependency table). Up to 16 fewer crates.
