# biodata is the base data layer

Decided 2026-10-23 by Ian. Updated 2026-10-31 to describe the approved separation of neutral values and consumer contracts.

The `biodata` crate holds neutral entity values, identifiers, coordinates, notation and vocabularies. It has no command, imports no workspace crate and performs no I/O. Adapters map publisher values into these checked values. It never fetches, caches or ranks.

`biomcp-types` holds BioMCP requests, replies, source identities and their schemas. It imports `biodata` without a reverse dependency. These consumer contracts no longer belong to the neutral value crate.

`crates/biomcp/tests/ratchets/dependencies.rs` enforces the dependency direction. `crates/biodata/clippy.toml` enforces I/O and mutable-state restrictions. `crates/biomcp-types/tests/schemas/ownership.rs` checks schema ownership. The architecture describes the layers; source declarations and schema registries own their inventories.
