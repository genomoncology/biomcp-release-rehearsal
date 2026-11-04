//! Conservation measurements retain source values without clinical inference.
use crate::Ctx;

use biomcp_fetch::Deadline;
use biomcp_types::{Section, VariantConservation, VariantGet, VariantHeader};
pub(super) fn section(
    cx: &Ctx<'_>,
    query: &VariantGet,
    candidates: &[VariantHeader],
    deadline: &mut Deadline<'_>,
) -> super::sections::Result<VariantConservation> {
    let (section, sources, problems, local, dropped) =
        super::components::measurements(cx, query, candidates, deadline, true);
    let section = match section {
        Section::Data { source, data } => Section::Data {
            source,
            data: VariantConservation {
                subject_qualification: data.subject_qualification,
                scores: data.scores,
                components: data.components,
                context: data.context,
            },
        },
        Section::Empty { source, record } => Section::Empty { source, record },
        Section::Unavailable { source, reason } => Section::Unavailable { source, reason },
        Section::Skipped { source, reason } => Section::Skipped { source, reason },
        Section::NotApplicable => Section::NotApplicable,
    };
    (section, sources, problems, local, dropped)
}
