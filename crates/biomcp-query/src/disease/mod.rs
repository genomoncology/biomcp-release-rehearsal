//! Source-owned disease facts without identity choice or cross-provider fallback.
/// Source-native Monarch association pages.
pub(crate) mod monarch;
/// MyDisease exact annotation outcomes.
pub(crate) mod mydisease;
/// OLS source-owned identity and discovery outcomes.
pub(crate) mod ols;
mod page;
/// Native SEER site and statistics pages.
pub(crate) mod seer;
use crate::{Ctx, Outcome};
use biomcp_types::{DiseaseGet, DiseaseSearch, DiseaseSource};
/// Discover one page from the explicitly selected source.
pub(crate) fn search(cx: &Ctx<'_>, query: &DiseaseSearch) -> Outcome {
    match query.source() {
        biomcp_types::DiseaseSource::Seer => seer::search(cx, query),
        _ => ols::search(cx, query),
    }
}
/// Read one exact concept or selected source page.
pub(crate) fn get(cx: &Ctx<'_>, query: &DiseaseGet) -> Outcome {
    match (query.id(), query.source()) {
        (biomcp_types::DiseaseId::Concept(id), DiseaseSource::MyDisease) => {
            mydisease::get(cx, query, id)
        }
        (biomcp_types::DiseaseId::Concept(id), DiseaseSource::Monarch) => {
            monarch::get(cx, query, id)
        }
        (biomcp_types::DiseaseId::SeerSite(id), biomcp_types::DiseaseSource::Seer) => {
            seer::get(cx, query, id)
        }
        _ => ols::get(cx, query),
    }
}
