use super::{NativeRow, SPEC, identity, partners};
use biodata::{DrugId, Text};
use biomcp_fetch::{
    BundleInfo, BundlePart, BundleSet, Fetch, Find, Freshness, KeyKind, Leg, RequestError,
    RowFormat,
};
use biomcp_types::{
    DrugCard, DrugHeader, DrugInteractions, DrugRecords, DrugSection, Section, Source,
};
use std::marker::PhantomData;
macro_rules! part {
    ($code:literal) => {
        BundlePart {
            format: None,
            name: $code,
            path: concat!(
                "/static/media/download/ddinter_downloads_code_",
                $code,
                ".csv"
            ),
        }
    };
}
pub(super) const PARTS: &[BundlePart] = &[
    part!("A"),
    part!("B"),
    part!("D"),
    part!("H"),
    part!("L"),
    part!("P"),
    part!("R"),
    part!("V"),
];
const PAIRS: BundleSet<NativeRow> = BundleSet {
    spec: &SPEC,
    name: "ddinter",
    parts: PARTS,
    format: RowFormat::Csv { banner_lines: 0 },
    fresh: Freshness::Days(7),
    row: PhantomData,
};
/// Synchronize the eight validated partitions as one atomic local replacement.
pub fn sync(fetch: &Fetch) -> Result<crate::Mapped<biomcp_types::SyncResult>, RequestError> {
    fetch
        .sync_set(&PAIRS)
        .map(|info| crate::mapped::bundle_sync(SPEC.source, info))
}
/// Discover literal native candidates from stored rows; never synchronize on a read.
pub fn search(leg: &Leg<'_>, query: &str) -> Result<crate::Mapped<Vec<DrugHeader>>, RequestError> {
    let (rows, _) = leg.stored_set_rows(&PAIRS, Find::Words(query))?;
    Ok(super::candidates(rows, query))
}
fn exact(leg: &Leg<'_>, id: &DrugId) -> Result<(Vec<super::PartRow>, BundleInfo), RequestError> {
    let id = id.to_string();
    let (rows, info) = leg.stored_set_rows(&PAIRS, Find::Exact(KeyKind::Id, &id))?;
    Ok((
        rows.into_iter()
            .filter(|r| r.value.id_a() == id || r.value.id_b() == id)
            .collect(),
        info,
    ))
}
fn selected(rows: &[super::PartRow], id: &str) -> (Option<DrugHeader>, crate::Refusal) {
    let found = rows
        .iter()
        .flat_map(|row| identity::identities(&row.value))
        .find(|(key, _)| *key == id);
    match found {
        None => (None, crate::Refusal::None),
        Some((key, name)) => match identity::identity(key, name) {
            Ok(value) => (Some(identity::header(value)), crate::Refusal::None),
            Err(_) => (None, crate::Refusal::Refused),
        },
    }
}
/// Return only the exact selected native identity and available source section.
pub fn card(leg: &Leg<'_>, id: &DrugId) -> Result<crate::Mapped<Option<DrugCard>>, RequestError> {
    let (rows, info) = exact(leg, id)?;
    let problems = rows
        .iter()
        .flat_map(|row| {
            row.value
                .problems(crate::mapped::record(format!("{}:{}", row.part, row.row)))
        })
        .collect();
    let (selected, refusals) = selected(&rows, &id.to_string());
    let value = selected.map(|header| DrugCard {
        header,
        alternate_names: Box::default(),
        official_names: Box::default(),
        cross_references: Box::default(),
        parents: Box::default(),
        products: Box::default(),
        formulations: Box::default(),
        version: info
            .content_revision
            .as_deref()
            .and_then(|v| Text::new(v).ok()),
        updated: None,
        retrieved: crate::received_date(info.retrieved_at),
        available_sections: vec![DrugSection::Interactions].into(),
        next: Box::default(),
    });
    Ok(crate::Mapped {
        refusals,
        value,
        problems,
    })
}
/// Page exact native partners, preserving the source order and every duplicate origin.
pub fn interactions(
    leg: &Leg<'_>,
    id: &DrugId,
    limit: u32,
    offset: u32,
) -> Result<crate::Mapped<Option<DrugInteractions>>, RequestError> {
    let (rows, info) = exact(leg, id)?;
    let (selected, mut refusals) = selected(&rows, &id.to_string());
    let Some(identity) = selected else {
        return Ok(crate::Mapped {
            refusals,
            value: None,
            problems: Vec::new(),
        });
    };
    let pairs = partners(rows, &id.to_string());
    let total = pairs.len() as u64;
    let mut problems = Vec::new();
    let records = pairs
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(|pair| {
            let mapped = pair.claim(info.content_revision.as_deref())?;
            refusals.include(mapped.refusals);
            problems.extend(mapped.problems);
            Ok::<_, biodata::Error>(mapped.value)
        })
        .collect::<Result<Box<[_]>, _>>()
        .map_err(|_| biomcp_fetch::CacheError::Limit)?;
    Ok(crate::Mapped {
        refusals,
        problems,
        value: Some(DrugInteractions {
            identity,
            interactions: Section::Data {
                source: Source::Ddinter,
                data: DrugRecords {
                    records,
                    offset,
                    total: Some(total),
                    truncated: None,
                    version: info
                        .content_revision
                        .as_deref()
                        .and_then(|v| Text::new(v).ok()),
                    retrieved: crate::received_date(info.retrieved_at),
                },
            },
        }),
    })
}

/// Inspect this source's validated local catalog without reading rows or synchronizing.
pub(crate) fn bundle_state(leg: &Leg<'_>) -> Result<biomcp_types::LocalBundleState, RequestError> {
    PAIRS.state(leg)
}
