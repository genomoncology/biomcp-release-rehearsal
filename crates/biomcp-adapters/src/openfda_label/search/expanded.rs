//! Exact-name union first; an explicitly selected word tier stays word-based on every page.
use super::{Search, candidates};
use biodata::Text;
use biomcp_fetch::{Leg, RequestError};
use biomcp_types::DrugSearch;
use serde::{Deserialize, Serialize};

/// Native label-name matching, distinct from unfielded label-text discovery.
/// Native matching tier retained with an expanded name union.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum NameTier {
    /// Harmonized exact brand and generic names.
    Exact,
    /// Harmonized word-matching brand and generic names.
    Words,
}
impl NameTier {
    /// Native exact-value versus analyzed-word name fields.
    /// Whether this retained tier uses exact fields.
    pub fn exact(self) -> bool {
        matches!(self, Self::Exact)
    }
}

/// Search all accepted spellings in one native expression; never combine per-name totals.
/// Search one bounded name union, selecting a tier only on its initial page.
pub fn search_expanded(
    leg: &Leg<'_>,
    query: &DrugSearch,
    names: &[Text],
    selected: Option<NameTier>,
) -> (Search, NameTier) {
    let mut tier = selected.unwrap_or(NameTier::Exact);
    let request = |tier: NameTier| -> Result<super::Page, RequestError> {
        let selector = query
            .label_name_selector(names, tier.exact())
            .map_err(|_| biomcp_fetch::InputError::Serialization)?;
        candidates(leg, selector.as_str(), query.limit(), query.offset())
    };
    let mut page = request(tier);
    if selected.is_none()
        && page
            .as_ref()
            .is_ok_and(|page| page.window.total() == Some(0))
    {
        tier = NameTier::Words;
        page = request(tier);
    }
    (
        Search {
            page,
            free_text: None,
        },
        tier,
    )
}
