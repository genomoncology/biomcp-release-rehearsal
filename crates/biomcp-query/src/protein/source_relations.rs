//! Complete checked source collections page without admitting inferred memberships.
use crate::{Ctx, Outcome, source_snapshot, status};
use biodata::{Text, UniProtIsoform};
use biomcp_adapters::{complex_portal, interpro};
use biomcp_fetch::{Deadline, RequestError, SourceError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    PageInfo, ProteinComplexMembersQuery, ProteinDomainsQuery, ProteinRelationQuery, ReplyData,
    Request, Source, StepKind,
};
mod saved;
use saved::Saved;
#[derive(Clone)]
pub(super) enum Selection {
    Domains(ProteinDomainsQuery),
    Complexes(ProteinRelationQuery),
    Members(ProteinComplexMembersQuery),
}
impl Selection {
    fn accession(&self) -> &UniProtIsoform {
        match self {
            Self::Domains(q) => q.accession(),
            Self::Complexes(q) => q.accession(),
            Self::Members(q) => q.accession(),
        }
    }
    fn window(&self) -> (u32, u32, Option<&Text>) {
        match self {
            Self::Domains(q) => (q.limit(), q.offset(), q.pool()),
            Self::Complexes(q) => (q.limit(), q.offset(), q.pool()),
            Self::Members(q) => (q.limit(), q.offset(), q.pool()),
        }
    }
    fn source(&self) -> Source {
        match self {
            Self::Domains(_) => Source::InterPro,
            _ => Source::ComplexPortal,
        }
    }
    fn spec(&self) -> &'static biomcp_fetch::SourceSpec {
        match self {
            Self::Domains(_) => &interpro::SPEC,
            _ => &complex_portal::SPEC,
        }
    }
    fn request(&self, page: Option<(Text, u32)>) -> Request {
        match self {
            Self::Domains(q) => {
                Request::ProteinDomains(page.map_or_else(|| q.restart(), |(p, o)| q.at_page(p, o)))
            }
            Self::Complexes(q) => Request::ProteinComplexes(
                page.map_or_else(|| q.restart(), |(p, o)| q.at_page(p, o)),
            ),
            Self::Members(q) => Request::ProteinComplexMembers(
                page.map_or_else(|| q.restart(), |(p, o)| q.at_page(p, o)),
            ),
        }
    }
    fn key(&self) -> Result<Vec<u8>, RequestError> {
        source_snapshot::key("protein-source-relations", &self.request(None))
    }
    pub(super) fn empty(&self) -> ReplyData {
        match self {
            Self::Domains(q) => biomcp_types::ProteinDomains::into_reply(q.clone(), None),
            Self::Complexes(q) => biomcp_types::ProteinComplexes::into_reply(q.clone(), None),
            Self::Members(q) => biomcp_types::ProteinComplexMembers::into_reply(q.clone(), None),
        }
    }
}
pub(super) fn run(cx: &Ctx<'_>, q: Selection) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let mut out = super::empty(q.empty());
    if let Err(e) = read(cx, &q, deadline, &mut out) {
        out.local = status::local(e);
        out.next.push(crate::next::Step::run(
            StepKind::Fresh,
            &[q.source()],
            q.request(None),
            true,
        ));
    }
    command.finish(deadline.remaining());
    out
}
fn restart(q: &Selection, out: &mut Outcome, mismatch: bool) {
    out.body = ReplyData::PoolError { mismatch };
    out.next.push(crate::next::Step::run(
        StepKind::Restart,
        &[q.source()],
        q.request(None),
        false,
    ));
}
pub(super) fn read(
    cx: &Ctx<'_>,
    q: &Selection,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let leg = cx.fetch.leg(q.spec(), deadline).fresh(cx.fresh);
    if cx.fetch.disabled(q.source()) {
        let e = SourceError::disabled(q.source()).into();
        out.sources
            .push(status::source(q.source(), leg.finish(), Some(e)));
        return Err(e);
    }
    let (limit, offset, selected) = q.window();
    if selected.is_none() && offset != 0 || selected.is_some() && cx.fresh {
        restart(q, out, false);
        return Ok(());
    }
    let Some((saved, pool)) = load(cx, q, leg, deadline, out)? else {
        return Ok(());
    };
    if let (Selection::Members(query), Saved::Members(None)) = (q, &saved) {
        out.problems.push(biomcp_types::Problem {
            source: q.source(),
            record: Text::new(query.complex().as_str()).ok(),
            field: format!("participants.anchor.{}", query.accession()).into(),
            code: biomcp_types::ProblemCode::ValueUnknown,
        });
        return Ok(());
    }
    let total = saved.total();
    if offset as u64 > total {
        restart(q, out, false);
        return Ok(());
    }
    let returned = limit.min(total.saturating_sub(offset.into()) as u32);
    saved.member_steps(q, offset, limit, cx, deadline, out)?;
    saved.render(q, offset, limit, out)?;
    {
        let unavailable = out
            .sources
            .iter()
            .any(|s| s.state != biomcp_types::SourceState::Ok);
        crate::status::item(out, true, false, unavailable);
    };
    out.page = Some(PageInfo {
        offset,
        returned,
        total,
        pool: Some(pool.clone()),
        truncated: None,
    });
    if u64::from(offset + returned) < total {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &[q.source()],
            q.request(Some((pool.id, offset + returned))),
            false,
        ));
    }
    Ok(())
}

fn load(
    cx: &Ctx<'_>,
    q: &Selection,
    leg: biomcp_fetch::Leg<'_>,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<Option<(Saved, biomcp_types::PoolInfo)>, RequestError> {
    let (_, _, selected) = q.window();
    let key = q.key()?;
    let (saved, mismatch) = if cx.fresh {
        (None, false)
    } else {
        source_snapshot::find::<Saved>(cx, &key, selected, deadline)?
    };
    let loaded = if let Some((saved, pool)) = saved {
        out.sources = pool.sources;
        (saved, pool.info)
    } else {
        if selected.is_some() {
            restart(q, out, mismatch);
            return Ok(None);
        }
        let result = Saved::fetch(q, &leg);
        let error = result.as_ref().err().copied();
        out.sources
            .push(status::source(q.source(), leg.finish(), error));
        let saved = match result {
            Ok(saved) => saved,
            Err(error) => {
                super::reply_problem(out, q.source(), q.accession(), error);
                return Err(error);
            }
        };
        let size = saved.total() as u32;
        let pool = source_snapshot::publish(cx, &key, &saved, size, &out.sources, None, deadline)?;
        (saved, pool.info)
    };
    Ok(Some(loaded))
}
