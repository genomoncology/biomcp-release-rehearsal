use crate::{Ctx, Outcome, status};
use biomcp_adapters::fda_510k;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DiagnosticCard, DiagnosticGet, DiagnosticSearch, Problem, ProblemCode, Request, Source,
    SourceState, Status, StepKind,
};
pub(super) fn get(cx: &Ctx<'_>, q: &DiagnosticGet) -> Outcome {
    let mut out = super::empty(biomcp_types::DiagnosticCard::into_reply(q.clone(), None));
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&fda_510k::SPEC, deadline).fresh(cx.fresh);
    let error = match fda_510k::get(&leg, q) {
        Ok(card) => {
            let card = card.map(|mapped| {
                out.include_refusals(mapped.refusals);
                out.problems.extend(mapped.problems);
                mapped.value
            });
            crate::status::item(&mut out, card.is_some(), false, false);
            if card.is_some() {
                if let biomcp_types::DiagnosticId::Clearance(id) = q.id()
                    && let Some(search) = crate::next::checked(
                        DiagnosticSearch::companion(id.clone()),
                        biomcp_types::Source::FdaCdx,
                        &mut out.problems,
                    )
                {
                    super::next::push(
                        &mut out,
                        StepKind::Pivot,
                        &[search.source().source()],
                        Request::SearchDiagnostic(search),
                        cx.fresh,
                    );
                }
                if !cx.fresh {
                    super::next::push(
                        &mut out,
                        StepKind::Fresh,
                        &[q.source().source()],
                        Request::GetDiagnostic(q.clone()),
                        true,
                    );
                }
            }
            out.body = biomcp_types::DiagnosticCard::into_reply(
                q.clone(),
                card.map(DiagnosticCard::Clearance),
            );
            None
        }
        Err(failure) => {
            if failure.identity_conflict {
                out.problems.push(Problem {
                    source: Source::Fda510k,
                    record: Some(q.record().clone()),
                    field: "k_number".into(),
                    code: ProblemCode::IdentityConflict,
                });
            }
            Some(failure.error)
        }
    };
    out.local = error.and_then(status::local);
    let source = status::source(Source::Fda510k, leg.finish(), error);
    if source.state == SourceState::Stale && out.status() == Status::Complete {
        crate::status::item(&mut out, true, false, true);
    }
    out.sources.push(source);
    command.finish(deadline.remaining());
    out
}
