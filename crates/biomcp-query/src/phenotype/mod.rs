//! Literal source term discovery and independent exact term cards.
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
mod associations;
mod record;
mod sections;
use crate::{Ctx, Outcome, status};
use biomcp_adapters::monarch;
use biomcp_types::{
    PageInfo, PhenotypeGet, PhenotypePage, PhenotypeSearch, Request, Source, StepKind,
};
/// Read a native association page or one exact source term reply.
pub(crate) fn get(cx: &Ctx<'_>, q: &PhenotypeGet) -> Outcome {
    if matches!(
        q.section(),
        Some(biomcp_types::PhenotypeSection::Diseases | biomcp_types::PhenotypeSection::Genes)
    ) {
        associations::get(cx, q)
    } else {
        record::get(cx, q)
    }
}
/// Search one native HP page without source substitution or local selection.
pub(crate) fn search(cx: &Ctx<'_>, q: &PhenotypeSearch) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&monarch::SPEC, deadline).fresh(cx.fresh);
    let reply = monarch::phenotype::search(&leg, q);
    let error = reply.as_ref().err().copied();
    let mut out = empty(
        (PhenotypePage {
            items: Box::default(),
            components: Box::default(),
            fetched: None,
        })
        .with_query(q.clone()),
    );
    out.sources
        .push(status::source(Source::Monarch, leg.finish(), error));
    out.local = error.and_then(status::local);
    if let Ok(page) = reply {
        out.include_refusals(page.refusals);
        out.problems.extend(page.problems);
        let returned = page.items.len() as u32;
        for item in &page.items {
            let get = PhenotypeGet::new(item.id.clone());
            out.next.push(crate::next::Step::new(
                StepKind::Disambiguate,
                &[record::source(&get)],
                run(Request::GetPhenotype(get)),
            ));
        }
        if page.received > 0 && u64::from(q.offset()) + u64::from(page.received) < page.total {
            out.next.push(crate::next::Step::new(
                StepKind::Page,
                &[Source::Monarch],
                run(Request::SearchPhenotype(
                    q.at_offset(q.offset().saturating_add(page.received)),
                )),
            ));
        }
        out.page = Some(PageInfo {
            offset: q.offset(),
            returned,
            total: page.total,
            pool: None,
            truncated: None,
        });
        out.body = (PhenotypePage {
            items: page.items.into(),
            components: monarch::components().unwrap_or_default(),
            fetched: page
                .metadata
                .fetched
                .and_then(|value| u64::try_from(value).ok()),
        })
        .with_query(q.clone());
        crate::status::search(&mut out);
    }
    problem(&mut out, Source::Monarch, None);
    recovery(&mut out, Request::SearchPhenotype(q.clone()));
    command.finish(deadline.remaining());
    out
}
use crate::next::run;
pub(super) use crate::status::empty;
fn problem(out: &mut Outcome, source: Source, record: Option<biodata::Text>) {
    if out
        .sources
        .iter()
        .any(|s| s.reason == Some(biomcp_types::Reason::BadReply))
    {
        out.problems.push(biomcp_types::Problem {
            source,
            record,
            field: "record".into(),
            code: biomcp_types::ProblemCode::ValueUnknown,
        });
    }
}

fn recovery(out: &mut Outcome, request: Request) {
    if out.sources.iter().any(|s| {
        matches!(
            s.state,
            biomcp_types::SourceState::Failed | biomcp_types::SourceState::Stale
        )
    }) {
        out.next.push(crate::next::Step::run(
            StepKind::Fresh,
            &out.sources
                .iter()
                .map(|source| source.source)
                .collect::<Vec<_>>(),
            request,
            true,
        ));
    }
}
