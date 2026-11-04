# Crates enforce dependency direction

Decided 2026-10-23 by the PM, applying Ian's layered design and his ratchet ruling. Updated 2026-10-31 to describe the implemented boundaries after the approved restructure.

`crates/biomcp/tests/ratchets/dependencies.rs` owns the allowed dependency table. Cargo manifests declare actual dependencies. `sdlc/planning/architecture.md` describes each layer without copying that table.

- `biodata` holds neutral values and has no workspace dependency or I/O. Its `clippy.toml` checks I/O restrictions.
- `biomcp-types` holds consumer requests, replies, sources and schemas above `biodata`.
- Adapters import fetch and value types. They cannot import query services, the binary, HTTP clients or SQLite.
- Query services own entity behavior, typed next actions, status and the store. The source ratchet confines their SQLite access to `src/store/`.
- The binary owns grammar, settings, MCP and rendering. Its dependency list imports query services rather than adapters. Rendering consumes serialized response values.
- The binary reads configuration and the environment at reviewed settings boundaries. Per-crate Clippy rules and the source ratchet check those exceptions.

Cargo, per-crate Clippy rules and `crates/biomcp/tests/ratchets/dependencies.rs` enforce these boundaries. `crates/biomcp/tests/ratchets/source.rs` checks placement and reasoned exceptions. The allowed table includes the fetch crate's bounded streaming runtime dependencies.
