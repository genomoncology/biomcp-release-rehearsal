//! Complete multi-file publication through the ordinary checked-byte transport.
use super::{Bundle, BundleInfo, Row, RowFormat, parse, store};
use crate::{Fetch, Freshness, RequestError, SourceSpec};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::marker::PhantomData;

/// One independently published part of a complete replacement set.
#[derive(Clone, Copy, Debug)]
pub struct BundlePart {
    /// Stable source-native part identifier.
    pub name: &'static str,
    /// Exact path under the source declaration.
    pub path: &'static str,
    /// A part-specific encoding; absent uses the set encoding.
    pub format: Option<RowFormat>,
}
/// A source requires every declared part before any row becomes visible.
#[derive(Debug)]
pub struct BundleSet<R: Row + 'static> {
    /// Canonical source and policy declaration.
    pub spec: &'static SourceSpec,
    /// Stable store name for the whole set.
    pub name: &'static str,
    /// Native part order, also defining row order.
    pub parts: &'static [BundlePart],
    /// Common native row encoding.
    pub format: RowFormat,
    /// Client freshness policy, independent of publication cadence.
    pub fresh: Freshness,
    /// Decoded native row type.
    pub row: PhantomData<R>,
}
/// A checked source row with its original location in the replacement set.
#[derive(Debug, Deserialize)]
pub struct SetRow<R> {
    /// Native part identifier.
    pub part: Box<str>,
    /// One-based logical row number, counting the header.
    pub row: u64,
    /// Original decoded native record.
    pub value: R,
}
impl<R: Row> Row for SetRow<R> {
    fn keys(&self) -> super::Keys {
        self.value.keys()
    }
}
impl<R: Row> BundleSet<R> {
    /// Inspect the complete stored set without downloading any part.
    pub fn state(
        &self,
        leg: &crate::Leg<'_>,
    ) -> Result<biomcp_types::LocalBundleState, RequestError> {
        leg.stored_bundle_state(&self.stored())
    }

    pub(super) fn stored(&self) -> Bundle<SetRow<R>> {
        Bundle {
            spec: self.spec,
            name: self.name,
            path: "",
            format: self.format,
            fresh: self.fresh,
            row: PhantomData,
        }
    }
}
impl Fetch {
    /// Download and check every part, then atomically replace the complete stored set.
    pub fn sync_set<R: Row>(&self, set: &BundleSet<R>) -> Result<BundleInfo, RequestError> {
        let deadline = self.deadline(self.config.sync_budget);
        let mut leg = self.leg(set.spec, deadline);
        leg.deadline = deadline;
        let bundle = set.stored();
        leg.bundle_source(&bundle)?;
        let cache = leg.bundle_cache()?;
        let mut complete = Vec::new();
        let mut rejections = Vec::new();
        let mut revision = Sha256::new();
        let mut bytes = 0_u64;
        for part in set.parts {
            let format = part.format.unwrap_or(set.format);
            let limit = match format {
                RowFormat::NativeCsv { max_bytes } => max_bytes.min(super::MAX_BUNDLE_BYTES),
                _ => R::MAX_BYTES.min(super::MAX_BUNDLE_BYTES),
            };
            let endpoint = set.spec.get(part.path).bytes(limit).fresh(Freshness::Never);
            let check = |body: &[u8]| {
                let parsed = parse::part_rows::<R>(body, format, part.name)
                    .map_err(|_| biomcp_types::Reason::BadReply)?;
                if parsed.is_empty() {
                    return Err(biomcp_types::Reason::BadReply);
                }
                Ok(())
            };
            let read = crate::Read::bytes().validate_body(&check);
            let read = match format {
                RowFormat::Document { media } => read.document_media(media),
                _ => read,
            };
            let body = leg
                .call(&endpoint, crate::Input::Params(&[]), read)?
                .value
                .0;
            bytes = bytes.saturating_add(body.len() as u64);
            if bytes > super::MAX_BUNDLE_BYTES {
                return Err(leg
                    .failure(biomcp_types::Reason::BadReply, Some(200))
                    .into());
            }
            revision.update((part.path.len() as u64).to_be_bytes());
            revision.update(part.path.as_bytes());
            revision.update((body.len() as u64).to_be_bytes());
            revision.update(&body);
            let rows = parse::part_rows::<R>(&body, format, part.name)
                .map_err(|_| leg.failure(biomcp_types::Reason::BadReply, Some(200)))?;
            rejections.extend(rows.rejections.into_iter().map(|mut rejection| {
                rejection.reason = format!("{}: {}", part.name, rejection.reason);
                rejection
            }));
            append_part(&mut complete, part, rows.rows)?;
        }
        if set.parts.is_empty() || leg.budget().expired() {
            return Err(leg.failure(biomcp_types::Reason::Syncing, None).into());
        }
        let hex: String = revision
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let content_revision = format!("sha256:{hex}");
        let metadata = store::Meta {
            rejections,
            bytes,
            rows: complete.len() as u64,
            fresh_until: crate::cache::now().saturating_add(
                i64::try_from(set.fresh.seconds().unwrap_or(0)).unwrap_or(i64::MAX),
            ),
            // Publisher HTTP validators never hold a computed set digest.
            etag: None,
            modified: None,
            content_revision: Some(content_revision),
            retrieved_at: Some(crate::cache::now()),
        };
        cache.publish(&bundle, None, &metadata, Some(&complete), leg.budget())?;
        let mut info = store::info(&bundle, &metadata);
        info.downloaded_bytes = leg.finish().downloaded_bytes;
        Ok(info)
    }
}
fn append_part(
    complete: &mut Vec<parse::Parsed>,
    part: &BundlePart,
    rows: Vec<parse::Parsed>,
) -> Result<(), RequestError> {
    for row in rows {
        let value: serde_json::Value =
            serde_json::from_str(&row.row).map_err(|_| crate::CacheError::Encoding)?;
        let document = serde_json::json!({"part":part.name,"row":row.number,"value":value});
        complete.push(parse::Parsed {
            number: row.number,
            row: document.to_string(),
            keys: row.keys,
        });
    }
    Ok(())
}
impl crate::Leg<'_> {
    /// Read one validated complete set without downloading or synchronizing any part.
    pub fn stored_set_rows<R: Row>(
        &self,
        set: &BundleSet<R>,
        find: super::Find<'_>,
    ) -> Result<(Vec<SetRow<R>>, BundleInfo), RequestError> {
        self.stored_rows(&set.stored(), find)
    }
}
