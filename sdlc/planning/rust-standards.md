# Rust standards

Agents write this code. The compiler, clippy and the ratchet check enforce what they can. A fresh reviewer judges the rest on every ticket.

## Toolchain and layout

- One Cargo workspace with resolver 3 and edition 2024. `rust-toolchain.toml` pins one exact release with `clippy` and `rustfmt`. `rustfmt.toml` holds `style_edition = "2024"`.
- Crate boundaries follow the dependency direction in `architecture.md`. `biodata` depends on nothing that does I/O, and its `clippy.toml` disallows file, environment, network, process and clock paths one by one.
- Tests run the feature set that ships. The lint gate also checks `biodata` and `biomcp-types` without default features, so their `schema` feature remains optional.

## Lint table

One lint table sits in the workspace `Cargo.toml`, and every crate inherits it.

- Rust: `unsafe_code`, `unreachable_pub`, `missing_debug_implementations` and `missing_docs` are denied. Lints are `deny`, never `forbid`, so a rare exception can carry `#[expect(lint, reason = "...")]`. The ratchet check lists the files allowed to hold one.
- Clippy: `allow_attributes`, `wildcard_imports`, `missing_assert_message`, `fn_params_excessive_bools`, `ref_option`, `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo`, `unimplemented`, `dbg_macro`, `print_stdout`, `print_stderr` and `allow_attributes_without_reason` are denied. `too_many_lines` is denied at 80 lines. Tests may unwrap, expect, panic and index through the shared `clippy-base.toml`. Clippy excludes test functions from assertion-message checks and test imports from wildcard checks. The ratchet compares every crate config with the shared base.
- No `allow` attribute. No `dead_code` or `unused_imports` expectation, except the private pinned generated message closure, whose single expectation names its reason.
- `disallowed_types` denies `Cell`, `RefCell`, `Mutex`, `RwLock`, `OnceLock`, `LazyLock` and the atomics in every crate. Every necessary state owner has an item-level expectation with its concurrency or ownership reason. Whole-module state expectations fail the ratchet. The store needs none, because each call opens its own connection.
- `disallowed_methods` denies environment reads outside the binary's config module, and thread spawns outside the fan-out, the transport and the MCP server.
- Warnings fail the gate.

## Size

- No file over 600 lines. An adapter's `mod.rs`, `raw.rs` and `map.rs` each stay under 300.
- No function over 80 lines.
- `architecture.md` records a product-line budget per crate once its first slice lands. A change that raises a budget states why in its commit body.

## Types

- Parse at the edge. Untrusted input becomes a typed value once, and the inside never checks it again.
- A validated value gets its own type with a fallible constructor: `Pmid`, `NctId`, `HgncId`, `GeneSymbol`, `Position1`.
- A fixed set of choices is an enum. `fn_params_excessive_bools` refuses boolean-heavy signatures. A single boolean flag remains valid.
- A field that must be present is not an `Option`.
- No `String` field holds an identifier, closed set, date or coordinate. The schema shows each field's type, and the reviewer checks it.

## biodata rules

Every type in `biodata` follows these rules.

1. Newtypes have private fields and one checked way in. Once a value exists, it is known to be good. A plain record may have public fields when every field is itself a checked type.
2. Every type with a text form implements `FromStr` and `Display`, and parsing the printed form gives back an equal value. A case checks this for each row.
3. Closed sets are enums. A set that may grow, such as trial phase or publication type, is `#[non_exhaustive]`. A set that never grows, such as the four DNA bases, is not.
4. One-based positions and counts are `NonZeroU32`.
5. Every coordinate carries its assembly and its counting rule in its type: `Position1`, `Span0`, `GenomicInterval`. Where a source does not state the assembly, the value says unstated. Nothing defaults to one build.
6. Values never change after they are built. Owned text and lists that never grow are `Box<str>` and `Box<[T]>`.
7. Errors are enums with stable codes that implement `Display` and `std::error::Error`. No error message holds input text.
8. Every identifier derives `Hash` with `Eq`, and they agree. A symbol compared without case hashes without case.
9. Serde runs in both directions, and deserializing runs the same checks as construction, through `try_from` a raw shape. Enums with data are internally tagged with a field named for what they choose, such as `state`, `resolution` or `command`, and snake_case names. No biodata type uses `#[serde(flatten)]`, because serde refuses it beside `deny_unknown_fields`. Field names in JSON are public API, and renaming one raises the schema's major version.
10. JSON Schema comes from `schemars` behind the `schema` feature, is checked in under `schemas/`, carries a name and major version such as `biodata.gene/1`, and fails the build on drift.
11. Each case value round-trips through JSON. BioMCP writes and reads only JSON, so other formats wait for a consumer.
12. Dependencies stay lean: `serde`, `winnow` and optional `schemars`. No HTTP client, XML parser, async runtime or database driver.
13. Types stay ready for bindings: owned data, no lifetimes, no trait objects and no generic parameter a binding cannot name. Bindings, when a consumer asks for them, live in their own crates.
14. The crate README is its crate docs through `#![doc = include_str!("../README.md")]`, so its example compiles.
15. Source sections and display text stay out of entity types. A card is a response type that holds biodata values and adds its sections and per-section states.

## Errors

- Each library crate defines one error enum with `thiserror`. No public signature returns a string error or a boxed unknown error.
- Every error has a stable code. The binary maps each error family to an exit code in one function.
- Source identity is an enum, never a string.
- An ignored `Result` is a bug.

## Ownership and state

- Borrow first. A clone is a decision.
- Memory stays bounded: body size limits, bounded fan-out and streamed bundle reads.
- Services take their context as arguments and hold no global state.

## Dependencies

A dependency is not free. Follow `decisions/2026-10-23-least-code-easy-upkeep-fast.md`: depend, vendor MIT or Apache code, or write our own, case by case. The commit body names the code a dependency saves, the upkeep it adds and whether it sits on a hot path. No GPL, LGPL or AGPL code. One crate per job: one date crate, one hash crate, one HTML reader, one XML reader and no HTTP cache crate. `architecture.md` lists each crate's allowed dependencies.

## Tests

- Outside-in cases define what the code does. Code they do not run is dead, and code that repeats other code is duplicated. Both are deleted before a ticket lands.
- Red, green, remove. Write the case first and watch it fail. Write the code that makes it pass. Then remove every line, branch and helper the cases do not need.
- Cases reach the code only through the `biomcp` binary and the MCP tool. Inline tests are allowed only for a listed exception with a stated reason, such as a parser table or the ranker's tie-break rules.
- One case per behavior, on the surface that owns it. Edge cases go in one table per behavior.
- Assert answers, statuses, exit codes and error codes. Pin message text only where the text is the contract.
- No case touches the network. Each adapter has a recorded 429, 500, timeout, broken body and 200-with-error-body case.
- The transport caches a reply only after it decodes and passes its raw type's `check`. It never caches another non-2xx reply, and caches a 404 for ten minutes.
- An adapter's `raw.rs` never uses `#[serde(default)]`. A field the provider may omit is an `Option`, and the type's `check` decides whether its absence is an error. The ratchet check enforces this.
- Each case step and each section a slice budgets carries a `max_bytes` or `max_tokens` ceiling from the slice's token budget.
- Product code holds no hook that only tests use. Replay works through the transport's offline mode, so no per-source base URL setting exists.

## Sources

- Each adapter's `SourceSpec` names one quota, and each quota declares a `SourceKey` chosen from the provider's documented limit. Sources that share a provider limit share the key. Nothing derives a limiter from a URL, host or endpoint. The limiter takes only a `&'static Quota`, and the ratchet check confines `SourceKey::declare` and quota statics to `quotas.rs` and refuses duplicate keys.
- A source whose terms need the user's own license declares `KeyUse::Required`. Without the user's key the transport never calls it, including its public endpoints, and the output reports it as skipped with reason `no_key`.

## Ratchets

Each ratchet starts at its 1.0 target because the code starts empty. The 0.9 column is BioMCP 0.9 as measured on 2026-10-23, to show what each rule prevents. `make landing-gate` enforces every ratchet whose mechanism is a tool.

| Ratchet | 0.9 | 1.0 target | Mechanism |
| --- | --- | --- | --- |
| Reverse layer edges (render to sources, sources to entities, entities to CLI or render, error to entities) | 21, 44, 3, 30 | 0 | Crate boundaries in Cargo |
| Crates that name the HTTP client (`ureq` in 1.0, `reqwest` in 0.9) | 1 crate, 15 files outside sources | `biomcp-fetch` only | Cargo and the ratchet check |
| JSON outputs with a checked-in, versioned schema | 0 | every command | Schema drift test and runner validation |
| Max product file length | 10,718 lines; 160 files over 600 | 600; adapter files 300 | Line count in the ratchet check |
| Max function length | threshold 645; 179 functions over 80 | 80 | `clippy::too_many_lines` |
| Per-adapter request helpers and retry paths | 29 and 3 | 0 and 1 | Ratchet check |
| Global or shared mutable state outside the transport | 101 statics, 110 lazy, 8 task-local, 47 `Mutex` | 0 | clippy `disallowed_types` |
| Unit tests in product crates | 4,061 | listed exceptions only | Ratchet check counts `#[test]` |
| Environment reads outside the config module | 88 in 47 files | 0 | clippy `disallowed_methods` |
| Source names as strings in errors | 415 | 0 | Typed source enum |
| `if json` branches in interface code | 185 | 1 | One render dispatch |
| Suppression attributes without a reason | 122 dead code, 56 unused imports, 29 too many arguments | 0 | `allow_attributes_without_reason`; no dead-code allows |
| `String` fields holding IDs, enums, dates or coordinates | 1,092 `String` fields in entities | free text only | Review and schema types |
| Test hooks in product code | 23 test variables, 76 base-URL overrides | 0 | Replay through the transport |
| Feature set tested against feature set shipped | differ | same | The binary enables `biodata`'s `schema` feature; one build profile in the gate |
| `#[serde(default)]` in adapter raw types | not measured | 0 | Ratchet check |
| Limiters keyed by URL or host | not measured | 0 | Limiter takes a `&'static Quota`; ratchet check on `quotas.rs` |
| Output size per command and section | not measured | under each case's ceiling | Runner `max_bytes` and `max_tokens` |
| Runtime dependencies | 57 direct, 481 locked | each justified, listed per crate | Allowed list in `architecture.md` |
| Documented commands run in a check | about 20% of spec pages | every docs example | Docs example check |

## Review

A fresh reviewer reads each ticket's change before it lands and checks:

- Duplicated logic anywhere in the workspace, not only in the diff.
- Lines the cases never run. Each is dead or missing its case.
- Defensive code that cannot fire.
- Parsing only at the edge, and validated values in their own types.
- A case for every command and option the change adds, with a story, a harvest source and a verified answer.
- Size: the change is no larger than its behavior needs.
