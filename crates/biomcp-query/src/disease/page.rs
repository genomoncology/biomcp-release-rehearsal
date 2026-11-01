use crate::Outcome;
pub(super) use crate::next::run;
pub(super) use crate::status::empty;
pub(super) use crate::status::page_info;
use biomcp_types::DiseaseGet;
pub(super) fn slice<T>(rows: Vec<T>, query: &DiseaseGet) -> (Vec<T>, u64) {
    let total = rows.len() as u64;
    (
        rows.into_iter()
            .skip(query.section_offset() as usize)
            .take(query.section_limit() as usize)
            .collect(),
        total,
    )
}
/// Every rejected disease get keeps its source reason and records the rejected identity.
pub(super) fn record_problem(out: &mut Outcome, query: &DiseaseGet, source: biomcp_types::Source) {
    if out.sources.iter().any(|outcome| {
        outcome.source == source && outcome.reason == Some(biomcp_types::Reason::BadReply)
    }) {
        out.problems.push(biomcp_types::Problem {
            source,
            record: Some(query.record().clone()),
            field: "record".into(),
            code: biomcp_types::ProblemCode::ValueUnknown,
        });
    }
}
