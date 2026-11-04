# 0424 — Share adapter helpers and run variant sections through fan-out

Status: COMPLETE.
Landed: 63caf10889d231102277e8818b6f3a19036477bc

Milestone: 5

Depends on: 0418, 0455, 0420, 0423, 0494

## Outcome

One `bad_reply` constructor, one openFDA envelope, one GraphQL alias batch, one `received_date` and one XML element search replace their copies. Variant sections run through `fan_out::run` like other multi-source gets, and output keeps the declared section order.

- Why: 29 local `fn bad` copies, seven openFDA envelopes, two GraphQL batchers, two date parsers and eight XML root searches each drift separately. Variant sections run one after another.
- Starts from: the copy sites in restructure (d), `crates/biomcp-query/src/variant/sections.rs`, `fan_out.rs`.
- Rules: `ratchets/source.rs` refuses `fn bad`, `fn bad_reply`, `fn received_date` and `fn root` outside each helper's home. Rebase variant changes on 0436. Concurrent sections wait for the in-process cache mutex through the existing deadline- and cancellation-aware `wait_mutex` in `Cache::try_sql`, while external SQLite contention keeps `busy_timeout` zero and falls back at once; a warm cache makes no live request (m17506). An optional reply lookup treats an explicit `CacheAccess::admit` hold as a miss and goes to the network, and waits only behind ordinary short reply SQL, so the existing busy-read and held-admission progress controls keep passing (m17562).
- Proof: a planted local `fn bad` fails the ratchet; a replay case delays two variant sections under a request deadline that one delay fits and two do not, then asserts both sections complete; it fails on the base build, where the second section times out; the reply diff is empty otherwise.
- Changes: the copy sites, `crates/biomcp-adapters/src/openfda.rs`, `graphql.rs`, `received.rs`, `xml_element.rs`, the variant section module.
- Defers: the transport call (0428).
