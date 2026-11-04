//! Canonical EMA source declaration.
mod map;
mod raw;
use biodata::DrugId;
use biomcp_fetch::{
    Bundle, Fetch, Find, Freshness, KeyKind, KeyUse, Leg, RequestError, RowFormat, SourceSpec,
    Terms,
};
use biomcp_types::Source;
use biomcp_types::{DrugCard, DrugHeader, DrugRecords, DrugRegulatory, Section};
use std::marker::PhantomData;
use std::time::Duration;
/// Publisher policy shared by health, request admission and response outcomes.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::Ema,
    base: "https://www.ema.europa.eu",
    terms: Terms {
        name: crate::drug_sources::EMA_TERMS.name,
        url: crate::drug_sources::EMA_TERMS.url,
        usage: crate::drug_sources::EMA_TERMS.usage,
    },
    quota: &crate::quotas::EMA,
    key: KeyUse::None,
    deadline: Duration::from_secs(15),
};

const MEDICINES: Bundle<raw::Medicine> = Bundle {
    spec: &SPEC,
    name: "ema",
    path: "/en/documents/report/medicines-output-medicines_json-report_en.json",
    format: RowFormat::Json {
        rows: "data",
        metadata: "meta",
    },
    fresh: Freshness::Days(1),
    row: PhantomData,
};
/// Explicit complete-file synchronization through the shared atomic bundle store.
pub fn sync(fetch: &Fetch) -> Result<crate::Mapped<biomcp_types::SyncResult>, RequestError> {
    fetch
        .sync(&MEDICINES)
        .map(|info| crate::mapped::bundle_sync(SPEC.source, info))
}
/// Literal medicine-name candidates in native bundle order; never synchronize on reads.
pub fn search(leg: &Leg<'_>, query: &str) -> Result<crate::Mapped<Vec<DrugHeader>>, RequestError> {
    let (rows, _) = leg.stored_rows(&MEDICINES, Find::Words(query))?;
    Ok(name_headers(&rows, query, false))
}
/// Resolve names from the existing local rows, including snapshots whose older word index omitted generic fields.
/// Native ID lookup remains indexed; reads never synchronize or modify the stored snapshot.
pub fn search_names(
    leg: &Leg<'_>,
    query: &str,
) -> Result<crate::Mapped<Vec<DrugHeader>>, RequestError> {
    let (rows, _) = leg.stored_rows(&MEDICINES, Find::Prefix(KeyKind::Id, ""))?;
    Ok(name_headers(&rows, query, true))
}
fn name_headers(
    rows: &[raw::Medicine],
    query: &str,
    only_matches: bool,
) -> crate::Mapped<Vec<DrugHeader>> {
    let mut problems = fields(None);
    let value = rows
        .iter()
        .filter_map(|row| {
            let exact = |field| {
                row.field(field)
                    .and_then(Result::ok)
                    .is_some_and(|n| biomcp_types::DrugNameMatch::same_name(n, query))
            };
            let fields = [
                "active_substance",
                "international_non_proprietary_name_common_name",
            ];
            let generic_exact = fields.iter().any(|field| exact(field));
            let generic = fields.iter().any(|field| {
                row.field(field)
                    .and_then(Result::ok)
                    .is_some_and(|value| biomcp_types::DrugNameMatch::generic_name(value, query))
            });
            let matched = biomcp_types::DrugNameMatch::from_fields(
                exact("name_of_medicine") && !generic_exact,
                generic,
            );
            if only_matches && matched.is_none() {
                return None;
            }
            problems.record = Some(row.record.clone());
            let mut header = map::header(row, &mut problems);
            header.matched_name = matched;
            Some(header)
        })
        .collect();
    crate::Mapped {
        refusals: crate::Refusal::None,
        value,
        problems: problems.list,
    }
}
fn exact(leg: &Leg<'_>, id: &DrugId) -> Result<Option<raw::Medicine>, RequestError> {
    let (mut rows, _) = leg.stored_rows(&MEDICINES, Find::Exact(KeyKind::Id, &id.to_string()))?;
    // SQLite indexes fold keys; exact get still requires the published identifier itself.
    Ok(rows.pop().filter(|row| row.id == *id))
}
/// Exact native EMA identity card without authorization events.
pub fn card(leg: &Leg<'_>, id: &DrugId) -> Result<crate::Mapped<Option<DrugCard>>, RequestError> {
    let mut problems = fields(Some(id));
    let value = exact(leg, id)?
        .as_ref()
        .map(|row| map::card(row, &mut problems));
    Ok(crate::Mapped {
        refusals: crate::Refusal::None,
        value,
        problems: problems.list,
    })
}
/// Source-native dates and status for only the selected EMA application record.
pub fn regulatory(
    leg: &Leg<'_>,
    id: &DrugId,
) -> Result<crate::Mapped<Option<DrugRegulatory>>, RequestError> {
    let mut problems = fields(Some(id));
    let value = exact(leg, id)?.map(|row| DrugRegulatory {
        identity: map::header(&row, &mut problems),
        regulatory: Section::Data {
            source: Source::Ema,
            data: DrugRecords {
                records: vec![map::event(&row, &mut problems)].into(),
                offset: 0,
                total: Some(1),
                truncated: None,
                version: problems.field("_bundle_meta.timestamp", row.meta.timestamp),
                retrieved: None,
            },
        },
    });
    Ok(crate::Mapped {
        refusals: crate::Refusal::None,
        value,
        problems: problems.list,
    })
}
fn fields(id: Option<&DrugId>) -> crate::mapped::Problems {
    crate::mapped::Problems {
        source: Source::Ema,
        record: id.and_then(|id| crate::mapped::text(&id.to_string()).ok()),
        list: Vec::new(),
    }
}

/// Inspect this source's validated local catalog without reading rows or synchronizing.
pub(crate) fn bundle_state(leg: &Leg<'_>) -> Result<biomcp_types::LocalBundleState, RequestError> {
    leg.stored_bundle_state(&MEDICINES)
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
