//! Component restrictions are decided before reading replies or caches.
use crate::{Ctx, status};
use biodata::Build;
use biomcp_fetch::Deadline;
use biomcp_types::{
    Reason, Section, Source, SourceState, VariantGet, VariantHeader, VariantPredictors,
};
pub(super) fn skipped<T>() -> super::sections::Result<T> {
    (
        Section::Skipped {
            source: Source::MyVariant,
            reason: Reason::Disabled,
        },
        vec![biomcp_adapters::registry::outcome(
            Source::MyVariant,
            SourceState::Skipped,
            Some(Reason::Disabled),
        )],
        Vec::new(),
        None,
        false,
    )
}
pub(super) fn target(candidates: &[VariantHeader]) -> Option<(String, Build)> {
    let [candidate] = candidates else { return None };
    if candidate.source == Source::MyVariant
        && let Some(build) = super::identity::native_build(candidate)
    {
        return Some((candidate.record.id.to_string(), build));
    }
    [biomcp_types::VariantGet::DEFAULT_BUILD, Build::GRCh37]
        .into_iter()
        .find_map(|build| super::join_for_build(candidate, build).map(|id| (id, build)))
}
pub(super) fn measurements(
    cx: &Ctx<'_>,
    _query: &VariantGet,
    candidates: &[VariantHeader],
    deadline: &mut Deadline<'_>,
    conservation: bool,
) -> super::sections::Result<VariantPredictors> {
    use biomcp_adapters::myvariant::{self, components};
    let cadd = !conservation && !components::disabled(cx.fetch, components::CADD.name);
    let dbnsfp = !components::disabled(cx.fetch, components::DBNSFP.name);
    if cx.fetch.disabled(Source::MyVariant) || (!cadd && !dbnsfp) {
        return skipped();
    }
    let Some((id, build)) = target(candidates) else {
        return (Section::NotApplicable, Vec::new(), Vec::new(), None, false);
    };
    let leg = cx.fetch.leg(&myvariant::SPEC, *deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let result = myvariant::measurements::get(&leg, &id, build, conservation, cadd, dbnsfp);
    let error = result.as_ref().err().copied();
    *deadline = deadline.after_stage(started.saturating_sub(leg.remaining()));
    let source = status::source(Source::MyVariant, leg.finish(), error);
    let mut dropped = false;
    let mut problems = Vec::new();
    let section = match result {
        Ok(mapped) => {
            dropped |= mapped.refusals == biomcp_adapters::Refusal::Refused;
            problems = mapped.problems;
            if problems
                .iter()
                .any(|p| p.field.as_ref() == "subject_qualification")
            {
                Section::Unavailable {
                    source: Source::MyVariant,
                    reason: Reason::NoJoinId,
                }
            } else if mapped.value.scores.is_empty()
                && mapped.value.context.is_empty()
                && (problems.is_empty() || problems.iter().any(|p| p.field.as_ref() == "_id"))
            {
                Section::Empty {
                    source: Source::MyVariant,
                    record: candidates.first().map(|c| c.record.clone()),
                }
            } else {
                Section::Data {
                    source: Source::MyVariant,
                    data: mapped.value,
                }
            }
        }
        Err(error) => Section::unavailable(
            Source::MyVariant,
            crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable),
        ),
    };
    (
        section,
        vec![source],
        problems,
        error.and_then(status::local),
        dropped,
    )
}
