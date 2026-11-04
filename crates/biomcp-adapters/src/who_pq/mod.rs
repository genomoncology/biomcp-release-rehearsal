//! WHO finished-product CSV stays source-scoped and synchronizes only explicitly.
mod raw;
use biomcp_fetch::{
    BundleInfo, BundlePart, BundleSet, Fetch, Find, Freshness, KeyKind, KeyUse, Leg, RequestError,
    RowFormat, SourceSpec, Terms,
};
use biomcp_types::{DrugWhoCard, DrugWhoRow, Source, Use};
use std::{marker::PhantomData, time::Duration};
/// One declaration owns publisher terms, admission and public source outcomes.
pub static SPEC: SourceSpec = SourceSpec {
    source: Source::WhoPq,
    base: "https://extranet.who.int/prequal",
    terms: Terms {
        name: "WHO copyright policy; export-specific reuse grant unverified",
        url: "https://www.who.int/about/policies/publishing/copyright",
        usage: Use::Noncommercial,
    },
    quota: &crate::quotas::WHO,
    key: KeyUse::None,
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    deadline: Duration::from_secs(15),
};
const SET: BundleSet<raw::Native> = BundleSet {
    spec: &SPEC,
    name: "who-pq",
    parts: &[BundlePart {
        name: "finished-products",
        path: "/medicines/prequalified/finished-pharmaceutical-products/export?page&_format=csv",
        format: None,
    }],
    format: RowFormat::NativeCsv {
        max_bytes: 4 * 1024 * 1024,
    },
    // Local freshness policy, not a claim about WHO's publication cadence.
    fresh: Freshness::Days(3),
    row: PhantomData,
};
/// Validate the whole native export before atomically replacing its stored snapshot.
pub fn sync(fetch: &Fetch) -> Result<crate::Mapped<biomcp_types::SyncResult>, RequestError> {
    fetch
        .sync_set(&SET)
        .map(|info| crate::mapped::bundle_sync(SPEC.source, info))
}
/// Literal whole-word names and inclusive native dates; read never downloads or resolves synonyms.
pub fn search(
    leg: &Leg<'_>,
    query: &biomcp_types::DrugSearch,
) -> Result<Vec<DrugWhoRow>, RequestError> {
    let find = query.query().map_or(Find::Prefix(KeyKind::Id, ""), |name| {
        Find::Words(name.as_str())
    });
    let (rows, _) = leg.stored_set_rows(&SET, find)?;
    let from = query.prequalified_from();
    let to = query.prequalified_to();
    let mut rows: Vec<_> = rows
        .into_iter()
        .filter_map(|row| {
            let row = row.value;
            if query
                .query()
                .is_some_and(|name| !row.matches(name.as_str()))
            {
                return None;
            }
            let date = row.prequalified_date();
            if (from.is_some() || to.is_some())
                && !date.is_ok_and(|date| {
                    let key = (
                        date.year(),
                        date.month().unwrap_or(1),
                        date.day().unwrap_or(1),
                    );
                    from.is_none_or(|bound| {
                        key >= (
                            bound.year(),
                            bound.month().unwrap_or(1),
                            bound.day().unwrap_or(1),
                        )
                    }) && to.is_none_or(|bound| {
                        key <= (
                            bound.year(),
                            bound.month().unwrap_or(12),
                            bound.day().unwrap_or(31),
                        )
                    })
                })
            {
                return None;
            }
            Some((date, row))
        })
        .collect();
    if query.query().is_none() {
        rows.sort_by_key(|(date, _)| {
            std::cmp::Reverse(
                date.map_or(None, |date| Some((date.year(), date.month(), date.day()))),
            )
        });
    }
    Ok(rows.into_iter().map(|(_, row)| row.into()).collect())
}
/// Exact case-sensitive native reference, preserving spaces and all native cells.
pub fn get(
    leg: &Leg<'_>,
    reference: &biodata::Text,
) -> Result<(Option<DrugWhoCard>, BundleInfo), RequestError> {
    let (rows, info) = leg.stored_set_rows(&SET, Find::Exact(KeyKind::Id, reference.as_str()))?;
    let rows: Box<[_]> = rows
        .into_iter()
        .filter(|row| row.value.reference() == reference)
        .map(|row| row.value.into())
        .collect();
    let card = (!rows.is_empty()).then(|| DrugWhoCard {
        reference: reference.clone(),
        native_rows: rows,
    });
    Ok((card, info))
}

/// Small read-only publisher probe; never an export or synchronization.
pub(crate) const HEALTH_PATH: &str = "/medicines/prequalified/finished-pharmaceutical-products";

/// Inspect this source's complete local set without synchronizing it.
pub(crate) fn bundle_state(
    leg: &biomcp_fetch::Leg<'_>,
) -> Result<biomcp_types::LocalBundleState, biomcp_fetch::RequestError> {
    SET.state(leg)
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
