# Document where the cache and store may live

Status: OPEN.
Milestone: 5

## Outcome

The public docs say that the cache and store folders belong on a local disk or, in Docker, on a named volume, and that several processes sharing one cache through a Docker Desktop host bind mount or a network filesystem is unsupported, because SQLite's write-ahead log needs shared memory that those mounts do not provide.

- Why: 0569 measured lost cache writes with SQLite WAL on a macOS Docker Desktop host bind mount: 0 of 12 loaded cases kept both rows while every write reported success, against 12 of 12 on container-native storage with the same binary, and 12 of 12 on the same bind mount with the rollback journal (Yellow `biomcp-redux-evidence/0569-cache-process-deadline-under-load`). The code always selects WAL (`crates/biomcp-fetch/src/cache.rs:118-122`, `crates/biomcp-query/src/store/mod.rs:103`), and `cache/contention.rs:111-151` requires WAL's reader concurrency, so the product keeps WAL. A lost cache row costs a refetch, not a wrong answer, so documentation is the proportionate fix. No doc says this today.
- Starts from: `docs/cautions.md`, the configuration and Docker sections of the public docs, SQLite's documented WAL limits.
- Rules: plain words, one short passage where users choose a cache folder and one in the Docker instructions. Do not add a runtime filesystem check or switch journal mode.
- Proof: the docs check passes; a reviewer finds the passage from the Docker instructions and the configuration reference.
- Changes: public docs only.
- Defers: any runtime detection or a mounted-cache mode, which would need its own witness and decision after 1.0.
