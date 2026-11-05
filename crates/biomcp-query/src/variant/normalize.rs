//! Explicit provider normalization with independent source outcomes.
use crate::{Ctx, Outcome, status};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;

use biomcp_adapters::{allele_registry, mutalyzer, variantvalidator};
use biomcp_fetch::{Deadline, RequestError, SourceSpec};
use biomcp_types::{
    NormalizationResult, NormalizationSource, Request, SourceOutcome, StepKind,
    VariantNormalization, VariantNormalize, VariantResolution,
};
type Row = (
    biomcp_adapters::Mapped<NormalizationResult>,
    SourceOutcome,
    Option<RequestError>,
);
/// Call each explicitly selected service independently under the shared deadline.
pub(crate) fn normalize(cx: &Ctx<'_>, query: &VariantNormalize) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let jobs = selected(query.source())
        .into_iter()
        .map(|spec| {
            Box::new(move || service(cx, query, spec, deadline)) as Box<dyn FnOnce() -> Row + Send>
        })
        .collect();
    let mut results = Vec::new();
    let mut sources = Vec::new();
    let mut local = None;
    let mut more = false;
    let mut states = Vec::new();
    let mut problems = Vec::new();
    for (mapped, source, error) in crate::fan_out::run(jobs) {
        problems.extend(mapped.problems);
        let mut result = mapped.value;
        states.push(
            if matches!(
                result.outcome,
                VariantResolution::Rejected | VariantResolution::Unavailable
            ) {
                crate::status::item_status(false, true, false)
            } else {
                crate::status::item_status(true, false, false)
            },
        );
        result.total_messages = result.messages.len() as u32;
        more |= result.total_messages.max(result.total_mappings) as u64
            > query.offset() as u64 + query.limit() as u64;
        result.messages = result
            .messages
            .into_vec()
            .into_iter()
            .skip(query.offset() as usize)
            .take(query.limit() as usize)
            .collect();
        result.mappings = result
            .mappings
            .into_vec()
            .into_iter()
            .skip(query.offset() as usize)
            .take(query.limit() as usize)
            .collect();
        results.push(result);
        sources.push(source);
        local = local.or(error);
    }
    let next = if more {
        vec![crate::next::Step::run(
            StepKind::Page,
            &selected(query.source())
                .iter()
                .map(|spec| spec.source)
                .collect::<Vec<_>>(),
            Request::VariantNormalize(
                query.at_offset(query.offset().saturating_add(query.limit())),
            ),
            false,
        )]
    } else {
        Vec::new()
    };
    let outcome = crate::status::finish(
        [status::status(states.into_iter())],
        crate::status::Parts {
            dropped: false,
            body: (VariantNormalization {
                results: results.into(),
            })
            .with_query(query.clone()),
            sources,
            problems,
            next,
            page: None,
            local,
        },
    );
    command.finish(deadline.remaining());
    outcome
}
fn selected(source: NormalizationSource) -> Vec<&'static SourceSpec> {
    match source {
        NormalizationSource::Car => vec![&allele_registry::SPEC],
        NormalizationSource::Mutalyzer => vec![&mutalyzer::SPEC],
        NormalizationSource::Variantvalidator => vec![&variantvalidator::SPEC],
        NormalizationSource::All => vec![
            &allele_registry::SPEC,
            &mutalyzer::SPEC,
            &variantvalidator::SPEC,
        ],
    }
}
fn service(
    cx: &Ctx<'_>,
    query: &VariantNormalize,
    spec: &'static SourceSpec,
    deadline: Deadline<'_>,
) -> Row {
    let leg = cx.fetch.leg(spec, deadline).fresh(cx.fresh);
    let result = match spec.source {
        biomcp_types::Source::AlleleRegistry => {
            allele_registry::normalize(&leg, query.hgvs(), query.build())
        }
        biomcp_types::Source::Mutalyzer => mutalyzer::normalize(&leg, query.hgvs()),
        _ => query
            .build()
            .ok_or(
                biomcp_fetch::SourceError {
                    source: spec.source,
                    reason: biomcp_types::Reason::Rejected,
                    status: None,
                    tls_cause: None,
                    retry_after: None,
                }
                .into(),
            )
            .and_then(|build| variantvalidator::normalize(&leg, query.hgvs(), build)),
    };
    let error = result.as_ref().err().copied();
    let mut source = status::source(spec.source, leg.finish(), error);
    let result = result.unwrap_or_else(|_| biomcp_adapters::Mapped {
        refusals: biomcp_adapters::Refusal::None,
        value: NormalizationResult::unavailable(spec.source),
        problems: Vec::new(),
    });
    if result.value.outcome == VariantResolution::Rejected {
        source.set_status(biomcp_types::SourceStatus::Failed(Some(
            biomcp_types::Reason::Rejected,
        )));
    }
    (result, source, error.and_then(status::local))
}
