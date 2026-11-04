use super::{failure, identity};
use crate::Outcome;
use biomcp_adapters::{NativeWindow, chembl, opentargets};
use biomcp_fetch::{Leg, RequestError};
use biomcp_types::ReplyBody;
use biomcp_types::{
    DrugGet, DrugHeader, DrugIndications, DrugRecords, DrugSection, DrugTargets, PageInfo, Request,
    Section, Source, StepKind,
};
#[path = "collection_page.rs"]
mod collection;
pub(super) fn chembl(
    leg: &Leg<'_>,
    query: &DrugGet,
    fresh: bool,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let source = Source::Chembl;
    let (row, metadata) = chembl::molecule(
        leg,
        &query
            .native_input()
            .map_err(|_| biomcp_fetch::InputError::Serialization)?,
    )?;
    if !matches!(
        query.sections(),
        [DrugSection::Targets] | [DrugSection::Indications]
    ) {
        chembl_card(row, metadata.fetched, query, fresh, out)?;
        return Ok(());
    }
    let header = identity::chembl_header(&row, &mut out.problems).map_err(|_| failure(source))?;
    let drug = biomcp_adapters::drug_sources::identity(&header).map_err(|_| failure(source))?;
    let limit = query.section_limit().unwrap_or(10);
    let offset = query.section_offset().unwrap_or(0);
    match query.sections() {
        [DrugSection::Targets] => {
            let (mut page, meta) = chembl::mechanisms(
                leg,
                &query
                    .native_input()
                    .map_err(|_| biomcp_fetch::InputError::Serialization)?,
                limit,
                offset,
            )?;
            let records = chembl::map_targets(&drug, std::mem::take(&mut page.window.rows))
                .map_err(|_| failure(source))?;
            out.problems.extend(page.problems);
            out.problems.extend(records.problems);
            page.window.include_refusals(records.refusals);
            targets(
                out,
                query,
                header,
                page.window.with_rows(records.value.into_vec()),
                meta.fetched,
                fresh,
            );
        }
        [DrugSection::Indications] => {
            let (mut page, meta) = chembl::indications(
                leg,
                &query
                    .native_input()
                    .map_err(|_| biomcp_fetch::InputError::Serialization)?,
                limit,
                offset,
            )?;
            let records = chembl::map_indications(&drug, std::mem::take(&mut page.window.rows))
                .map_err(|_| failure(source))?;
            out.problems.extend(page.problems);
            out.problems.extend(records.problems);
            page.window.include_refusals(records.refusals);
            indications(
                out,
                query,
                header,
                page.window.with_rows(records.value.into_vec()),
                meta.fetched,
                fresh,
            );
        }
        _ => return Ok(()),
    }
    Ok(())
}
pub(super) fn opentargets(
    leg: &Leg<'_>,
    query: &DrugGet,
    fresh: bool,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let (row, meta) = opentargets::drug(
        leg,
        &query
            .native_input()
            .map_err(|_| biomcp_fetch::InputError::Serialization)?,
    )?;
    let Some(row) = row else {
        crate::status::item(out, false, false, false);
        return Ok(());
    };
    let source = Source::Opentargets;
    let header = opentargets::header(&row).map_err(|_| failure(source))?;
    out.problems.extend(header.problems);
    out.include_refusals(header.refusals);
    let header = header.value;
    let drug = biomcp_adapters::drug_sources::identity(&header).map_err(|_| failure(source))?;
    let limit = query.section_limit().unwrap_or(10) as usize;
    let offset = query.section_offset().unwrap_or(0) as usize;
    match query.sections() {
        [DrugSection::Targets] => {
            let mut collection =
                opentargets::map_targets(&drug, row.mechanisms()).map_err(|_| failure(source))?;
            out.problems.append(&mut collection.problems);
            out.include_refusals(collection.refusals);
            let window = collection::page(&mut collection, offset, limit);
            targets(out, query, header, window, meta.fetched, fresh);
        }
        [DrugSection::Indications] => {
            let mut collection = opentargets::map_indications(&drug, row.indications())
                .map_err(|_| failure(source))?;
            out.problems.append(&mut collection.problems);
            out.include_refusals(collection.refusals);
            let window = collection::page(&mut collection, offset, limit);
            indications(out, query, header, window, meta.fetched, fresh);
        }
        _ => {
            crate::status::item(out, true, false, false);
            out.body = (opentargets::card(header, meta.fetched)).with_query(query.clone());
            card_next(out, query, fresh, false);
        }
    }
    Ok(())
}

fn card_next(out: &mut Outcome, query: &DrugGet, fresh: bool, xrefs: bool) {
    for section in [
        DrugSection::Targets,
        DrugSection::Indications,
        DrugSection::Xrefs,
    ]
    .into_iter()
    .filter(|s| *s != DrugSection::Xrefs || xrefs)
    {
        if let Some(get) = crate::next::checked(
            query.for_section(section),
            query.source().source(),
            &mut out.problems,
        ) {
            out.next.push(crate::next::Step::run(
                StepKind::Section,
                &[get.source().source()],
                Request::GetDrug(get),
                fresh,
            ));
        }
    }
}
fn page<T>(
    out: &mut Outcome,
    query: &DrugGet,
    window: NativeWindow<T>,
    fetched: Option<i64>,
    fresh: bool,
) -> Section<DrugRecords<T>> {
    out.include_refusals(window.refusals());
    let offset = window.offset();
    let total = window.total().unwrap_or(0);
    let next_offset = window.next_offset();
    let records = window.rows.into_boxed_slice();
    let returned = records.len() as u32;
    let source = query.source().source();
    {
        let found = total != 0;
        crate::status::item(out, found, false, false);
    };
    out.page = Some(PageInfo {
        offset,
        returned,
        total,
        pool: None,
        truncated: None,
    });
    if let Some(offset) = next_offset
        && let Some(get) = crate::next::checked(
            query.at_section_offset(offset),
            query.source().source(),
            &mut out.problems,
        )
    {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &[get.source().source()],
            Request::GetDrug(get),
            fresh,
        ));
    }
    if total == 0 {
        Section::Empty {
            source,
            record: query
                .input()
                .to_string()
                .parse()
                .ok()
                .map(|id| biomcp_types::SourceRecord {
                    namespace: biodata::IdentifierNamespace::Unknown,
                    id,
                    url: None,
                }),
        }
    } else {
        Section::Data {
            source,
            data: DrugRecords {
                records,
                offset,
                total: Some(total),
                truncated: None,
                version: None,
                retrieved: biomcp_adapters::drug_sources::received_date(fetched),
            },
        }
    }
}
fn targets(
    out: &mut Outcome,
    query: &DrugGet,
    identity: DrugHeader,
    window: NativeWindow<biodata::DrugTargetRelation>,
    fetched: Option<i64>,
    fresh: bool,
) {
    let targets = page(out, query, window, fetched, fresh);
    out.body = (DrugTargets { identity, targets }).with_query(query.clone());
}
fn indications(
    out: &mut Outcome,
    query: &DrugGet,
    identity: DrugHeader,
    window: NativeWindow<biodata::DrugIndicationClaim>,
    fetched: Option<i64>,
    fresh: bool,
) {
    let indications = page(out, query, window, fetched, fresh);
    out.body = (DrugIndications {
        identity,
        indications,
    })
    .with_query(query.clone());
}

fn chembl_card(
    row: chembl::Molecule,
    fetched: Option<i64>,
    query: &DrugGet,
    fresh: bool,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let source = Source::Chembl;
    let limit = query.section_limit().unwrap_or(10);
    let offset = query.section_offset().unwrap_or(0);
    let mut card = identity::chembl_card(row, fetched, &mut out.dropped, &mut out.problems)
        .map_err(|_| failure(source))?;
    let has_xrefs = !card.cross_references.is_empty();
    let xrefs = std::mem::take(&mut card.cross_references);
    if query.sections() == [DrugSection::Xrefs] {
        let total = xrefs.len() as u64;
        let records = xrefs
            .into_vec()
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();
        let xrefs = page(
            out,
            query,
            NativeWindow::new(
                records,
                offset,
                limit,
                Some(total),
                total
                    .saturating_sub(u64::from(offset))
                    .min(u64::from(limit)) as u32,
                biomcp_adapters::Refusal::None,
            ),
            fetched,
            fresh,
        );
        out.body = (biomcp_types::DrugXrefs {
            identity: card.header,
            xrefs,
        })
        .with_query(query.clone());
        return Ok(());
    }
    crate::status::item(out, true, false, false);
    out.body = card.with_query(query.clone());
    card_next(out, query, fresh, has_xrefs);
    if let Some(get) = crate::next::checked(
        DrugGet::new(
            query
                .native_input()
                .map_err(|_| biomcp_fetch::InputError::Serialization)?,
            biomcp_types::DrugSource::Opentargets,
        ),
        query.source().source(),
        &mut out.problems,
    ) {
        out.next.push(crate::next::Step::run(
            StepKind::Section,
            &[get.source().source()],
            Request::GetDrug(get),
            fresh,
        ));
    }
    Ok(())
}
