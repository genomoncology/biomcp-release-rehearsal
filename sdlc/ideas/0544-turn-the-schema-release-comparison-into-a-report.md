# Turn the schema release comparison into a report that never blocks (release review C6)

Status: closed.
Declined: The check already passes any break that bumps the schema ID's major version (check_reviewed in compatibility.rs), which Ian's stance keeps: versions stay so readers can refuse; weighed 2026-11-01.

Kind: idea
When: If Ian rules that schema IDs need not keep versions

## Problem

Turn the schema release comparison into a report that never blocks (release review C6). Breaking changes land without a gate.
