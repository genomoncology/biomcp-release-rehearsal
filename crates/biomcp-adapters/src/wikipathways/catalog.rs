use super::{SPEC, catalog_row::CatalogRow};
use crate::Mapped;
use biodata::{Pathway, PathwayId, PathwaySpecies, WikiPathwaysId};
use biomcp_fetch::{
    BundleInfo, BundlePart, BundleSet, Fetch, Find, Freshness, KeyKind, Leg, RequestError,
    RowFormat,
};
use biomcp_types::{PathwayCard, PathwayHeader, PathwaySection, WikiPathwaysSync};
use std::marker::PhantomData;
const CATALOGS: BundleSet<CatalogRow> = BundleSet {
    spec: &SPEC,
    name: "wikipathways",
    parts: &[
        BundlePart {
            name: "search",
            path: "/json/findPathwaysByText.json",
            format: None,
        },
        BundlePart {
            name: "get",
            path: "/json/getPathwayInfo.json",
            format: None,
        },
    ],
    format: RowFormat::Document {
        media: biomcp_fetch::DocumentMedia::Json,
    },
    fresh: Freshness::Days(7),
    row: PhantomData,
};

/// Download both metadata catalogs and publish their checked pair atomically.
pub fn sync(fetch: &Fetch) -> Result<crate::Mapped<WikiPathwaysSync>, RequestError> {
    let info = fetch.sync_set(&CATALOGS)?;
    let leg = fetch.leg(&SPEC, fetch.deadline(SPEC.deadline));
    let (rows, _) = leg.stored_set_rows(&CATALOGS, Find::Exact(KeyKind::Band, "catalog"))?;
    let search = rows
        .iter()
        .find(|r| r.part.as_ref() == "search")
        .ok_or_else(|| biomcp_fetch::bad_reply(biomcp_types::Source::WikiPathways, None))?
        .value
        .receipt
        .clone();
    let get = rows
        .iter()
        .find(|r| r.part.as_ref() == "get")
        .ok_or_else(|| biomcp_fetch::bad_reply(biomcp_types::Source::WikiPathways, None))?
        .value
        .receipt
        .clone();
    let problems = crate::mapped::bundle_problems(SPEC.source, &info);
    Ok(crate::Mapped {
        refusals: crate::bundle_mapping::bundle_refusals(&info),
        problems,
        value: WikiPathwaysSync {
            search,
            get,
            downloaded_bytes: info.downloaded_bytes,
        },
    })
}
/// Search only native titles in original catalog order, with exact source species admission.
pub fn search(
    leg: &Leg<'_>,
    text: &str,
) -> Result<(Mapped<Vec<PathwayHeader>>, BundleInfo), RequestError> {
    let (rows, info) = leg.stored_set_rows(&CATALOGS, Find::Exact(KeyKind::Band, "catalog"))?;
    let needle = text.to_lowercase();
    let mut mapped = Mapped {
        refusals: crate::Refusal::None,
        value: Vec::new(),
        problems: Vec::new(),
    };
    for row in rows.into_iter().filter(|r| r.part.as_ref() == "search") {
        mapped
            .problems
            .extend(row.value.problems.into_iter().flatten());
        let Some(row) = row.value.native else {
            mapped.refusals = crate::Refusal::Refused;
            continue;
        };
        if row.species.as_ref() == "Homo sapiens"
            && (needle.is_empty()
                || row
                    .name
                    .as_ref()
                    .is_some_and(|n| n.as_str().to_lowercase().contains(&needle)))
        {
            mapped.value.push(PathwayHeader {
                id: PathwayId::Wikipathways(row.id),
                name: row.name,
                species: PathwaySpecies::Human,
                version: None,
                revision: row.revision,
            });
        }
    }
    Ok((mapped, info))
}
/// Read an exact native human identity from the stored get catalog without source requests.
pub fn card(
    leg: &Leg<'_>,
    id: &WikiPathwaysId,
    summary: bool,
) -> Result<Mapped<Option<PathwayCard>>, RequestError> {
    let (rows, _) = leg.stored_set_rows(&CATALOGS, Find::Exact(KeyKind::Id, &id.to_string()))?;
    let mut mapped = Mapped {
        refusals: crate::Refusal::None,
        value: None,
        problems: Vec::new(),
    };
    for row in rows.into_iter().filter(|r| r.part.as_ref() == "get") {
        mapped
            .problems
            .extend(row.value.problems.into_iter().flatten());
        let Some(row) = row.value.native else {
            mapped.refusals = crate::Refusal::Refused;
            continue;
        };
        if row.id != *id || row.species.as_ref() != "Homo sapiens" {
            continue;
        }
        mapped.value = Some(PathwayCard {
            pathway: Pathway {
                id: PathwayId::Wikipathways(row.id),
                name: row.name,
                species: PathwaySpecies::Human,
                description: summary.then_some(row.description).flatten(),
                summations: None,
                references: None,
                cited_in: row.cited_in,
                version: None,
                revision: row.revision,
            },
            release: None,
            description_truncated: summary.then_some(true),
            sections: vec![PathwaySection::Summary, PathwaySection::Genes].into(),
            genes: None,
            events: None,
        });
    }
    Ok(mapped)
}

/// Inspect this source's validated local catalog without reading rows or synchronizing.
pub(crate) fn bundle_state(leg: &Leg<'_>) -> Result<biomcp_types::LocalBundleState, RequestError> {
    CATALOGS.state(leg)
}
