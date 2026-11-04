use biomcp_types::Source;
pub(crate) fn author(id: &biodata::AuthorId) -> Source {
    match id {
        biodata::AuthorId::SemanticScholar(_) => Source::SemanticScholar,
        biodata::AuthorId::Orcid(_) => Source::Orcid,
    }
}
