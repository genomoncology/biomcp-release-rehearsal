//! Explicit publication study results, preserving native sample descriptions.
use crate::{Ctx, Outcome, status};
use biomcp_adapters::gwas;
use biomcp_types::{GwasSearch, GwasStudyPage, PageInfo, Paged, ReplyBody, Source, StepKind};
pub(super) fn search(cx: &Ctx<'_>, q: &GwasSearch) -> Outcome {
    let mut out = status::empty(GwasStudyPage::into_reply(q.clone(), None));
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&gwas::SPEC, deadline).fresh(cx.fresh);
    let result = gwas::studies(&leg, q);
    let error = result.as_ref().err().copied();
    let mut source = status::source(Source::Gwas, leg.finish(), error);
    match result {
        Ok(rows) => {
            out.include_refusals(rows.refusals);
            out.dropped |= rows.items.iter().any(|row| row.association_count.is_none());
            out.problems.extend(rows.problems);
            source.retrieved_count = Some(rows.items.len() as u32);
            source.count = Some(rows.total);
            let start = q.offset() - q.offset() % q.limit();
            let continuation = (u64::from(start) + u64::from(rows.received) < rows.total)
                .then(|| q.at_offset(start.saturating_add(rows.received)));
            let page = PageInfo {
                offset: q.offset(),
                returned: rows.items.len() as u32,
                total: rows.total,
                pool: None,
                truncated: None,
            };
            out.page = Some(page.clone());
            if let Some(query) = &continuation {
                out.next.push(crate::next::Step::run(
                    StepKind::Page,
                    &[Source::Gwas],
                    biomcp_types::Request::SearchGwas(query.clone()),
                    cx.fresh,
                ));
            }
            for row in rows.items.iter().take(3) {
                let get = biomcp_types::GwasGet::Study {
                    id: row.study.id.clone(),
                    section: None,
                    page: Default::default(),
                };
                out.next.push(crate::next::Step::run(
                    StepKind::Pivot,
                    &[Source::Gwas],
                    get.into(),
                    false,
                ));
            }
            out.body = GwasStudyPage {
                items: rows.items.into(),
                page,
                continuation,
            }
            .with_query(q.clone());
            status::item(
                &mut out,
                true,
                false,
                source.state == biomcp_types::SourceState::Stale,
            );
        }
        Err(error) => out.local = status::local(error),
    }
    out.sources.push(source);
    status::search(&mut out);
    command.finish(deadline.remaining());
    out
}
