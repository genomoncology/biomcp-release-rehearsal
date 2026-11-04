//! One exact term reply supplies every local section page.
use super::{empty, problem, recovery, run};
use crate::{Ctx, Outcome, source_snapshot, status};
use biodata::{PhenotypeHierarchyLink, PhenotypeSynonym, PhenotypeXref, Text};
use biomcp_adapters::{hpo, monarch};
use biomcp_fetch::{Deadline, RequestError, SourceError};
use biomcp_types::Paged;
use biomcp_types::{
    PhenotypeGet, PhenotypeSource, PhenotypeTermCard, Reason, ReplyData, Request, Source,
    SourceOutcome, StepKind,
};
pub(super) type Rows = (
    PhenotypeTermCard,
    Option<Box<[PhenotypeSynonym]>>,
    Option<Box<[PhenotypeXref]>>,
    Option<Box<[PhenotypeHierarchyLink]>>,
    Option<u64>,
);
type Saved = (Rows, Vec<SourceOutcome>, Vec<biomcp_types::Problem>, bool);
fn key(q: &PhenotypeGet) -> Result<Vec<u8>, RequestError> {
    source_snapshot::key("phenotype", &(q.source(), q.id()))
}
/// Read one independent source card or a local section from its retained reply.
pub(super) fn get(cx: &Ctx<'_>, q: &PhenotypeGet) -> Outcome {
    let mut out = empty(super::sections::empty(q));
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    match load(cx, q, deadline, &mut out) {
        Ok(Some((rows, token))) => super::sections::apply(
            &mut out,
            q,
            &token.map_or_else(|| q.clone(), |t| q.with_snapshot(t)),
            rows,
        ),
        Ok(None) => (),
        Err(error) => {
            out.local = status::local(error);
            if matches!(error,RequestError::Source(e) if e.reason==Reason::NotFound) {
                crate::status::item(&mut out, false, false, false);
            }
        }
    }
    problem(&mut out, source(q), q.id().as_str().parse().ok());
    recovery(&mut out, Request::GetPhenotype(q.restart()));
    command.finish(deadline.remaining());
    out
}
pub(super) fn source(q: &PhenotypeGet) -> Source {
    match q.source() {
        PhenotypeSource::Hpo => Source::Hpo,
        PhenotypeSource::Monarch => Source::Monarch,
    }
}
fn load(
    cx: &Ctx<'_>,
    q: &PhenotypeGet,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<Option<(Rows, Option<Text>)>, RequestError> {
    let spec = match q.source() {
        PhenotypeSource::Hpo => &hpo::SPEC,
        PhenotypeSource::Monarch => &monarch::SPEC,
    };
    let leg = cx.fetch.leg(spec, deadline).fresh(cx.fresh);
    if cx.fetch.disabled(source(q))
        || (q.source() == PhenotypeSource::Monarch
            && biomcp_adapters::disease_sources::components::monarch(&leg).is_err())
    {
        let error = RequestError::Source(SourceError::disabled(source(q)));
        out.sources
            .push(status::source(source(q), leg.finish(), Some(error)));
        return Err(error);
    }
    let (saved, token) = if let Some(token) = q.snapshot() {
        let saved: Option<Saved> = source_snapshot::read(cx, &key(q)?, token.as_str(), deadline)?;
        let Some(saved) = saved else {
            out.body = ReplyData::PoolError { mismatch: false };
            out.next.push(crate::next::Step::new(
                StepKind::Restart,
                &[source(q)],
                run(Request::GetPhenotype(q.restart())),
            ));
            return Ok(None);
        };
        (saved, Some(token.clone()))
    } else {
        let saved = acquire(q, leg, out)?;
        let token = crate::publish::keep(
            source_snapshot::write(cx, &key(q)?, &saved, &out.sources, deadline),
            out,
        )?;
        (saved, token)
    };
    out.sources = saved.1;
    out.problems = saved.2;
    out.dropped = saved.3;
    {
        let absent = false;
        crate::status::item(out, !absent, false, false);
    }
    Ok(Some((saved.0, token)))
}

fn acquire(
    q: &PhenotypeGet,
    leg: biomcp_fetch::Leg<'_>,
    out: &mut Outcome,
) -> Result<Saved, RequestError> {
    let fetched = |metadata: biomcp_fetch::ReplyMetadata| {
        metadata.fetched.and_then(|value| u64::try_from(value).ok())
    };
    let reply = match q.source() {
        PhenotypeSource::Hpo => hpo::get(&leg, q).map(|r| {
            out.problems.extend(r.problems);
            (r.card, r.synonyms, r.xrefs, None, fetched(r.metadata))
        }),
        PhenotypeSource::Monarch => monarch::phenotype::get(&leg, q).map(|r| {
            out.include_refusals(r.refusals);
            out.problems.extend(r.problems);
            (
                r.card,
                r.synonyms,
                r.xrefs,
                r.hierarchy,
                fetched(r.metadata),
            )
        }),
    };
    let error = reply.as_ref().err().copied();
    out.sources.push(status::source(
        source(q),
        leg.finish(),
        error.filter(
            |e| !matches!(e, RequestError::Source(source) if source.reason == Reason::NotFound),
        ),
    ));
    Ok((
        reply?,
        out.sources.clone(),
        out.problems.clone(),
        out.dropped,
    ))
}
