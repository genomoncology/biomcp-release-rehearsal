//! Direct sequence identity never inherits another isoform's residues or audit.
use crate::{Ctx, Outcome, status};
use biodata::{ProteinSequence, Text};
use biomcp_adapters::{Mapped, uniprot};
use biomcp_fetch::{Leg, RequestError, SourceError};
use biomcp_types::ReplyBody;
use biomcp_types::{
    Problem, ProblemCode, ProteinSequenceQuery, ProteinSequenceResult, Reason, Request, Source,
    StepKind,
};
pub(super) fn read(
    leg: &Leg<'_>,
    q: &ProteinSequenceQuery,
) -> Result<Mapped<Option<ProteinSequence>>, RequestError> {
    if q.isoform.as_str().contains('-') {
        return uniprot::sequence(leg, &q.isoform).map(|sequence| Mapped {
            refusals: biomcp_adapters::Refusal::None,
            value: Some(sequence),
            problems: Vec::new(),
        });
    }
    let mapped = uniprot::entry(leg, &q.isoform)?;
    let value = mapped
        .value
        .map(|entry| {
            entry.sequence.ok_or(RequestError::Source(SourceError {
                source: Source::UniProt,
                reason: Reason::BadReply,
                status: None,
                tls_cause: None,
                retry_after: None,
            }))
        })
        .transpose()?;
    Ok(Mapped {
        refusals: mapped.refusals,
        value,
        problems: mapped.problems,
    })
}
pub(super) fn problem(id: &biodata::UniProtIsoform) -> Problem {
    Problem {
        source: Source::UniProt,
        record: Text::new(id.as_str()).ok(),
        field: "sequence.identity_or_residues".into(),
        code: ProblemCode::ValueUnknown,
    }
}
pub(super) fn run(cx: &Ctx<'_>, q: &ProteinSequenceQuery) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&uniprot::SPEC, deadline).fresh(cx.fresh);
    let mut out = super::empty(biomcp_types::ProteinSequenceResult::into_reply(
        q.clone(),
        None,
    ));
    let result = read(&leg, q);
    let error = result.as_ref().err().copied();
    if let Ok(mapped) = result {
        out.include_refusals(mapped.refusals);
        out.problems = mapped.problems;
        {
            let found = mapped.value.is_some();
            crate::status::item(&mut out, found, false, false);
        };
        out.body = biomcp_types::ProteinSequenceResult::into_reply(
            q.clone(),
            mapped
                .value
                .map(|sequence| ProteinSequenceResult { sequence }),
        );
    }
    if error.is_some_and(|e| matches!(e, RequestError::Source(e) if e.reason == Reason::BadReply)) {
        out.problems.push(problem(&q.isoform));
    }
    out.sources
        .push(status::source(Source::UniProt, leg.finish(), error));
    if error.is_some() {
        out.next.push(crate::next::Step::run(
            StepKind::Fresh,
            &[biomcp_types::Source::UniProt],
            Request::ProteinSequence(q.clone()),
            true,
        ));
    } else if out
        .sources
        .iter()
        .any(|s| s.state == biomcp_types::SourceState::Stale)
    {
        crate::status::item(&mut out, true, false, true);
    }
    out.local = error.and_then(status::local);
    command.finish(deadline.remaining());
    out
}
