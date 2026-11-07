//! Independent FDA facts beside Open Targets phases; no indication correspondence is inferred.
use crate::{Ctx, Outcome, status};
use biomcp_adapters::{drugsfda, received_date};
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{
    DrugApprovals, DrugGet, DrugRecords, DrugRegulatoryContext, ReplyData, Section, Source,
    SourceNotice,
};
const NOTICE: &str = concat!(
    "FDA application events are matched by drug name only. ",
    "Indications are not linked to the Open Targets phase facts. ",
    "Neither source is labeled as superseding the other; unknown source dates remain unknown."
);

pub(super) fn attach(cx: &Ctx<'_>, query: &DrugGet, deadline: Deadline<'_>, out: &mut Outcome) {
    let Some(name) = (match &out.body {
        ReplyData::DrugIndications {
            body: Some(body), ..
        } if matches!(&body.indications, Section::Data { data, .. } if !data.records.is_empty()) => {
            body.identity.name.clone()
        }
        _ => None,
    }) else {
        return;
    };
    let limit = query.section_limit().unwrap_or(10);
    let leg = cx.fetch.leg(&drugsfda::SPEC, deadline).fresh(cx.fresh);
    let reply = drugsfda::search(&leg, &name, limit, 0);
    let error = reply.as_ref().err().copied();
    let mut source = status::source(Source::Drugsfda, leg.finish(), error);
    source.notices.push(SourceNotice::RequestGuidance {
        data_date: None,
        message: NOTICE.into(),
        retry_after_seconds: None,
    });
    match reply {
        Ok(page) => {
            source.count = Some(page.total);
            source.retrieved_count = Some(page.records.len() as u32);
            out.problems.extend(page.problems.iter().cloned());
            out.include_refusals(page.refusals);
            match context(page, &name, limit) {
                Ok(context) => {
                    if let ReplyData::DrugIndications {
                        body: Some(body), ..
                    } = &mut out.body
                    {
                        body.regulatory_context = Some(context);
                    }
                }
                Err(error) => {
                    out.local = status::local(error);
                }
            }
        }
        Err(error) => out.local = out.local.or_else(|| status::local(error)),
    }
    out.sources.push(source);
    status::item(out, true, false, false);
}
fn context(
    page: drugsfda::Page,
    name: &biodata::Text,
    limit: u32,
) -> Result<DrugRegulatoryContext, RequestError> {
    let mut remaining = limit;
    let mut applications = Vec::new();
    for row in &page.records {
        if row.name_rank(name.as_str()).is_none() {
            continue;
        }
        let records = row.submissions(0, remaining)?;
        remaining = remaining.saturating_sub(records.len() as u32);
        let total = row.submission_count() as u64;
        let truncated = (records.len() as u64) < total;
        applications.push(DrugApprovals {
            identity: row.header(),
            products: row.products(),
            source_updated: page.updated.map(Into::into),
            order: DrugApprovals::ORDER.into(),
            approvals: Section::Data {
                source: Source::Drugsfda,
                data: DrugRecords {
                    records,
                    offset: 0,
                    total: Some(total),
                    truncated: Some(truncated),
                    version: None,
                    retrieved: received_date(page.metadata.fetched),
                },
            },
        });
    }
    Ok(DrugRegulatoryContext {
        indications_linked: false,
        notice: NOTICE.into(),
        total_applications: page.total,
        applications: applications.into(),
    })
}
