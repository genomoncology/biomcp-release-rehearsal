//! Independent source-backed sample and mutation row sections.
use super::{continuation, finish};
use crate::{Ctx, Outcome};
use biomcp_adapters::cbioportal::{SPEC, studies};
use biomcp_fetch::{Deadline, Leg, RequestError};
use biomcp_types::{
    PageInfo, ReplyBody, ReplyData, Section, Source, StudyGet, StudyMutations, StudySamples,
    StudySection,
};

pub(super) fn section(cx: &Ctx<'_>, q: &StudyGet, deadline: Deadline<'_>, out: &mut Outcome) {
    for section in [StudySection::Samples, StudySection::Mutations] {
        if q.section() != Some(section) && q.section() != Some(StudySection::All) {
            continue;
        }
        if section == StudySection::Mutations && q.gene().is_none() {
            continue;
        }
        let query = q.with_section(section);
        let query = if q.section() == Some(section) {
            q.clone()
        } else {
            query
        };
        let leg = cx.fetch.leg(&SPEC, deadline).fresh(cx.fresh);
        let result = page(&leg, &query, cx, out);
        let error = result.err();
        if let Some(error) = error
            && let ReplyData::StudyCard {
                body: Some(card), ..
            } = &mut out.body
        {
            let reason = crate::status::reason(error).unwrap_or(biomcp_types::Reason::BadReply);
            match section {
                StudySection::Samples => {
                    card.samples = Some(Section::unavailable(Source::Cbioportal, reason))
                }
                StudySection::Mutations => {
                    card.mutations = Some(Section::unavailable(Source::Cbioportal, reason))
                }
                _ => {}
            }
        }
        let partial = out.status() == biomcp_types::Status::Partial;
        finish(out, leg, error);
        if q.section() == Some(StudySection::All) && (error.is_some() || partial) {
            crate::status::item(out, true, false, true);
        }
    }
}
fn page(leg: &Leg<'_>, q: &StudyGet, cx: &Ctx<'_>, out: &mut Outcome) -> Result<(), RequestError> {
    if q.section() == Some(StudySection::Samples) {
        let rows = studies::samples(leg, q)?;
        let page = page_info(&rows.window);
        let offset = rows.window.next_offset();
        out.include_refusals(rows.window.refusals());
        out.problems.extend(rows.problems);
        let data = StudySamples {
            study_id: q.id().clone(),
            items: rows.window.rows.into(),
            page: page.clone(),
            acquired: rows.acquired,
        };
        if let ReplyData::StudyCard {
            body: Some(card), ..
        } = &mut out.body
        {
            card.samples = Some(Section::Data {
                source: Source::Cbioportal,
                data,
            });
        } else {
            out.body = data.with_query(q.clone());
            out.page = Some(page);
        }
        continuation(out, cx, q, rows.rejected, offset);
    } else {
        let (rows, profile, gene) = studies::mutations(leg, q)?;
        let page = page_info(&rows.window);
        let offset = rows.window.next_offset();
        out.include_refusals(rows.window.refusals());
        out.problems.extend(rows.problems);
        let query = q.with_molecular_profile(profile.clone());
        let data = StudyMutations {
            study_id: q.id().clone(),
            molecular_profile_id: profile,
            entrez_gene_id: gene,
            items: rows.window.rows.into(),
            page: page.clone(),
            acquired: rows.acquired,
        };
        if let ReplyData::StudyCard {
            body: Some(card), ..
        } = &mut out.body
        {
            card.mutations = Some(Section::Data {
                source: Source::Cbioportal,
                data,
            });
        } else {
            out.body = data.with_query(q.clone());
            out.page = Some(page);
        }
        continuation(out, cx, &query, rows.rejected, offset);
    }
    Ok(())
}
fn page_info<T>(window: &biomcp_adapters::NativeWindow<T>) -> PageInfo {
    PageInfo {
        offset: window.offset(),
        returned: window.rows.len() as u32,
        total: window.total().unwrap_or(0),
        pool: None,
        truncated: None,
    }
}
