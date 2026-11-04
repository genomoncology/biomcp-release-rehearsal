# One transport

Decided 2026-10-23 by Ian, as one of the ratchets enforced by lint. Details applied by the PM. Updated 2026-10-31 to describe the approved transport and streaming implementations.

`biomcp-fetch` owns network clients, deadlines, quotas, retry, body limits, checked reply caching, recording, replay and bundle transfer. Ordinary HTTP uses blocking `ureq` with rustls and platform trust. The bounded streaming RPC worker uses its async dependencies inside the same crate. Adapters own provider declarations and codecs. They construct no network clients and perform no sleeps or environment reads.

`Fetch::leg` binds declared source policy and a deadline. `Leg::call` accepts an endpoint, `Input` and `Read`, and returns `Received<T>` with checked data and reply metadata. Response readers carry request-bound checks and cache-admission policy. Probe, batch, bundle and streaming operations keep specialized entry points within the transport.

Quota keys come from source declarations rather than URLs, hosts or endpoints. A required-key source sends no request without its credential. The binary configuration boundary supplies proxy settings and additive certificate roots; it reads supported proxy environment settings only when no explicit proxy is set. Replay serves recordings and reports a missing exchange. Offline mode performs no upstream request.

`crates/biomcp/tests/ratchets/dependencies.rs` enforces client ownership. Per-crate Clippy rules enforce adapter I/O restrictions. `crates/biomcp-fetch/tests/transport.rs`, `post.rs`, `quotas.rs`, `offline.rs` and `record.rs` exercise ordinary transport behavior. `crates/biomcp/tests/runner/cache_admission.rs` checks request-bound reuse. `crates/biomcp-fetch/tests/grpc_worker/shutdown.rs` and `grpc_persistence.rs` check streaming ownership and persistence. The architecture names each implemented rule's checks.
