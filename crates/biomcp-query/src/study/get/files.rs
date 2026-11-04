//! File references share the command deadline and retain independent section outcomes.
use super::{finish, next};
use crate::{Ctx, Outcome};
use biomcp_adapters::cbioportal::{SPEC, studies};
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::ReplyBody;
use biomcp_types::{
    PageInfo, ReplyData, Section, Source, Status, StepKind, StudyFiles, StudyGet, StudySection,
};
pub(super) fn section(cx: &Ctx<'_>, q: &StudyGet, deadline: Deadline<'_>, out: &mut Outcome) {
    let on_card = matches!(out.body, ReplyData::StudyCard { body: Some(_), .. });
    if !matches!(q.section(), Some(StudySection::Files | StudySection::All)) {
        return;
    }
    let leg = cx.fetch.leg(&SPEC, deadline).fresh(cx.fresh);
    let result = studies::files(&leg, q.id(), q.section_offset(), q.section_limit())
        .and_then(|directory| apply(cx, q, out, directory));
    let error = result.err();
    if let Some(error) = error
        && let ReplyData::StudyCard {
            body: Some(card), ..
        } = &mut out.body
    {
        let reason = crate::status::reason(error).unwrap_or(biomcp_types::Reason::BadReply);
        card.files = Some(Section::unavailable(Source::Cbioportal, reason));
    }
    let partial = out.status() == Status::Partial;
    finish(out, leg, error);
    if on_card && (error.is_some() || partial) {
        crate::status::item(out, true, false, true);
    }
}
fn apply(
    cx: &Ctx<'_>,
    q: &StudyGet,
    out: &mut Outcome,
    directory: studies::Directory,
) -> Result<(), RequestError> {
    let Some(window) = directory.window else {
        if let ReplyData::StudyCard {
            body: Some(card), ..
        } = &mut out.body
        {
            card.files = Some(Section::Empty {
                source: Source::Cbioportal,
                record: None,
            });
        } else {
            crate::status::item(out, false, false, false);
        }
        return Ok(());
    };
    let query = if q.section() == Some(StudySection::All) {
        q.with_section(StudySection::Files)
    } else {
        q.clone()
    };
    let next_offset = window.next_offset();
    let bounds = PageInfo {
        offset: window.offset(),
        returned: window.rows.len() as u32,
        total: window.total().unwrap_or(0),
        pool: None,
        truncated: None,
    };
    let items = window.rows.into();
    let data = StudyFiles {
        study_id: q.id().clone(),
        items,
        page: bounds.clone(),
        acquired: directory.acquired,
        license_notice: directory.license_notice,
    };
    if let ReplyData::StudyCard {
        body: Some(card), ..
    } = &mut out.body
    {
        card.files = Some(Section::Data {
            source: Source::Cbioportal,
            data,
        });
    } else {
        if q.section() != Some(StudySection::Files) {
            return Err(biomcp_fetch::bad_reply(Source::Cbioportal, None));
        }
        out.body = data.with_query(q.clone());
        out.page = Some(bounds.clone());
        crate::status::item(out, true, false, false);
    }
    if let Some(offset) = next_offset {
        let query = query
            .at_section_offset(offset)
            .map_err(|_| biomcp_fetch::bad_reply(Source::Cbioportal, None))?;
        next(out, cx, StepKind::Page, query);
    }
    Ok(())
}
