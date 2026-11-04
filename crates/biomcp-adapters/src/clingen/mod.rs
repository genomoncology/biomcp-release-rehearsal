//! Published ClinGen validity and dosage bundles, keyed only by HGNC identity.
mod raw;
use crate::{Mapped, mapped::Problems, quotas};
use biodata::{GeneSymbol, HgncId};
use biomcp_fetch::{
    Bundle, Fetch, Find, Freshness, KeyKind, KeyUse, Leg, RequestError, RowFormat, SourceSpec,
};
use biomcp_types::{ClinGenValidity, Source, SyncResult};
use std::{marker::PhantomData, time::Duration};
/// ClinGen published summaries are generated anew for each request.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "CC0 1.0",
        url: "https://clinicalgenome.org/docs/terms-of-use/",
        usage: biomcp_types::Use::Open,
    },
    source: Source::ClinGen,
    base: "https://search.clinicalgenome.org",
    key: KeyUse::None,
    quota: &quotas::CLINGEN,
    deadline: Duration::from_secs(8),
};
const VALIDITY: Bundle<raw::Validity> = Bundle {
    spec: &SPEC,
    name: "clingen-validity",
    path: "/kb/gene-validity/download",
    format: RowFormat::Csv { banner_lines: 4 },
    fresh: Freshness::Days(1),
    row: PhantomData,
};
const DOSAGE: Bundle<raw::Dosage> = Bundle {
    spec: &SPEC,
    name: "clingen-dosage",
    path: "/kb/gene-dosage/download",
    format: RowFormat::Csv { banner_lines: 4 },
    fresh: Freshness::Days(1),
    row: PhantomData,
};
/// Synchronize both required files and sum their published metadata.
pub fn sync(fetch: &Fetch) -> Result<crate::Mapped<SyncResult>, RequestError> {
    let validity = fetch.sync(&VALIDITY)?;
    let dosage = fetch.sync(&DOSAGE)?;
    let mut problems = crate::mapped::bundle_problems(Source::ClinGen, &validity);
    problems.extend(crate::mapped::bundle_problems(Source::ClinGen, &dosage));
    let mut refusals = crate::bundle_mapping::bundle_refusals(&validity);
    refusals.include(crate::bundle_mapping::bundle_refusals(&dosage));
    Ok(crate::Mapped {
        refusals,
        problems,
        value: SyncResult {
            native_rows: None,
            snapshot: None,
            etag: None,
            last_modified: None,
            duplicate_identities: None,
            rows: validity.rows + dosage.rows,
            bytes: validity.bytes + dosage.bytes,
            downloaded_bytes: validity.downloaded_bytes + dosage.downloaded_bytes,
        },
    })
}
/// Read gene–disease validity without depending on the dosage summary.
pub fn validity(
    leg: &Leg<'_>,
    id: HgncId,
    symbol: &GeneSymbol,
) -> Result<Mapped<Box<[ClinGenValidity]>>, RequestError> {
    let mut problems = Problems {
        source: Source::ClinGen,
        record: crate::mapped::record(id),
        list: Vec::new(),
    };
    let key = id.to_string();
    let rejected = raw::rejected_key(symbol);
    let mut refusals = crate::Refusal::None;
    let value = leg
        .rows(&VALIDITY, Find::AnyExact(KeyKind::Id, &[&key, &rejected]))?
        .into_iter()
        .filter_map(|row| match row.checked(&mut problems) {
            Some(row) => Some(row),
            None => {
                refusals = crate::Refusal::Refused;
                None
            }
        })
        .collect();
    Ok(Mapped {
        refusals,
        value,
        problems: problems.list,
    })
}
/// Checked dosage fields from the independent dosage summary.
#[derive(Debug)]
pub struct Dosage {
    /// Published haploinsufficiency assessment.
    pub haploinsufficiency: Option<biodata::Text>,
    /// Published triplosensitivity assessment.
    pub triplosensitivity: Option<biodata::Text>,
    /// Published assessment date.
    pub date: Option<biodata::Text>,
}
/// Read dosage sensitivity without depending on the validity summary.
pub fn dosage(
    leg: &Leg<'_>,
    id: HgncId,
    symbol: &GeneSymbol,
) -> Result<Mapped<Option<Dosage>>, RequestError> {
    let key = id.to_string();
    let rejected = raw::rejected_key(symbol);
    let rows = leg.rows(&DOSAGE, Find::AnyExact(KeyKind::Id, &[&key, &rejected]))?;
    let mut problems = Problems {
        source: Source::ClinGen,
        record: None,
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let mut rows: Vec<_> = rows
        .into_iter()
        .filter_map(|row| match row.checked(&mut problems) {
            Some(row) => Some(row),
            None => {
                refusals = crate::Refusal::Refused;
                None
            }
        })
        .collect();
    let value = match rows.len() {
        0 => None,
        1 => rows.pop(),
        _ => {
            refusals = crate::Refusal::Refused;
            problems.required::<Dosage>("HGNC ID", None)
        }
    };
    Ok(Mapped {
        refusals,
        value,
        problems: problems.list,
    })
}
/// Small publisher landing page, never a validity or dosage export.
pub(crate) const HEALTH_PATH: &str = "/kb";
/// Both source-owned stored files are required for complete local readiness.
pub(crate) fn bundle_state(leg: &Leg<'_>) -> Result<biomcp_types::LocalBundleState, RequestError> {
    Ok(leg
        .stored_bundle_state(&VALIDITY)?
        .with(leg.stored_bundle_state(&DOSAGE)?))
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
