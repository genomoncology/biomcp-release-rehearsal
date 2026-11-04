use super::{empty, finish_bundle};
use crate::{Action, Bundle, Ctx, Outcome};
use biomcp_adapters::who_ivd;
use biomcp_types::ReplyBody;
use biomcp_types::{DiagnosticCard, DiagnosticGet, Request, StepKind};

pub(super) fn get(cx: &Ctx<'_>, query: &DiagnosticGet) -> Outcome {
    let mut out = empty(biomcp_types::DiagnosticCard::into_reply(
        query.clone(),
        None,
    ));
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&who_ivd::SPEC, deadline);
    let result = who_ivd::get(&leg, query);
    let error = result.as_ref().err().copied();
    if let Ok((product, _info)) = result {
        {
            let found = product.is_some();
            crate::status::item(&mut out, found, false, false);
        };
        out.body = biomcp_types::DiagnosticCard::into_reply(
            query.clone(),
            product.map(DiagnosticCard::WhoIvdProduct),
        );
        out.next.push(crate::next::Step::new(
            StepKind::Sync,
            &[biomcp_types::Source::WhoIvd],
            Action::Sync {
                bundle: Bundle::WHO_IVD,
            },
        ));
        if !cx.fresh {
            out.next.push(crate::next::Step::run(
                StepKind::Fresh,
                &[biomcp_types::Source::WhoIvd],
                Request::GetDiagnostic(query.clone()),
                true,
            ));
        }
    }
    finish_bundle(
        &mut out,
        leg,
        error,
        biomcp_types::Source::WhoIvd,
        Bundle::WHO_IVD,
    );
    command.finish(deadline.remaining());
    out
}
