use crate::Outcome;
use biomcp_types::{DiagnosticCandidate, DiagnosticGet, DiagnosticSearch, Request, StepKind};
pub(super) fn push(
    out: &mut Outcome,
    kind: StepKind,
    sources: &[biomcp_types::Source],
    request: Request,
    fresh: bool,
) {
    out.next
        .push(crate::next::Step::run(kind, sources, request, fresh));
}
pub(super) fn candidates(
    out: &mut Outcome,
    query: &DiagnosticSearch,
    rows: &[DiagnosticCandidate],
    fresh: bool,
) {
    for row in rows {
        let (id, source) = match row {
            DiagnosticCandidate::PmaSubmission { header } => (
                header.pma_number.to_string(),
                biomcp_types::DiagnosticGetSource::FdaPma,
            ),
            DiagnosticCandidate::Clearance { header } => (
                header.k_number.to_string(),
                biomcp_types::DiagnosticGetSource::Fda510k,
            ),
            DiagnosticCandidate::DeviceRecall { header } => (
                header.recall_number.to_string(),
                biomcp_types::DiagnosticGetSource::FdaDeviceRecall,
            ),
            DiagnosticCandidate::Resolved { header, .. } => (
                header.id.to_string(),
                biomcp_types::DiagnosticGetSource::Gtr,
            ),
            DiagnosticCandidate::WhoIvd { id, .. } => {
                (id.to_string(), biomcp_types::DiagnosticGetSource::WhoIvd)
            }
            _ => continue,
        };
        let get = id
            .parse()
            .map_err(|_| biomcp_types::QueryInvalid::new(&["id"]))
            .and_then(|id| {
                let supplement = match row {
                    DiagnosticCandidate::PmaSubmission { header } => header
                        .supplement_number
                        .as_ref()
                        .filter(|s| !s.as_str().is_empty())
                        .map(|s| s.as_str().parse())
                        .transpose()
                        .map_err(|_| biomcp_types::QueryInvalid::new(&["supplement"]))?,
                    _ => None,
                };
                DiagnosticGet::new(id, source, supplement)
            });
        if let Some(mut get) = crate::next::checked(get, source.source(), &mut out.problems) {
            if let Some(revision) = query.bundle_revision() {
                get = get.with_bundle_revision(revision.clone());
            }
            push(
                out,
                StepKind::Disambiguate,
                &[get.source().source()],
                Request::GetDiagnostic(get),
                fresh,
            );
        }
    }
}
