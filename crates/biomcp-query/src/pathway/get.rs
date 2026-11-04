use super::{empty, run};
use crate::{Ctx, Outcome, status};
use biodata::PathwayId;
use biomcp_adapters::reactome;
use biomcp_types::ReplyBody;
use biomcp_types::{PathwayGet, PathwaySection, ReplyData, Request, Source, StepKind};
/// Read one exact native human pathway and only its requested bounded disclosures.
pub(crate) fn get(cx: &Ctx<'_>, q: &PathwayGet) -> Outcome {
    if matches!(q.id(), PathwayId::Wikipathways(_)) {
        return super::wikipathways::get(cx, q);
    }
    let mut out = empty(biomcp_types::PathwayCard::into_reply(q.clone(), None));
    let PathwayId::Reactome(id) = q.id() else {
        return out;
    };
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx
        .fetch
        .leg(&reactome::SPEC, deadline)
        .fresh(cx.fresh || q.fresh());
    let reply = reactome::card(
        &leg,
        id,
        matches!(
            q.section(),
            Some(PathwaySection::Summary | PathwaySection::All)
        ),
    );
    let mut error = reply.as_ref().err().copied();
    let mut card = match reply {
        Ok(mapped) => {
            out.include_refusals(mapped.refusals);
            out.problems = mapped.problems;
            mapped.value
        }
        Err(_) => None,
    };
    if let Some(value) = &mut card {
        match reactome::release(&leg) {
            Ok(release) => value.release = Some(release),
            Err(e) => error = Some(e),
        }
    }
    out.sources
        .push(status::source(Source::Reactome, leg.finish(), error));
    if let Some(value) = &mut card {
        super::get_sections::attach(cx, q, value, deadline, &mut out);
        if !matches!(out.body, ReplyData::PoolError { .. }) {
            for section in &value.sections {
                if q.section() != Some(PathwaySection::All)
                    && Some(*section) != q.section()
                    && let Some(query) = crate::next::checked(
                        q.with_section(*section),
                        biomcp_types::Source::Reactome,
                        &mut out.problems,
                    )
                {
                    out.next.push(crate::next::Step::new(
                        StepKind::Section,
                        &[biomcp_types::Source::Reactome],
                        run(Request::GetPathway(query)),
                    ));
                }
            }
        }
    }
    if !matches!(out.body, ReplyData::PoolError { .. }) {
        if card.is_some() || error.is_none() {
            {
                let absent = card.is_none();
                crate::status::item(&mut out, !absent, false, false);
            }
        }
        out.body = biomcp_types::PathwayCard::into_reply(q.clone(), card);
    }
    out.local = out.local.or_else(|| error.and_then(status::local));
    command.finish(deadline.remaining());
    out
}
