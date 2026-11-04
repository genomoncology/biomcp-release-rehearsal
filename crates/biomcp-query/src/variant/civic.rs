use crate::{Ctx, status};

use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{
    Problem, Reason, Section, Source, SourceOutcome, VariantCivic, VariantGet, VariantHeader,
};

pub(super) fn section(
    cx: &Ctx<'_>,
    query: &VariantGet,
    candidates: &[VariantHeader],
    deadline: Deadline<'_>,
) -> (
    Section<VariantCivic>,
    Vec<SourceOutcome>,
    Vec<Problem>,
    Option<RequestError>,
    bool,
) {
    let leg = cx
        .fetch
        .leg(&biomcp_adapters::civic::SPEC, deadline)
        .fresh(cx.fresh);
    let result = candidates.first().map_or_else(
        || {
            Err(biomcp_fetch::SourceError {
                source: Source::Civic,
                reason: Reason::NoJoinId,
                status: None,
                tls_cause: None,
                retry_after: None,
            }
            .into())
        },
        |candidate| biomcp_adapters::civic::variant(&leg, query, candidate),
    );
    let error = result.as_ref().err().copied();
    let dropped = result
        .as_ref()
        .is_ok_and(|mapped| mapped.refusals == biomcp_adapters::Refusal::Refused);
    let source = status::source(Source::Civic, leg.finish(), error);
    let (value, problems) = match result {
        Ok(mapped) => (
            if mapped.value.profiles.is_empty() {
                Section::Empty {
                    source: Source::Civic,
                    record: None,
                }
            } else {
                Section::Data {
                    source: Source::Civic,
                    data: mapped.value,
                }
            },
            mapped.problems,
        ),
        Err(error) => (
            Section::unavailable(
                Source::Civic,
                crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable),
            ),
            Vec::new(),
        ),
    };
    (
        value,
        vec![source],
        problems,
        error.and_then(status::local),
        dropped,
    )
}
