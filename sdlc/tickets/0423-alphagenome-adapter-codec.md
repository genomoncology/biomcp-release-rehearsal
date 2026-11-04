# 0423 — Move the AlphaGenome codec into its adapter

Status: COMPLETE.
Landed: 6748934a48dc262030b1e0ed293a125ff61cfc8a

Milestone: 5

Depends on: 0429

## Outcome

The AlphaGenome request encoder, reply envelope, score limits, tensor message decoding and generated protobuf messages live in `crates/biomcp-adapters/src/alphagenome/`, behind `score(leg, request)`. The transport keeps the gRPC worker, deadlines, recording, cache admission, `Limits`, the stream decoder trait and `grpc::decompress`, which owns ZSTD. The query layer stops importing `biomcp_fetch::grpc::alphagenome`. A ratchet stops source names returning to the transport.

- Why: the transport names a provider and holds its schema, which breaks the layer rule in architecture 2.1.
- Starts from: `crates/biomcp-fetch/src/grpc/alphagenome/`, `grpc/generated/`, `grpc/proto.rs`, `grpc/tensor.rs`, `crates/biomcp-adapters/src/alphagenome.rs`, `crates/biomcp-query/src/variant/alphagenome.rs`, `alphagenome_join.rs`, `alphagenome_page.rs`; fetch tests `alphagenome.rs`, `alphagenome/routing.rs`, `score_variant.rs`, `tensors.rs`, `grpc_persistence.rs`; runner `alphagenome_tls.rs`; `crates/biomcp/src/config/deferred_keys_tests.rs`; architecture 10.4's ratchet list.
- Rules: `biomcp-adapters` adds `prost` with its derive feature (PM ruling 2026-10-28) and no other dependency. ZSTD stays in the transport. Codec tests move with the codec to `crates/biomcp-adapters/tests/`. The generated code's dead-code exception moves with it. `make tensor-proto-check` reads the new path. The new ratchet fails when a `Source` spelling appears as a word in `crates/biomcp-fetch/src`, with an allow list of `secret.rs`, `cache_hgnc.rs`, `bundle/transfer.rs` and `cache/schema.rs` that 0422 empties.
- Proof: the reply diff against the base is empty; the installed AlphaGenome trial from 0429 passes on the installed artifact; a planted `alphagenome` identifier in a transport file fails the ratchet; the dependency ratchet refuses `prost` in `biomcp-query`.
- Changes: the files above, `crates/biomcp-adapters/Cargo.toml`, `crates/biomcp-fetch/src/grpc/mod.rs`, `crates/biomcp/tests/ratchets/`, `Makefile`.
- Defers: HGNC in the transport and `SecretName::source()` (0422).
