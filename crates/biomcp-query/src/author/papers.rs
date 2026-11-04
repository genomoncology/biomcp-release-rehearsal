use crate::{Ctx, Outcome};
use biomcp_types::AuthorPapersQuery;
/// Dispatch exact provider records without an identity join.
pub(crate) fn papers(cx: &Ctx<'_>, q: &AuthorPapersQuery) -> Outcome {
    match q.id() {
        biodata::AuthorId::Orcid(id) => super::orcid::papers(cx, q, id),
        biodata::AuthorId::SemanticScholar(id) => super::semantic_scholar::papers(cx, q, id),
    }
}
