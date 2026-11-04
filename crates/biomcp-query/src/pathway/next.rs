use biomcp_types::{PathwaySearch, Source};
pub(super) fn identity(id: &biodata::PathwayId) -> Source {
    match id {
        biodata::PathwayId::Reactome(_) => Source::Reactome,
        biodata::PathwayId::Wikipathways(_) => Source::WikiPathways,
    }
}
pub(super) fn search(query: &PathwaySearch) -> Vec<Source> {
    query
        .sources()
        .iter()
        .map(|source| source.source())
        .collect()
}
