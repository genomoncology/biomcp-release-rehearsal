# The key list names each key's source use label

Status: COMPLETE.
Landed: 3c13b3304cff0095492faf79106bcbc3c714aee5
Milestone: 5
Depends on: 0585

## Outcome

Each entry in the key-list output carries its source's use label (`open`, `attribution`, `per-record`, `noncommercial` or `licensed`), so hosting can tell which keys do anything under the `restricted` use without parsing help prose.

- Why: hosting's account page shows only the keys that work under `restricted` (m22772). At 41476c291 `RegisteredKey` (`crates/biomcp-types/src/response/key.rs`) carries source, config, environment, header, tier and state but no use label, and only the prose of `biomcp help sources` lists group membership.
- Starts from: `RegisteredKey` in `crates/biomcp-types/src/response/key.rs`, its builder in `crates/biomcp/src/keys.rs`, the registry labels from 0585, and `crates/biomcp-types/schemas/registered-keys.json`.
- Rules: the label comes from the same registry declaration that the declared uses read, never from a second table. A key whose source has components with other labels reports the source's own label; the components stay in source listings. No key value or fingerprint appears in the new field. Allowed exception to the command freeze for the hosted launch: only the key-list reply and its schema change.
- Proof: an installed-binary case lists keys with their labels, OncoKB `licensed` and NCBI `open` or `attribution`, matching what a hosted `restricted` call allows. The schema comparison names the one added field.
- Changes: `key.rs`, `keys.rs`, the key-list schema, and one case.
- Defers: nothing.
