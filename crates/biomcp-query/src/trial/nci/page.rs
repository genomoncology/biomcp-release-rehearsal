//! Intact native records page by source position within an immutable NCI snapshot.
use super::get::Snapshot;
use crate::Outcome;
use biodata::Text;
use biomcp_fetch::RequestError;
use biomcp_types::{
    NciEligibilityPage, NciTrialItem, NciTrialSections, Request, Section, Source, StepKind,
    TrialGet, TrialSection, TrialSource,
};
const SECTION_BYTES: usize = 6000;
pub(super) fn valid(q: &TrialGet, s: &Snapshot) -> bool {
    let n = match q.sections().first() {
        Some(TrialSection::Eligibility) => s
            .detail
            .eligibility
            .as_ref()
            .map_or(0, |e| e.unstructured.len()),
        Some(TrialSection::Outcomes) => s.detail.outcomes.len(),
        Some(TrialSection::Arms) => s.detail.arms.len(),
        Some(TrialSection::Diseases) => s.detail.diseases.len(),
        Some(TrialSection::Summary) => s.detail.summary.as_ref().map_or(0, |s| s.chars().count()),
        _ => 0,
    };
    q.offset() as usize <= n
}
pub(super) fn render(
    q: &TrialGet,
    s: Snapshot,
    pool: Option<Text>,
    out: &mut Outcome,
) -> Result<NciTrialItem, RequestError> {
    let requested =
        |section| q.sections().contains(&section) || q.sections().contains(&TrialSection::All);
    let mut sections = NciTrialSections {
        summary: None,
        eligibility: None,
        outcomes: None,
        arms: None,
        diseases: None,
    };
    let id = s.detail.card.nct_id;
    out.problems.extend(s.detail.problems.clone());
    if let Some(problem) = super::phase_problem(&s.detail.card) {
        out.problems.push(problem);
    }
    if requested(TrialSection::Summary) {
        sections.summary = Some(summary(q, &s, pool.as_ref(), out));
    }
    if requested(TrialSection::Eligibility) {
        sections.eligibility = Some(eligibility(q, &s, pool.as_ref(), out)?);
    }
    if requested(TrialSection::Outcomes) {
        sections.outcomes = Some(section(records(
            q,
            &s.detail.outcomes,
            TrialSection::Outcomes,
            id,
            pool.as_ref(),
            out,
            serde_json::to_vec,
        )?));
    }
    if requested(TrialSection::Arms) {
        sections.arms = Some(section(records(
            q,
            &s.detail.arms,
            TrialSection::Arms,
            id,
            pool.as_ref(),
            out,
            serde_json::to_vec,
        )?));
    }
    if requested(TrialSection::Diseases) {
        sections.diseases = Some(section(records(
            q,
            &s.detail.diseases,
            TrialSection::Diseases,
            id,
            pool.as_ref(),
            out,
            serde_json::to_vec,
        )?));
    }
    let more = omitted(q, id, out);
    let unsupported: Box<[_]> = q
        .sections()
        .iter()
        .filter(|s| **s != TrialSection::All && !TrialSource::Nci.sections().contains(s))
        .copied()
        .collect();
    let incomplete = s.detail.refusals == biomcp_adapters::Refusal::Refused
        || !unsupported.is_empty()
        || s.sources.iter().any(|s| {
            matches!(
                s.state,
                biomcp_types::SourceState::Stale | biomcp_types::SourceState::Failed
            )
        });
    let state = crate::status::item_status(true, false, incomplete);
    Ok(NciTrialItem {
        input: id,
        status: state,
        card: Some(s.detail.card),
        fetched: s.fetched,
        sections: (!q.sections().is_empty()).then_some(sections),
        more,
        unsupported_sections: unsupported,
    })
}
fn records<T: Clone>(
    q: &TrialGet,
    rows: &[T],
    section: TrialSection,
    id: biodata::NctId,
    pool: Option<&Text>,
    out: &mut Outcome,
    size: impl Fn(&T) -> Result<Vec<u8>, serde_json::Error>,
) -> Result<biomcp_types::TrialRecords<T>, RequestError> {
    let mut page = crate::trial::get::page::records(q, rows, SECTION_BYTES, size)?;
    crate::trial::get::page::continue_records(&mut page, q, id, section, pool);
    crate::trial::get::page::advance(
        q,
        id,
        section,
        pool,
        page.offset + page.items.len() as u32,
        page.total,
        out,
    );
    Ok(page)
}
fn section<T>(page: biomcp_types::TrialRecords<T>) -> Section<biomcp_types::TrialRecords<T>> {
    if page.total == 0 {
        Section::Empty {
            source: Source::Nci,
            record: None,
        }
    } else {
        Section::Data {
            source: Source::Nci,
            data: page,
        }
    }
}

fn summary(
    q: &TrialGet,
    s: &Snapshot,
    pool: Option<&Text>,
    out: &mut Outcome,
) -> Section<biomcp_types::NciTextPage> {
    let Some(text) = s.detail.summary.as_ref() else {
        return Section::Empty {
            source: Source::Nci,
            record: None,
        };
    };
    let mut fragment = String::new();
    let mut bytes = 0;
    for ch in text.chars().skip(q.offset() as usize) {
        let size = biomcp_adapters::jats::escaped_bytes(&ch.to_string());
        if bytes + size > SECTION_BYTES {
            break;
        }
        bytes += size;
        fragment.push(ch);
    }
    let total = text.chars().count() as u32;
    crate::trial::get::page::advance(
        q,
        s.detail.card.nct_id,
        TrialSection::Summary,
        pool,
        q.offset() + fragment.chars().count() as u32,
        total,
        out,
    );
    Section::Data {
        source: Source::Nci,
        data: biomcp_types::NciTextPage {
            continuation: crate::trial::get::page::continuation(
                q,
                s.detail.card.nct_id,
                TrialSection::Summary,
                pool,
                q.offset() + fragment.chars().count() as u32,
                total,
            ),
            text: fragment.into(),
            offset: q.offset(),
            total_characters: total,
        },
    }
}

fn eligibility(
    q: &TrialGet,
    s: &Snapshot,
    pool: Option<&Text>,
    out: &mut Outcome,
) -> Result<Section<NciEligibilityPage>, RequestError> {
    Ok(if let Some(e) = &s.detail.eligibility {
        let records = records(
            q,
            &e.unstructured,
            TrialSection::Eligibility,
            s.detail.card.nct_id,
            pool,
            out,
            serde_json::to_vec,
        )?;
        Section::Data {
            source: Source::Nci,
            data: NciEligibilityPage {
                structured: e.structured.clone(),
                unstructured: records,
            },
        }
    } else {
        Section::Empty {
            source: Source::Nci,
            record: None,
        }
    })
}

fn omitted(q: &TrialGet, id: biodata::NctId, out: &mut Outcome) -> Box<[TrialSection]> {
    let more: Box<[_]> = biomcp_types::TrialSource::Nci
        .sections()
        .iter()
        .copied()
        .filter(|s| !q.sections().contains(s) && !q.sections().contains(&TrialSection::All))
        .collect();
    for section in &more {
        out.next.push(crate::next::Step::run(
            StepKind::Section,
            &[biomcp_types::Source::Nci],
            Request::GetTrial(
                TrialGet::single(id, Some(*section), q.limit()).with_source(TrialSource::Nci),
            ),
            false,
        ));
    }
    more
}
