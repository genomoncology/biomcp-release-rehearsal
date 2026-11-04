# 0508 — Keep an all-refused study catalog from replacing the last good one

Status: COMPLETE.
Landed: 7f5c1056bf8ca6c0e0c2c798d109ee47086b77c2

Milestone: 5

Depends on: 0419, 0496

## Outcome

A cBioPortal study catalog refresh publishes and activates a new bundle only when the native catalog is a valid empty list or holds at least one accepted study identity. When every native row is refused, the refresh publishes nothing, the last good catalog stays active, and the reply returns the refusal problems and native counts. With a last good catalog the reply serves it as stale; without one, the reply has zero studies. The status follows 0419's shared rules.

- Why: the 0496 admission inventory (m16776). `CatalogRow::decode_document` rejects a native `[]`, malformed study identities become `ParsedRows.rejections`, and `bundle/download.rs` publishes even when no row was accepted. Bundle acquisition returns only `BundleInfo` and stored reads need activation, so 0496's HTTP admission check cannot cover this owner.
- Starts from: `CatalogRow`, `ParsedRows`, `bundle/download.rs`, bundle activation, 0495's refusal facts and 0419's status rules.
- Rules: the sync receipt reports an all-refused refresh with a last good catalog as the retained rows and snapshot, a stale source state, and the native refresh count and problems listed separately; it gains no studies body, and later study reads keep serving the retained catalog (m17186). The bundle owner decides publication; no HTTP cache or paging layer is added. A valid native empty catalog publishes as empty. A failed refresh keeps the last good catalog, as stale serving already does. Partly refused catalogs still publish their accepted rows, with the refusals returned.
- Proof: over recorded replies, an all-refused catalog refresh leaves the earlier catalog active and returns its refusal problems and native count, and a native `[]` publishes an empty catalog; both fail on the base build.
- Changes: the cBioPortal catalog decoder, bundle download and activation, cases and baseline.
- Defers: nothing.
