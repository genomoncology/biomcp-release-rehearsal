# 0428 — Replace transport combination methods with one call

Status: COMPLETE.
Landed: 328206179e1eb043f44794225a3670deeb480002

Milestone: 5

Depends on: 0424, 0422

## Outcome

`Leg::call(endpoint, input, read)` with a public `Read` options type replaces the combination methods. `probe`, bundle reads, `batch` and `grpc` stay. `Decoded<T>` becomes a named struct. One `Paged` trait replaces the 20 `at_offset` and 12 `restart` copies.

- Why: each new option combination added a method, and 174 call sites choose among about 30 near-identical names.
- Starts from: `crates/biomcp-fetch/src/request.rs`, `json_request.rs`, `cache_request.rs`, `counted.rs`, `xml.rs` `Decoder`, the call sites in adapters and query, and the paging copies and `page!` macros in `biomcp-types`.
- Rules: restructure (e). Count the methods and macros before starting. Reply keys stay, so recordings match. Old methods become delegates, call sites convert, then the delegates go, all on one branch. `page!` macros merge only under the `$defs` names 0438 fixes.
- Proof: deleting the old methods turns every unconverted call into a compile error; the transport's tests pass with unchanged assertions; the reply diff is empty; every recording still matches with no new recording.
- Changes: the files above and every adapter folder.
- Defers: nothing.
