use super::{SPEC, raw::Native};
use biomcp_fetch::{
    BundleInfo, BundlePart, BundleSet, Fetch, Find, Freshness, KeyKind, Leg, RequestError,
    RowFormat, SetRow,
};
use biomcp_types::SyncResult;
use std::{collections::BTreeMap, marker::PhantomData, time::Duration};
const SET: BundleSet<Native> = BundleSet {
    spec: &SPEC,
    name: "gtr",
    fresh: Freshness::Days(7),
    row: PhantomData,
    format: RowFormat::NativeTsv {
        gzip: false,
        max_decoded_bytes: 256 * 1024 * 1024,
    },
    parts: &[
        BundlePart {
            name: "versions",
            path: "/data/test_version.gz",
            format: Some(RowFormat::NativeTsv {
                gzip: true,
                max_decoded_bytes: 256 * 1024 * 1024,
            }),
        },
        BundlePart {
            name: "relations",
            path: "/data/test_condition_gene.txt",
            format: None,
        },
    ],
};
pub(super) fn read(
    leg: &Leg<'_>,
    find: Find<'_>,
) -> Result<(Vec<SetRow<Native>>, BundleInfo), RequestError> {
    leg.stored_set_rows(&SET, find)
}
/// Synchronize both native files atomically and count identities before any status filter.
pub fn sync(fetch: &Fetch) -> Result<crate::Mapped<SyncResult>, RequestError> {
    let info = fetch.sync_set(&SET)?;
    let deadline = fetch.deadline(Duration::from_secs(120));
    let leg = fetch.leg(&SPEC, deadline);
    let (rows, snapshot) = read(&leg, Find::Exact(KeyKind::Band, "versions"))?;
    if info.content_revision != snapshot.content_revision {
        return Err(biomcp_fetch::bad_reply(biomcp_types::Source::Gtr, None));
    }
    let mut counts = BTreeMap::new();
    for row in rows {
        if let Some(id) = row.value.id() {
            *counts.entry(id.to_string()).or_insert(0_u32) += 1;
        }
    }
    let problems = crate::mapped::bundle_problems(biomcp_types::Source::Gtr, &info);
    Ok(crate::Mapped {
        refusals: crate::bundle_mapping::bundle_refusals(&info),
        problems,
        value: SyncResult {
            native_rows: None,
            snapshot: None,
            etag: None,
            last_modified: None,
            rows: info.rows,
            bytes: info.bytes,
            downloaded_bytes: info.downloaded_bytes,
            duplicate_identities: Some(counts.values().filter(|&&n| n > 1).count() as u64),
        },
    })
}

/// Inspect this source's complete local set without synchronizing it.
pub(crate) fn state(leg: &Leg<'_>) -> Result<biomcp_types::LocalBundleState, RequestError> {
    SET.state(leg)
}
