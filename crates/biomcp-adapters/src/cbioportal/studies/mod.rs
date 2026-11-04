//! Complete checked catalog replacement; reads never synchronize.
mod context;
mod raw;
mod resource;
use super::SPEC;
use biomcp_fetch::{
    Bundle, BundleInfo, BundleSync, Fetch, Find, Freshness, KeyKind, Leg, RequestError, RowFormat,
};
use biomcp_types::{StudySearch, StudySnapshot, SyncResult};
use std::marker::PhantomData;
const CATALOG: Bundle<raw::CatalogRow> = Bundle {
    spec: &SPEC,
    name: "cbioportal-studies",
    path: "/studies?projection=DETAILED",
    format: RowFormat::Document {
        media: biomcp_fetch::DocumentMedia::Json,
    },
    fresh: Freshness::Days(7),
    row: PhantomData,
};

/// Acquire a native catalog while preserving the last good catalog on refusal.
/// The second value reports stale serving from the retained active catalog.
pub fn sync(fetch: &Fetch) -> Result<(crate::Mapped<SyncResult>, bool), RequestError> {
    let (info, native_rows, rejections, bytes, downloaded_bytes, stale) =
        match fetch.sync_catalog(&CATALOG)? {
            BundleSync::Published(info) => {
                let native_rows = info.rows + info.rejections.len() as u64;
                let rejections = info.rejections.clone();
                let bytes = info.bytes;
                let downloaded_bytes = info.downloaded_bytes;
                (
                    Some(info),
                    native_rows,
                    rejections,
                    bytes,
                    downloaded_bytes,
                    false,
                )
            }
            BundleSync::Refused {
                native_rows,
                rejections,
                bytes,
                downloaded_bytes,
                retained,
            } => {
                let stale = retained.is_some();
                (
                    retained,
                    native_rows,
                    rejections,
                    bytes,
                    downloaded_bytes,
                    stale,
                )
            }
        };
    let refusals = if rejections.is_empty() {
        crate::Refusal::None
    } else {
        crate::Refusal::Refused
    };
    let problems =
        crate::bundle_mapping::rejection_problems(SPEC.source, CATALOG.name, &rejections);
    Ok((
        crate::Mapped {
            refusals,
            problems,
            value: SyncResult {
                snapshot: info.as_ref().map(snapshot).transpose()?,
                etag: info.as_ref().and_then(|info| info.etag.clone()),
                last_modified: info.as_ref().and_then(|info| info.last_modified.clone()),
                rows: info.as_ref().map_or(0, |info| info.rows),
                native_rows: Some(native_rows),
                bytes,
                downloaded_bytes,
                duplicate_identities: None,
            },
        },
        stale,
    ))
}
/// Return literal matches in original catalog order from the accepted local copy.
pub fn search(
    leg: &Leg<'_>,
    query: &StudySearch,
) -> Result<(crate::Mapped<Vec<biodata::StudyHeader>>, BundleInfo), RequestError> {
    let (rows, info) = leg.stored_rows(&CATALOG, Find::Exact(KeyKind::Band, "catalog"))?;
    let items = rows
        .into_iter()
        .filter(|r| r.matches(query))
        .map(|r| {
            r.header()
                .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::Cbioportal, None))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut problems = Vec::new();
    let mut refusals = crate::Refusal::None;
    let value = items
        .into_iter()
        .map(|mapped: crate::Mapped<biodata::StudyHeader>| {
            refusals.include(mapped.refusals);
            problems.extend(mapped.problems);
            mapped.value
        })
        .collect();
    Ok((
        crate::Mapped {
            refusals,
            value,
            problems,
        },
        info,
    ))
}
/// Read an exact case-sensitive metadata card from the stored catalog.
pub fn card(
    leg: &Leg<'_>,
    id: &biodata::StudyId,
) -> Result<(crate::Mapped<Option<biodata::Study>>, BundleInfo), RequestError> {
    let (rows, info) = leg.stored_rows(&CATALOG, Find::Exact(KeyKind::Id, id.as_str()))?;
    let row = rows
        .into_iter()
        .map(|r| {
            r.card()
                .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::Cbioportal, None))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .find(|v| v.value.header.study_id == *id);
    let mut problems = Vec::new();
    let mut refusals = crate::Refusal::None;
    let value = row.map(|row| {
        refusals.include(row.refusals);
        problems.extend(row.problems);
        row.value
    });
    Ok((
        crate::Mapped {
            refusals,
            value,
            problems,
        },
        info,
    ))
}
/// Catalog provenance never substitutes acquisition time for a publisher revision.
pub fn snapshot(info: &BundleInfo) -> Result<StudySnapshot, RequestError> {
    Ok(StudySnapshot {
        revision: info
            .content_revision
            .as_deref()
            .ok_or_else(|| biomcp_fetch::bad_reply(biomcp_types::Source::Cbioportal, None))?
            .parse()
            .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::Cbioportal, None))?,
        acquired: info
            .retrieved_at
            .and_then(|v| u64::try_from(v).ok())
            .ok_or_else(|| biomcp_fetch::bad_reply(biomcp_types::Source::Cbioportal, None))?,
    })
}

mod rows;
pub use rows::{mutations, samples};
mod sections;
pub use sections::{Records, profiles, sample_lists};
mod files;
pub use files::{Directory, files};

/// Inspect this source's validated local catalog without reading rows or synchronizing.
pub(crate) fn bundle_state(leg: &Leg<'_>) -> Result<biomcp_types::LocalBundleState, RequestError> {
    leg.stored_bundle_state(&CATALOG)
}
