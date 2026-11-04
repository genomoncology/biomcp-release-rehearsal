use super::raw;
use biodata::{CellLineEntryDates, CellLineWarning, Text};
use biomcp_fetch::{BadReply, InputError, RequestError};
use biomcp_types::{
    Capped, CellLineCard, CellLineHeader, CellLineInput, CellLineSection, CellLineSections, Status,
};
pub(super) fn capped<T>(rows: Vec<T>, limit: usize) -> Capped<T> {
    let total = rows.len() as u64;
    Capped {
        items: rows.into_iter().take(limit).collect(),
        total,
    }
}
// Every header/card request explicitly asks for species and diseases. A missing native
// collection therefore means source omission, unlike unrequested xrefs or variations.
pub(super) fn header(row: raw::Record) -> Result<CellLineHeader, RequestError> {
    Ok(CellLineHeader {
        accession: row
            .accession()
            .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::Cellosaurus, None))?
            .clone(),
        name: row.name().cloned(),
        category: row.category,
        species: capped(row.species_list.unwrap_or_default().0.into_vec(), 3),
        diseases: capped(row.disease_list.unwrap_or_default().0.into_vec(), 3),
    })
}
pub(super) fn card(row: raw::Record) -> Result<CellLineCard, RequestError> {
    let accession = row
        .accession()
        .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::Cellosaurus, None))?
        .clone();
    let name = row.name().cloned();
    let warnings = warnings(row.comment_list.as_deref().unwrap_or_default());
    let version = row
        .entry_version
        .map(|v| v.as_str().parse().map_err(|_| BadReply))
        .transpose()
        .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::Cellosaurus, None))?;
    Ok(CellLineCard {
        input: CellLineInput::Accession(accession.clone()),
        status: Status::Complete,
        header: Some(CellLineHeader {
            accession: accession.clone(),
            name,
            category: row.category,
            species: capped(row.species_list.unwrap_or_default().0.into_vec(), 3),
            diseases: capped(row.disease_list.unwrap_or_default().0.into_vec(), 3),
        }),
        candidates: Box::default(),
        dates: Some(CellLineEntryDates {
            created: row.created,
            updated: row.last_updated,
            version,
        }),
        age: row.age,
        sex: row.sex,
        warnings: Some(capped(warnings, 3)),
        url: Some(
            Text::new(&format!("https://www.cellosaurus.org/{accession}"))
                .map_err(|_| InputError::Serialization)?,
        ),
        sections: CellLineSections {
            names: None,
            identity: None,
            xrefs: None,
            variants: None,
            chembl: None,
        },
        more: [
            CellLineSection::Names,
            CellLineSection::Variants,
            CellLineSection::Xrefs,
            CellLineSection::Chembl,
            CellLineSection::Identity,
        ]
        .into(),
        release: None,
        fetched: None,
    })
}

pub(super) fn warnings(rows: &[raw::Comment]) -> Vec<CellLineWarning> {
    rows.iter()
        .filter(|r| matches!(r.category.as_str(), "Problematic cell line" | "Caution"))
        .map(|r| CellLineWarning {
            scope: r.category.clone(),
            text: r.value.clone(),
        })
        .collect()
}
