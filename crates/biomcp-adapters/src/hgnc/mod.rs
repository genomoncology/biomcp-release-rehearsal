//! HGNC published approved and withdrawn human gene identities.
mod fields;
mod map;
mod raw;
use crate::{Mapped, quotas};
use biodata::{Gene, GeneSymbol, HgncId, HgncStatus, MatchKind, Text};

use biomcp_fetch::{
    Bundle, BundleInfo, Fetch, Find, Freshness, KeyKind, KeyUse, Leg, RequestError, RowFormat,
    SourceSpec,
};
use biomcp_types::Source;
use biomcp_types::{GeneHeader, GeneInput, Match};
use std::{marker::PhantomData, time::Duration};

/// HGNC source declaration for published TSV bundles.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "CC0 1.0",
        url: "https://www.genenames.org/about/license/",
        usage: biomcp_types::Use::Open,
    },
    source: Source::Hgnc,
    base: "https://storage.googleapis.com/public-download-files/hgnc/tsv/tsv",
    key: KeyUse::None,
    quota: &quotas::HGNC,
    deadline: Duration::from_secs(120),
};
const COMPLETE: Bundle<raw::Gene> = Bundle {
    spec: &SPEC,
    name: "hgnc",
    path: "/hgnc_complete_set.txt",
    format: RowFormat::Tsv,
    fresh: Freshness::Days(7),
    row: PhantomData,
};
const WITHDRAWN: Bundle<raw::Withdrawn> = Bundle {
    spec: &SPEC,
    name: "hgnc-withdrawn",
    path: "/withdrawn.txt",
    format: RowFormat::Tsv,
    fresh: Freshness::Days(7),
    row: PhantomData,
};
/// Check publisher metadata through the routed transport without synchronizing rows.
pub(crate) fn probe(leg: &Leg<'_>) -> Result<(), RequestError> {
    leg.probe(&SPEC.head(COMPLETE.path), biomcp_fetch::Input::Params(&[]))
}
/// A published HGNC file, without exposing its wire rows.
#[derive(Debug, Clone, Copy)]
pub enum HgncBundle {
    /// Approved complete set.
    Complete,
    /// Withdrawn and merged identities.
    Withdrawn,
}
/// Synchronize one selected HGNC file through the checked transport.
pub fn sync(fetch: &Fetch, selector: HgncBundle) -> Result<BundleInfo, RequestError> {
    match selector {
        HgncBundle::Complete => fetch.sync(&COMPLETE),
        HgncBundle::Withdrawn => fetch.sync(&WITHDRAWN),
    }
}
/// Synchronize both identity files and retain rejected row disclosures.
pub fn sync_all(fetch: &Fetch) -> Result<Mapped<biomcp_types::SyncResult>, RequestError> {
    let complete = sync(fetch, HgncBundle::Complete)?;
    let withdrawn = sync(fetch, HgncBundle::Withdrawn)?;
    let mut problems = crate::mapped::bundle_problems(Source::Hgnc, &complete);
    problems.extend(crate::mapped::bundle_problems(Source::Hgnc, &withdrawn));
    let mut refusals = crate::bundle_mapping::bundle_refusals(&complete);
    refusals.include(crate::bundle_mapping::bundle_refusals(&withdrawn));
    Ok(Mapped {
        refusals,
        problems,
        value: biomcp_types::SyncResult {
            native_rows: None,
            snapshot: None,
            etag: None,
            last_modified: None,
            duplicate_identities: None,
            rows: complete.rows + withdrawn.rows,
            bytes: complete.bytes + withdrawn.bytes,
            downloaded_bytes: complete.downloaded_bytes + withdrawn.downloaded_bytes,
        },
    })
}
/// Map approved genes from one literal indexed lookup, retaining rejected fields.
pub fn lookup(leg: &Leg<'_>, find: Find<'_>) -> Result<Mapped<Vec<Gene>>, RequestError> {
    Ok(map::genes(leg.rows(&COMPLETE, find)?))
}
/// Withdrawn identity and every stated successor, without following chains.
#[derive(Debug)]
pub struct Withdrawn {
    /// Original HGNC identity.
    pub hgnc_id: HgncId,
    /// Original source symbol.
    pub symbol: GeneSymbol,
    /// Source-stated withdrawn status.
    pub hgnc_status: HgncStatus,
    /// All source-stated merge targets, in source order.
    pub merged_into: Box<[HgncId]>,
}
/// Find withdrawn records by explicit HGNC identity or source symbol.
pub fn withdrawn(leg: &Leg<'_>, input: &GeneInput) -> Result<Mapped<Vec<Withdrawn>>, RequestError> {
    let text = input.to_string();
    let kind = match input {
        GeneInput::HgncId(_) => KeyKind::Id,
        GeneInput::Text(_) => KeyKind::Symbol,
        _ => {
            return Ok(Mapped {
                refusals: crate::Refusal::None,
                value: Vec::new(),
                problems: Vec::new(),
            });
        }
    };
    Ok(map::withdrawn(
        leg.rows(&WITHDRAWN, Find::Exact(kind, &text))?,
    ))
}
/// Headers for checked single-band locations matching the literal prefix.
pub fn in_band(leg: &Leg<'_>, prefix: &str) -> Result<Mapped<Vec<GeneHeader>>, RequestError> {
    let rows = lookup(leg, Find::Prefix(KeyKind::Band, prefix))?;
    let value = rows
        .value
        .into_iter()
        .map(|gene| GeneHeader {
            hgnc_id: gene.hgnc_id,
            symbol: gene.symbol,
            locus_group: gene.locus_group,
            r#match: gene
                .location
                .as_ref()
                .and_then(|band| Text::new(band.as_str()).ok())
                .map(|text| Match {
                    kind: MatchKind::Band,
                    text,
                }),
            location: gene.location,
            status: gene.status,
            name: gene.name,
        })
        .collect();
    Ok(Mapped {
        refusals: rows.refusals,
        value,
        problems: rows.problems,
    })
}

/// Both source-owned identity files are required for complete local readiness.
pub(crate) fn bundle_state(leg: &Leg<'_>) -> Result<biomcp_types::LocalBundleState, RequestError> {
    Ok(leg
        .stored_bundle_state(&COMPLETE)?
        .with(leg.stored_bundle_state(&WITHDRAWN)?))
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
