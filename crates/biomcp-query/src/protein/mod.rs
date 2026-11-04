//! Exact reviewed-human protein discovery and cards; no clinical interpretation.
mod get;
mod next;
mod relations;
mod search;
mod sections;
mod sequence;
use crate::Outcome;
pub(super) use crate::status::empty;
pub(crate) use get::get;
pub(crate) use search::search;

/// Page checked native feature rows on their own sequence identity.
pub(crate) fn features(cx: &crate::Ctx<'_>, q: &biomcp_types::ProteinFeaturesQuery) -> Outcome {
    relations::run(cx, relations::Selection::Features(q.clone()))
}
/// Page native alternative products without constructing their sequences.
pub(crate) fn isoforms(cx: &crate::Ctx<'_>, q: &biomcp_types::ProteinRelationQuery) -> Outcome {
    relations::run(cx, relations::Selection::Isoforms(q.clone()))
}
/// Page native structure references and source properties.
pub(crate) fn structures(cx: &crate::Ctx<'_>, q: &biomcp_types::ProteinRelationQuery) -> Outcome {
    relations::run(cx, relations::Selection::Structures(q.clone()))
}
/// Read a whole direct sequence under its exact source identity.
pub(crate) fn sequence(cx: &crate::Ctx<'_>, q: &biomcp_types::ProteinSequenceQuery) -> Outcome {
    sequence::run(cx, q)
}
mod source_relations;
mod source_sections;
/// Read complete native InterPro match rows without changing their entry types.
pub(crate) fn domains(cx: &crate::Ctx<'_>, q: &biomcp_types::ProteinDomainsQuery) -> Outcome {
    source_relations::run(cx, source_relations::Selection::Domains(q.clone()))
}
/// Read exact participant-admitted Complex Portal headers.
pub(crate) fn complexes(cx: &crate::Ctx<'_>, q: &biomcp_types::ProteinRelationQuery) -> Outcome {
    source_relations::run(cx, source_relations::Selection::Complexes(q.clone()))
}
/// Page original participants of one exact containing complex.
pub(crate) fn complex_members(
    cx: &crate::Ctx<'_>,
    q: &biomcp_types::ProteinComplexMembersQuery,
) -> Outcome {
    source_relations::run(cx, source_relations::Selection::Members(q.clone()))
}

fn reply_problem(
    out: &mut Outcome,
    source: biomcp_types::Source,
    id: &biodata::UniProtIsoform,
    error: biomcp_fetch::RequestError,
) {
    if matches!(error, biomcp_fetch::RequestError::Source(e) if matches!(e.reason, biomcp_types::Reason::Disabled | biomcp_types::Reason::NoKey))
    {
        return;
    }
    crate::status::append(
        &mut out.problems,
        [biomcp_types::Problem {
            source,
            record: biodata::Text::new(id.as_str()).ok(),
            field: "reply".into(),
            code: biomcp_types::ProblemCode::ValueUnknown,
        }],
    );
}
