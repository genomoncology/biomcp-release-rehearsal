# Architecture

BioMCP retrieves source records for cancer care and human health research. The command line and MCP server share checked requests, query execution and response envelopes. Adapters preserve publisher facts. BioMCP does not classify variants, select treatments or run bioinformatics tools.

This file describes the implemented boundaries. The source files own declarations, limits and exception lists. Each rule below names its enforcement file with `Check:`. `scripts/check_docs.py` rejects missing check files; it does not establish that a named check proves the prose. Code review establishes that relationship.

## 1. Layers

The dependency direction follows `../decisions/2026-10-23-crates-enforce-dependency-direction.md`. `crates/biomcp/tests/ratchets/dependencies.rs` owns the allowed dependency table. Cargo manifests declare the actual dependencies.

| Rule | Owner | Enforcement |
| --- | --- | --- |
| Neutral entity values have no workspace dependency or I/O. | [BioData](https://github.com/genomoncology/biodata) | Check: the published crate's dependency and lint checks |
| Consumer queries, replies, source identities and schemas sit above neutral values. | `crates/biomcp-types` | Check: `crates/biomcp/tests/ratchets/dependencies.rs`; Check: `crates/biomcp-types/clippy.toml` |
| Transport owns network clients, request caching, recording, replay and bundle transfer. | `crates/biomcp-fetch` | Check: `crates/biomcp/tests/ratchets/dependencies.rs` |
| Adapters declare sources and map native replies without importing query services, the binary, HTTP clients or SQLite. | `crates/biomcp-adapters` | Check: `crates/biomcp/tests/ratchets/dependencies.rs`; Check: `crates/biomcp-adapters/clippy.toml` |
| Query execution owns resolution, fan-out, merge, ranking, pools, pages, sections, status and typed next actions. | `crates/biomcp-query` | Check: `crates/biomcp/tests/ratchets/dependencies.rs` |
| SQLite access in the query crate stays under `src/store/`. | `crates/biomcp-query/src/store` | Check: `crates/biomcp/tests/ratchets/source.rs` |
| The binary owns grammar, configuration, MCP transport, output and the Markdown view; it imports query services rather than adapters. | `crates/biomcp/src` | Check: `crates/biomcp/tests/ratchets/dependencies.rs` |
| Environment reads and mutable-state exceptions stay in reviewed locations. | Per-crate `clippy.toml` and reasoned exceptions | Check: `crates/biomcp/tests/ratchets/lints.rs`; Check: `crates/biomcp/tests/ratchets/source.rs` |

## 2. Commands and schemas

`crates/biomcp-types/src/query/commands.rs` defines `command_types!`. Its rows connect request variants, checked query types, command paths, schema identities, presentation order and explicit rewrites. Request dispatch, grammar discovery and schema registration consume those rows. Reply contracts have their own registry in the response layer.

| Rule | Owner | Enforcement |
| --- | --- | --- |
| Each command path has one grammar or one declared shared rewrite, with a registered schema. | `command_types!` consumers | Check: `crates/biomcp-types/tests/schemas/commands.rs` |
| Generated help and reference text agree with the grammar. | `crates/biomcp/src/grammar/reference.rs` | Check: `crates/biomcp/src/grammar/reference.rs` |
| Case replies validate against their advertised schemas before assertions. | `crates/biomcp/tests/runner/schema.rs` | Check: `crates/biomcp/tests/runner/schema_proof.rs` |
| Schema generation and release comparison require a major-version bump for incompatible transitions. | `scripts/schema_release.py` | Check: `scripts/test_schema_release.py` |

The command reference owns the public command inventory. This file carries no second command or schema list.

## 3. Source declarations

`crates/biomcp-adapters/src/decl.rs` defines `SourceDecl` and an exhaustive `decl(Source)` dispatcher. Each source module owns its declaration and `SourceSpec`. The declaration carries display and discovery order, probe behavior, bundles, carried components, auxiliary policies, guidance and credential registration. The adapter registry derives its views from these declarations.

| Rule | Owner | Enforcement |
| --- | --- | --- |
| Each concrete source policy has one matching declaration and publisher terms. | Adapter source declarations | Check: `crates/biomcp/tests/ratchets/source_declarations.rs`; Check: `crates/biomcp/tests/ratchets/source_declaration_proof.rs` |
| Transport code contains no canonical provider names. | Generic transport policies | Check: `crates/biomcp/tests/ratchets/transport_sources.rs` |
| Bundle command names are unique and retain declaration order. | Adapter registry | Check: `crates/biomcp-adapters/tests/source_declarations.rs` |
| Source outcomes obtain their terms from the registry. | `crates/biomcp-adapters/src/registry.rs` | Check: `crates/biomcp/tests/ratchets/source_files.rs` |
| Disabled source policy applies before requests and cached or stored reuse. | Fetch restrictions and query store reads | Check: `tests/cases/local/source-controls.yaml`; Check: `tests/cases/author/papers-disabled-pool.yaml` |

Terms describe publisher conditions. They do not grant reuse rights. Provider details and numerical limits stay in each source declaration.

## 4. Values and native reply mapping

`biodata` owns neutral checked values in its public repository. BioMCP depends on BioData v1.0.0 at the exact commit in the workspace manifest; the release build vendors that locked source for offline compilation. `biomcp-types` owns BioMCP requests and replies. This separation implements `../decisions/2026-10-23-biodata-is-the-base-data-layer.md`.

`crates/biomcp-adapters/src/raw.rs` represents a malformed optional field with `Field::Invalid`. Mapping helpers in `mapped.rs` preserve the useful record and attach a field problem. `Mapped<T>` carries the value, problems and the independent `Refusal` fact. Native member refusal is different from an optional field omission.

| Rule | Owner | Enforcement |
| --- | --- | --- |
| Malformed optional source values become field problems without discarding unrelated valid fields. | Raw field and mapping helpers | Check: `tests/cases/trial/adapter-values/nci.yaml`; Check: `tests/cases/drug/adapter-values/openfda-label.yaml` |
| Refused native members remain explicit through mapping and stored round trips. | `crates/biomcp-adapters/src/mapped/refusals.rs` | Check: `crates/biomcp-adapters/tests/custom_refusals.rs`; Check: `tests/cases/biomarker/member-values/nci.yaml` |
| New silent optional-value drops and raw serde defaults require the existing audit classification. | Adapter value audit | Check: `crates/biomcp/tests/ratchets/adapter_values.rs` |
| Only reviewed private refusal fields escape serialized output. | Private refusal owner list | Check: `crates/biomcp/tests/ratchets/private_refusal.rs`; Check: `crates/biomcp/tests/ratchets/mapped_refusals.rs` |

The audit retains explicit classified exceptions and ceilings. It does not prove the absence of every possible silent drop. The source and the named behavior cases remain the basis for review.

## 5. Transport calls

`../decisions/2026-10-23-one-transport.md` puts network work in `biomcp-fetch`. Ordinary HTTP requests use blocking `ureq`. The bounded streaming RPC worker uses an async runtime inside the fetch crate. The adapter owns the AlphaGenome message codec.

`Fetch::leg` binds a `SourceSpec` and deadline. `Leg::call(endpoint, input, read)` returns `Received<T>` with the checked value and reply metadata. `Input` carries parameters or a bounded JSON body. `Read` carries the response codec, request-bound checks, body validation, count checks, byte bounds and cache-admission predicate. JSON, XML, text and binary readers use this shared entry point. Probe, batch, bundle and streaming operations retain their specialized owners.

| Rule | Owner | Enforcement |
| --- | --- | --- |
| HTTP parameter and JSON-body calls share preparation, decode and checked reuse. | `crates/biomcp-fetch/src/call.rs`, `request.rs`, `request_body.rs`, `read.rs` | Check: `crates/biomcp-fetch/tests/post.rs`; Check: `crates/biomcp-fetch/tests/transport.rs` |
| Request identity checks run before admission and on replay or cache reuse. | `Read::check` and checked response readers | Check: `crates/biomcp/tests/runner/cache_admission.rs` |
| Quotas use declared source keys, and required-key sources refuse requests without a key. | `SourceSpec`, quota and credential admission | Check: `crates/biomcp-fetch/tests/quotas.rs`; Check: `crates/biomcp-fetch/tests/transport/credentials.rs` |
| Replay fails on a missing exchange and preserves captured reply metadata. | Recording reader | Check: `crates/biomcp-fetch/tests/record.rs`; Check: `crates/biomcp-fetch/tests/record/metadata.rs` |
| Streaming work respects bounded ownership, cancellation and persistence. | `crates/biomcp-fetch/src/grpc` | Check: `crates/biomcp-fetch/tests/grpc_worker/shutdown.rs`; Check: `crates/biomcp-fetch/tests/grpc_persistence.rs` |

## 6. Errors and status

The fetch layer distinguishes setup errors, input errors, source failures, cache errors and recording errors. `RequestError` preserves these categories. The consumer layer owns usage errors and error envelopes. The binary's `exit::Status` owns process exit codes. Product code returns typed errors; display implementations supply stable codes.

`crates/biomcp-query/src/status.rs` owns `Outcome` status. Search classification uses the asked providers and the native refusal flag. A provider with zero rows can answer successfully. Failed, stale or synchronizing requested data can make an otherwise answered result partial. Skipped sources do not establish absence. Get classification combines identity results and requested availability. Optional field problems alone do not establish a refused record. Local failures remain separate from useful source answers.

| Rule | Owner | Enforcement |
| --- | --- | --- |
| Unit error conversions and numeric exit literals do not bypass typed owners. | Error and exit types | Check: `crates/biomcp/tests/ratchets/errors.rs` |
| A failed requested article section keeps the valid card and a partial result. | Shared status owner and article query | Check: `tests/cases/article/sections/failure-cut.yaml`; Check: `tests/cases/article/sections/failure-500.yaml` |
| Native member refusal makes the retained answer partial. | Shared status owner and adapter refusal fact | Check: `tests/cases/phenotype/refused-member.yaml`; Check: `tests/cases/orcid/refused-member.yaml` |
| Disabled source outcomes retain their reason through pool reuse. | Source policy and status mapping | Check: `tests/cases/author/papers-disabled-pool.yaml`; Check: `tests/cases/author/search-disabled-pool.yaml` |

`crates/biomcp-query/src/next.rs` owns typed follow-up actions. The binary spells them as command arguments. CLI and MCP use the same execution path; research wrappers retain the command envelope alongside their compatibility output.

Check: `crates/biomcp/tests/runner/drug_next/native.rs`; Check: `crates/biomcp/tests/runner/research.rs`.

## 7. Interface and state boundaries

Configuration and credentials enter through the binary settings boundary. Fetch views can restrict sources and uses or replace an immutable secret snapshot while sharing underlying transport resources. Hosted caller ownership and source pacing remain transport and store concerns.

| Rule | Owner | Enforcement |
| --- | --- | --- |
| Credential settings defer reads until the command needs them. | Binary configuration | Check: `crates/biomcp/src/config/credential_reads_tests.rs` |
| HTTP and hosted MCP calls enforce their operation restrictions. | MCP command policy | Check: `crates/biomcp/tests/runner/mcp_hosted.rs` |
| Licensed replies and stored continuations retain caller ownership. | Transport cache and query store | Check: `crates/biomcp-fetch/tests/cache/credentials.rs`; Check: `crates/biomcp-query/src/store/privacy_tests.rs` |
| Expired state cleanup preserves owned bundles and supported generations. | Store cleanup | Check: `crates/biomcp/tests/runner/store_clean.rs` |

## 8. Documentation

Generated reference text comes from the parser boundary. Feature names live once in `sdlc/features.md`; YAML cases declare a feature and Rust trial names carry the feature prefix. Documentation prose points to declarations rather than copying command inventories, dependency versions or limits.

| Rule | Owner | Enforcement |
| --- | --- | --- |
| Feature declarations have unique names and one outcome sentence. | `sdlc/features.md` | Check: `scripts/check_docs.py` |
| Discovered trials name listed features, and complete discovery covers every listed feature. | Runner discovery | Check: `crates/biomcp/tests/runner/features.rs` |
| Every architecture section names an existing check file. | This file | Check: `scripts/check_docs.py`; Check: `scripts/test_docs.py` |

The check-file check verifies references, not coverage or runtime results. A reviewer reads every section against its implementation and enforcement files.

## 9. Test strategy

`../decisions/2026-10-29-one-small-outside-in-suite.md` governs the suite. The `cases` target exercises the built command and MCP surfaces through recorded or synthetic source replies. The `tools` target holds opt-in generators, native proofs, parity work and live checks. Required public-surface accounting derives from shipped help and the MCP registry. Its pending exceptions remain explicit in the existing pending list.

| Rule | Owner | Enforcement |
| --- | --- | --- |
| Case fixture access uses the shared launcher boundary. | `crates/biomcp/tests/runner/launch.rs` | Check: `crates/biomcp/tests/ratchets/source_files.rs` |
| Required command, option, section and MCP coverage comes from the shipped surface. | Required coverage ratchet | Check: `crates/biomcp/tests/ratchets/required.rs`; Check: `crates/biomcp/tests/ratchets/required/proof.rs` |
| Tool-mode flags stay in the tools entry point. | `cases` and `tools` entry points | Check: `crates/biomcp/tests/ratchets/source.rs`; Check: `crates/biomcp/tests/ratchets/proof.rs` |
| The scheduled live check runs a short fixed list of real commands without credentials. | `scripts/nightly_live_check.py` | Check: `scripts/test_nightly_live_check.py` |

## 10. Test boundaries

### 10.1 Installed artifacts

The release rehearsal copies fixtures outside the checkout and selects the installed executable through the runner launcher. Library contract tests in the routine gate validate library APIs separately; they do not establish installed-binary behavior.

Check: `release/test_checks.py`; Check: `crates/biomcp/tests/ratchets/source_files.rs`.

### 10.2 Coverage

`scripts/coverage.sh` counts only product-binary profiles and the linked binary object. Generated protobuf code is excluded. Coverage is a report and gates nothing. It does not justify adding duplicate cases.

Check: `scripts/test_coverage.py`.

### 10.3 Mutation testing and reply comparison

`release/mutations.py` plans and collects bounded candidate mutation runs. Its rule-file selection, deadline and outcome categories stay in the release implementation. Survivors remain advisory under the suite decision. `docs/development.md` describes the commands and their limits.

Check: `release/test_mutations.py`.

`scripts/reply_diff.py` stays as an optional transcript diagnostic under `../decisions/2026-10-31-reply-comparison-remains-optional.md`. Capture and comparison cover the recorded executions, not every operation in a trial. Runtime binding support is explicit in `scripts/reply_runtime.py`; unsupported wire, path, clock or pool forms remain differences or tool errors. A zero exit alone does not prove complete capture or full behavior equality. Refactor review accounts for every public value and complete record, including unsupported comparisons and uncaptured operations. Unknown differences remain unresolved.

Check: `scripts/test_reply_diff.py`.

### 10.4 Where unit tests are allowed

The ratchet owns the authoritative file and reason list below and checks this table against it. Every other test of reachable command behavior runs through the runner. Public library contracts remain only where the binary cannot construct the state without a test hook.

The runner tool proof `schema-discovery/capture-retains-replay-owner` drops the actual launcher and command capture separately, observes the owned temporary directory's lifetime and reads retained fixture bytes after deletion. YAML cannot perform those private ownership operations. Its version command only exercises the real capture boundary.

| File | Reason |
| --- | --- |
| `crates/biomcp-fetch/tests/record/data_age.rs` | Check Gregorian calendar arithmetic, including leap years and invalid dates, without adding a public clock control. |
| `crates/biomcp/src/mcp/http/load_benchmark.rs` | Measure explicitly invoked bounded HTTP load against one owned service and Fetch. |
| `crates/biomcp/tests/ratchets/source_declaration_proof.rs` | Plant source declaration omissions, identity mismatches, unnamed provider facts and transport source literals at the structural boundary. |
| `crates/biomcp-adapters/tests/source_declarations.rs` | Check unique declared bundle command names and their stable public discovery order through the adapter registry. |
| `crates/biomcp-fetch/src/hosts_tests.rs` | Check credential-free exact publisher origins at the private admission boundary without a product inventory helper. |
| `crates/biomcp-fetch/tests/transport/tls_builds.rs` | Count TLS configuration builds across hosted calls at the private constructor boundary without exposing test controls. |
| `crates/biomcp-fetch/tests/grpc_worker/shutdown.rs` | Prove cancellation joins active network work at the private worker boundary without exporting shutdown control. |
| `crates/biomcp-fetch/tests/quotas/expiry.rs` | Check idle bucket expiry after scaled refill without sleeping or exposing transport state. |
| `crates/biomcp/src/grammar/reference.rs` | Generate and compare documentation at the private parser boundary without adding a public command. |
| `crates/biomcp/src/mcp/worker_tests.rs` | Test worker panic completion and exact serialized tool result character boundaries that public requests cannot intentionally produce. |
| `crates/biomcp/src/mcp/http/call.rs` | Prove HTTP failure recovery, safe diagnostic fields and hosted worker capacity and shutdown ownership at private boundaries. |
| `crates/biomcp/src/config/credential_reads_tests.rs` | Observe forbidden credential environment reads at the private settings boundary while executing real commands. |
| `crates/biomcp-adapters/tests/deferred_keys_tests.rs` | Exercise deferred credential admission through the public AlphaGenome adapter without reading binary settings. |
| `crates/biomcp-types/tests/author_contracts.rs` | Exercise checked source-qualified author records and standalone native paper and work-group contracts before adapters exist. |
| `crates/biomcp-types/tests/cell_line_contracts.rs` | Exercise checked cell identities, native variation members and provenance, and standalone consumer contracts through the public library API before adapters exist. |
| `crates/biomcp-types/tests/gwas_contracts.rs` | Exercise checked GWAS identities, exact probabilities and standalone consumer contracts through the public library API before adapters exist. |
| `crates/biomcp-types/tests/phenotype_contracts.rs` | Exercise neutral phenotype records and checked standalone source and section contracts before adapters exist. |
| `crates/biomcp-types/tests/pathway_contracts.rs` | Exercise neutral pathway identities and checked standalone consumer contracts through the public library API before adapters exist. |
| `crates/biomcp-types/tests/protein_contracts.rs` | Exercise neutral protein records and checked standalone consumer contracts through the public library API before adapters exist. |
| `crates/biomcp-types/tests/study_contracts.rs` | Exercise exact study identities, neutral source metadata and bounded standalone consumer contracts through the public library API before adapters exist. |
| `crates/biomcp-types/tests/diagnostic_contracts.rs` | Exercise neutral diagnostic records, native identities and checked standalone consumer contracts through the public library API before adapters exist. |
| `crates/biomcp-types/tests/variant_coordinates.rs` | Exercise VCF coordinate parser boundaries and assembly-qualified full-reference validation strings through the public library API. |
| `crates/biomcp-types/tests/variant_values.rs` | Exercise copied common variant values and constructor boundaries through the public library API. |
| `crates/biomcp-types/tests/article_ids.rs` | Exercise checked values and schema contracts through the public library API. |
| `crates/biomcp-types/tests/disease_contracts.rs` | Exercise neutral disease records and standalone consumer contracts through the public library API before adapters exist. |
| `crates/biomcp-types/tests/pgx_contracts.rs` | Exercise checked pharmacogenomics requests and neutral native record contracts before adapters exist. |
| `crates/biomcp-types/tests/adverse_event_contracts.rs` | Exercise neutral report records and checked source-specific adverse event contracts before adapters exist. |
| `crates/biomcp-types/tests/drug_contracts.rs` | Exercise published drug identifier, neutral record and consumer schema contracts through the public library API before adapters exist. |
| `crates/biomcp-types/tests/core.rs` | Exercise checked values and schema contracts through the public library API. |
| `crates/biomcp-types/tests/gene_ids.rs` | Exercise checked values and schema contracts through the public library API. |
| `crates/biomcp-types/tests/gene_model.rs` | Exercise checked values and schema contracts through the public library API. |
| `crates/biomcp-types/tests/rust_api.rs` | Exercise the external Rust API consumer compile/runtime contract. |
| `crates/biomcp-types/tests/biodata_switch.rs` | Exercise reviewed shared identifier identity, selector wire text and refusal codes through the consumer API. |
| `crates/biomcp-types/tests/query.rs` | Exercise checked values and schema contracts through the public library API. |
| `crates/biomcp-types/tests/response.rs` | Exercise checked values and schema contracts through the public library API. |
| `crates/biomcp-types/tests/schemas.rs` | Exercise checked values and schema contracts through the public library API. |
| `crates/biomcp-types/tests/schemas/disease.rs` | Exercise standalone disease option metadata and synthetic envelope schema contracts before command registration. |
| `crates/biomcp-types/tests/schemas/compatibility_cases.rs` | Exercise checked values and schema contracts through the public library API. |
| `crates/biomcp-types/tests/schemas/policy.rs` | Test reviewed schema pattern changes at exact paths and reject near misses. |
| `crates/biomcp-types/tests/schemas/compatibility_recursive_cases.rs` | Exercise recursive public schema graphs and their compatibility constraints. |
| `crates/biomcp-types/tests/schemas/temporal.rs` | Exercise native timestamp schema spelling and temporal property consistency. |
| `crates/biomcp-types/tests/schemas/drug.rs` | Exercise public drug header and card disclosure boundaries through their serialized values and schemas. |
| `crates/biomcp-types/tests/schemas/registry.rs` | Register public value and research query schema contracts for schema drift checks. |
| `crates/biomcp-types/tests/schemas/ownership.rs` | Check published schema namespaces against the crate that defines each Rust type. |
| `crates/biomcp-types/tests/schemas/values.rs` | Exercise checked values and schema contracts through the public library API. |
| `crates/biomcp-types/tests/value_conventions/sweep.rs` | Exercise public text value construction, parsing, JSON and hashing through the Rust consumer boundary. |
| `crates/biomcp-types/tests/value_conventions.rs` | Exercise checked values and schema contracts through the public library API. |
| `crates/biomcp-adapters/src/monarch/native_contracts.rs` | Exercise retained gene-keyed Monarch rows through the private mapper without adding a gene-keyed command. |
| `crates/biomcp-adapters/tests/alphagenome_registration.rs` | Exercise restricted source registration and required-key admission before a product section exists. |
| `crates/biomcp-adapters/tests/batch_bounds.rs` | Exercise literal indexed lookup inputs outside command validation and empty/oversized batches or chunks beyond command limits. |
| `crates/biomcp-adapters/tests/ctgov_refusals.rs` | Inspect registry refusal facts, legacy snapshot defaults and native continuation fields omitted from command JSON. |
| `crates/biomcp-adapters/tests/custom_refusals.rs` | Inspect adapter refusal facts and legacy snapshot codecs omitted from command JSON. |
| `crates/biomcp-adapters/tests/custom_refusals/collections.rs` | Inspect retained native member refusal facts, snapshot round trips and consumed counts omitted from command JSON. |
| `crates/biomcp-adapters/tests/custom_refusals/stored.rs` | Inspect stored native relation refusal facts and source member indexes omitted from command JSON. |
| `crates/biomcp-adapters/tests/custom_refusals/transport.rs` | Inspect native page consumption and retained member refusal facts omitted from command JSON. |
| `crates/biomcp-adapters/tests/protein_storage.rs` | Read legacy serialized adapter values through the public storage type without a source request. |
| `crates/biomcp-adapters/tests/pmc_body.rs` | Reconstruct complete source rows and retain JATS reference IDs and links omitted from command JSON. |
| `crates/biomcp-adapters/tests/pmc_limits.rs` | Reject invalid table spans and duplicate table IDs before allocation with a tiny decoder boundary table. |
| `crates/biomcp-fetch/src/cache/contention.rs` | Order independent SQLite snapshots and writer admission at the private transaction boundary without product hooks. |
| `crates/biomcp-fetch/src/destination_tests.rs` | Cover public DNS address and redirect rejection tables without contacting private destinations. |
| `crates/biomcp-fetch/src/grpc/dns_tests.rs` | Prove a stalled platform lookup cannot outlive its caller budget or begin a stream, without changing the host resolver configuration. |
| `crates/biomcp-fetch/src/retry_tests.rs` | Cover Retry-After forms and backoff boundaries without one recording per table row. |
| `crates/biomcp-fetch/src/xml_tests.rs` | Cover inline XML markup and decoder limits with many tiny inputs. |
| `crates/biomcp-adapters/tests/alphagenome.rs` | Exercise the selected request encoding and complete bounded envelopes through the public protocol API. |
| `crates/biomcp-fetch/tests/alphagenome.rs` | Exercise required-key routing through a synthetic stream. |
| `crates/biomcp-fetch/tests/alphagenome/routing.rs` | Exercise required-key admission and placeholder credential routing through a local TLS stream. |
| `crates/biomcp-adapters/tests/score_variant.rs` | Exercise the ruled scoring request and complete scorer admission through the public protocol API. |
| `crates/biomcp-fetch/tests/batch.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/bundle.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/bundle/download.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/bundle/native.rs` | Exercise complete native TSV replacement sets and decoded bounds through the public transport API. |
| `crates/biomcp-fetch/tests/bundle/rejections.rs` | Exercise rejected source rows, persisted revalidation and old-store migration through the public transport API. |
| `crates/biomcp-fetch/tests/bundle/rows.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/cache.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/cache/credentials.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/cache/freshness.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/cache/storage.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/grpc_persistence.rs` | Exercise checked RPC cache admission and bounded recording replay through the public transport API. |
| `crates/biomcp-fetch/tests/grpc_persistence/policy.rs` | Exercise credential containment, concurrent stream capture and atomic publication through the public transport API. |
| `crates/biomcp-fetch/tests/grpc_persistence/process.rs` | Exercise ordinary checked reply cache reuse from an isolated second process through the public transport API. |
| `crates/biomcp-fetch/tests/grpc_worker.rs` | Exercise blocking streaming RPC admission, local TLS HTTP/2, proxy routing, deadlines and cancellation through the public transport API. |
| `crates/biomcp-fetch/tests/double_gzip.rs` | Exercise double-compressed replies and body limits on each layer through the public transport API. |
| `crates/biomcp-fetch/tests/offline.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/paths.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/post.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/quotas.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/record.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/record/errors.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/record/metadata.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/retries.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/support/cache_process.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-adapters/tests/tensors.rs` | Exercise bounded tensor decoding with synthetic protobuf payloads through its public API. |
| `crates/biomcp-fetch/tests/transport.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/transport/cancellation.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/transport/publisher.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/transport/network.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/transport/credentials.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-fetch/tests/xml.rs` | Exercise transport ownership, local server behavior and cache/record failures through its public API. |
| `crates/biomcp-query/tests/article_search.rs` | Exercise concurrent pool publication/hydration, injected SQL corruption, external writers and API-only deadline boundaries. |
| `crates/biomcp-query/src/store/privacy_tests.rs` | Prove caller ownership at the private query store security boundary before licensed routes publish continuations. |
| `crates/biomcp-query/tests/gene_resolution.rs` | Mutate isolated cache freshness and preserve a local bundle failure under an API-only short command deadline. |
| `crates/biomcp/tests/ratchets.rs` | Enforce repository structure and prove refusal of planted violations in isolated trees. |
| `crates/biomcp/tests/ratchets/lints.rs` | Prove the shared Clippy settings reject planted scalar, banned-type and reason drift. |
| `crates/biomcp/tests/ratchets/dependency_sources.rs` | Reject unapproved Git revisions and missing source policy inputs through the dependency guard. |
| `crates/biomcp/tests/ratchets/proof.rs` | Enforce repository structure and prove refusal of planted violations in isolated trees. |
| `crates/biomcp-query/tests/article_rights.rs` | Inspect raw replies, ranked pools and full-text refusal identities through public queries using synthetic transport. |
| `crates/biomcp-query/tests/article_rights/authors.rs` | Prove private abstract refusal metadata survives raw cache reuse and rewrite legacy author continuation bytes through public queries. |
| `crates/biomcp-query/tests/article_rights/contexts.rs` | Inspect retained citation reply bytes and distinguish absent contexts from rights refusals through public adapters. |
| `crates/biomcp-query/tests/article_rights/lookups.rs` | Inspect stored replies and pools after failed or capped license lookups, and repeat reads, through public queries. |
| `crates/biomcp-query/tests/article_rights/routes.rs` | Inspect synthetic transport replies for source-owned abstract and passage rights and bounded dependency admission. |

Everything else goes through the runner.

## 11. Structural checks

Workspace lints, per-crate Clippy configuration and repository ratchets enforce dependency direction, source declarations, file boundaries, reasoned exceptions and test placement. The source owns each limit and exception list. `make landing-gate` combines formatting, lint, documentation, generated protobuf checks, dependency policy, secret scanning, library tests and binary coverage. Required coverage and structural ratchets also run from the coverage script.

Check: `crates/biomcp/tests/ratchets.rs`; Check: `scripts/test_coverage.py`; Check: `Makefile`.

The architecture omits proposed wiki tables, future web services, porting sketches and hand-maintained provider inventories. Each belongs in its own approved design or source declaration when implemented.
