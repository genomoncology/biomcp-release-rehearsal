//! Exact NCIt lookups keep multiple native rows as candidates.
use crate::{Action, Ctx, Outcome, status};
use biomcp_adapters::nci;
use biomcp_types::ReplyBody;
use biomcp_types::{BiomarkerGet, BiomarkerResult, Reason, ReplyData, Source, StepKind};
/// Read every native row attached to one exact code.
pub(crate) fn get(cx: &Ctx<'_>, q: &BiomarkerGet) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&nci::SPEC, deadline).fresh(cx.fresh);
    let reply = nci::biomarker::get(&leg, &q.id);
    let error = reply.as_ref().err().copied();
    let source = status::source(Source::Nci, leg.finish(), error);
    let mut out = crate::status::finish(
        [crate::status::item_status(false, true, false)],
        crate::status::Parts {
            dropped: false,
            body: (BiomarkerResult {
                card: None,
                candidates: Box::default(),
            })
            .with_query(q.clone()),
            page: None,
            sources: vec![source],
            problems: Vec::new(),
            next: Vec::new(),
            local: error.and_then(status::local),
        },
    );
    if let Some(error) = error {
        if status::reason(error) == Some(Reason::TooLarge) {
            out.body = ReplyData::Usage(biomcp_types::UsageError::BiomarkerTooBroad);
        }
        if status::reason(error) == Some(Reason::NoKey) {
            out.next.push(crate::next::Step::new(
                StepKind::Key,
                &[biomcp_types::Source::Nci],
                Action::KeyHelp(biomcp_fetch::SecretName::NciApiKey),
            ));
        }
    }
    if let Ok(reply) = reply {
        out.include_refusals(reply.refusals);
        let mut rows = reply.value;
        let total = rows.len();
        if let Some(source) = out.sources.first_mut() {
            source.count = Some(total as u64);
            source.retrieved_count = Some(total as u32);
        }
        status::item(&mut out, total != 0, false, false);
        for row in &rows {
            for id in &row.header.codes {
                let step = crate::next::Step::run(
                    StepKind::Pivot,
                    &[biomcp_types::Source::Nci],
                    biomcp_types::Request::SearchTrial(
                        biomcp_types::TrialSearch::for_nci_biomarker(id.clone()),
                    ),
                    false,
                );
                out.next.push(step);
            }
        }
        out.problems = reply.problems;
        out.body = (BiomarkerResult {
            card: if total == 1 { rows.pop() } else { None },
            candidates: rows.into(),
        })
        .with_query(q.clone());
    }
    command.finish(deadline.remaining());
    out
}
