mod eligibility;
mod publications;
mod results;
mod sites;
use super::Snapshot;
use crate::Outcome;
use biodata::{NctId, Text};

use biomcp_fetch::{CacheError, RequestError};
use biomcp_types::{
    Request, Section, Source, SourceRecord, StepKind, TrialArmRecord, TrialCardItem,
    TrialEligibilityPage, TrialGet, TrialRecords, TrialSection, TrialSections,
};

pub(super) fn valid(query: &TrialGet, snapshot: &Snapshot) -> bool {
    let d = &snapshot.detail;
    let total = match query.sections().first() {
        Some(TrialSection::Eligibility) => d
            .eligibility
            .as_ref()
            .and_then(|e| e.criteria.as_ref())
            .map_or(0, |s| s.as_str().chars().count()),
        Some(TrialSection::Outcomes) => d.outcomes.len(),
        Some(TrialSection::Results) => d.results.len(),
        Some(TrialSection::Arms) => d.arms.len() + d.interventions.len(),
        Some(TrialSection::References) => d.references.len(),
        Some(TrialSection::Documents) => d.documents.len(),
        Some(TrialSection::Locations) => d.locations.len(),
        Some(TrialSection::Contacts) => d.contacts.len() + d.officials.len(),
        _ => 0,
    };
    query.offset() as usize <= total
}
pub(super) fn render(
    query: &TrialGet,
    snapshot: Snapshot,
    pool: Option<Text>,
    out: &mut Outcome,
) -> Result<TrialCardItem, RequestError> {
    let requested = |section| {
        query.sections().contains(&section) || query.sections().contains(&TrialSection::All)
    };
    let id = snapshot.detail.card.nct_id;
    let mut sections = TrialSections::default();
    if requested(TrialSection::Eligibility) {
        sections.eligibility = Some(eligibility(query, &snapshot, pool.as_ref(), out));
    }
    results::render(query, &snapshot, pool.as_ref(), &mut sections, out)?;
    records_sections(query, &snapshot, pool.as_ref(), &mut sections, out)?;
    sites::render(query, &snapshot, pool.as_ref(), &mut sections, out)?;
    publications::render(query, &snapshot, pool.as_ref(), &mut sections, out)?;
    for section in biomcp_types::TrialSource::Ctgov
        .sections()
        .iter()
        .copied()
        .filter(|s| !requested(*s))
    {
        out.next.push(crate::next::Step::run(
            StepKind::Section,
            &[query.source().source()],
            Request::GetTrial(TrialGet::single(id, Some(section), query.limit())),
            false,
        ));
    }
    out.problems
        .extend(snapshot.detail.problems.iter().cloned());
    for problem in &snapshot.processing_problems {
        if !out.problems.contains(problem) {
            out.problems.push(problem.clone());
        }
    }
    Ok(TrialCardItem {
        input: id,
        status: if super::partial(&snapshot) {
            crate::status::item_status(true, false, true)
        } else {
            crate::status::item_status(true, false, false)
        },
        card: Some(biomcp_types::TrialPostedCard {
            protocol: snapshot.detail.card,
            results_first_posted: snapshot.detail.results_first_posted,
        }),
        fetched: snapshot.fetched,
        sections: (!query.sections().is_empty()).then_some(sections),
        more: biomcp_types::TrialSource::Ctgov
            .sections()
            .iter()
            .copied()
            .filter(|s| !requested(*s))
            .collect(),
    })
}
fn eligibility(
    query: &TrialGet,
    snapshot: &Snapshot,
    pool: Option<&Text>,
    out: &mut Outcome,
) -> Section<TrialEligibilityPage> {
    let id = snapshot.detail.card.nct_id;
    let Some(e) = snapshot.detail.eligibility.as_ref() else {
        return Section::Empty {
            source: Source::ClinicalTrialsGov,
            record: record(id),
        };
    };
    let text = e.criteria.as_ref().map_or("", |s| s.as_str());
    let total = text.chars().count() as u32;
    let (fragment, criterion_continues) = eligibility::fragment(text, query.offset() as usize);
    let end = query.offset() + fragment.chars().count() as u32;
    advance(query, id, TrialSection::Eligibility, pool, end, total, out);
    Section::Data {
        source: Source::ClinicalTrialsGov,
        data: TrialEligibilityPage {
            continuation: continuation(query, id, TrialSection::Eligibility, pool, end, total),
            minimum_age: e.minimum_age.clone(),
            maximum_age: e.maximum_age.clone(),
            sex: e.sex,
            healthy_volunteers: e.healthy_volunteers,
            criteria: fragment.into(),
            criterion_continues,
            offset: query.offset(),
            total_characters: total,
        },
    }
}
pub(in crate::trial) fn records<T: Clone>(
    query: &TrialGet,
    rows: &[T],
    budget: usize,
    size: impl Fn(&T) -> Result<Vec<u8>, serde_json::Error>,
) -> Result<TrialRecords<T>, RequestError> {
    let mut items = Vec::new();
    let mut bytes = 0;
    for item in rows
        .iter()
        .skip(query.offset() as usize)
        .take(query.limit() as usize)
    {
        let size = size(item).map_err(|_| CacheError::Encoding)?.len() + 1;
        if !items.is_empty() && bytes + size > budget {
            break;
        }
        bytes += size;
        items.push(item.clone());
    }
    Ok(TrialRecords {
        continuation: None,
        items: items.into(),
        offset: query.offset(),
        total: rows.len() as u32,
    })
}
fn section<T>(page: TrialRecords<T>, id: NctId) -> Section<TrialRecords<T>> {
    if page.total == 0 {
        Section::Empty {
            source: Source::ClinicalTrialsGov,
            record: record(id),
        }
    } else {
        Section::Data {
            source: Source::ClinicalTrialsGov,
            data: page,
        }
    }
}
fn record(id: NctId) -> Option<SourceRecord> {
    Text::new(&id.to_string()).ok().map(|id| SourceRecord {
        namespace: biodata::IdentifierNamespace::Unknown,
        id,
        url: None,
    })
}
pub(in crate::trial) fn advance(
    query: &TrialGet,
    id: NctId,
    section: TrialSection,
    pool: Option<&Text>,
    end: u32,
    total: u32,
    out: &mut Outcome,
) {
    if end < total
        && let Some(pool) = pool
    {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &[query.source().source()],
            Request::GetTrial(query.page(id, section, pool.clone(), end)),
            false,
        ));
    }
}

fn records_sections(
    query: &TrialGet,
    snapshot: &Snapshot,
    pool: Option<&Text>,
    sections: &mut TrialSections,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let id = snapshot.detail.card.nct_id;
    let requested = |section| {
        query.sections().contains(&section) || query.sections().contains(&TrialSection::All)
    };
    if requested(TrialSection::Outcomes) {
        let items = &snapshot.detail.outcomes;
        let mut page = records(query, items, 8000, serde_json::to_vec)?;
        continue_records(&mut page, query, id, TrialSection::Outcomes, pool);
        advance(
            query,
            id,
            TrialSection::Outcomes,
            pool,
            page.offset + page.items.len() as u32,
            page.total,
            out,
        );
        sections.outcomes = Some(section(page, id));
    }
    if requested(TrialSection::Arms) {
        let rows: Vec<_> = snapshot
            .detail
            .arms
            .iter()
            .cloned()
            .map(TrialArmRecord::Arm)
            .chain(
                snapshot
                    .detail
                    .interventions
                    .iter()
                    .cloned()
                    .map(TrialArmRecord::Intervention),
            )
            .collect();
        let mut page = records(query, &rows, 4500, serde_json::to_vec)?;
        continue_records(&mut page, query, id, TrialSection::Arms, pool);
        advance(
            query,
            id,
            TrialSection::Arms,
            pool,
            page.offset + page.items.len() as u32,
            page.total,
            out,
        );
        sections.arms = Some(section(page, id));
    }
    Ok(())
}

pub(in crate::trial) fn continuation(
    query: &TrialGet,
    id: NctId,
    section: TrialSection,
    pool: Option<&Text>,
    end: u32,
    total: u32,
) -> Option<TrialGet> {
    pool.filter(|_| end < total)
        .map(|p| query.page(id, section, p.clone(), end))
}
pub(in crate::trial) fn continue_records<T>(
    page: &mut TrialRecords<T>,
    query: &TrialGet,
    id: NctId,
    section: TrialSection,
    pool: Option<&Text>,
) {
    page.continuation = continuation(
        query,
        id,
        section,
        pool,
        page.offset + page.items.len() as u32,
        page.total,
    );
}
