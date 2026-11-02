use crate::{Doi, Error, EuropePmcId, Pmcid, Pmid, text_value};
/// A checked article identifier, retaining its namespace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArticleInput {
    /// PubMed identifier.
    Pmid(Pmid),
    /// PMC identifier.
    Pmcid(Pmcid),
    /// Digital object identifier.
    Doi(Doi),
    /// Europe PMC preprint identity.
    EuropePmc(EuropePmcId),
}
impl std::str::FromStr for ArticleInput {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        if s.starts_with("PPR") {
            s.parse().map(Self::EuropePmc)
        } else if s.starts_with("PMC") {
            s.parse().map(Self::Pmcid)
        } else if s.starts_with("10.") {
            s.parse().map(Self::Doi)
        } else if let Some(doi) = s
            .strip_prefix("https://doi.org/")
            .or_else(|| s.strip_prefix("http://doi.org/"))
        {
            doi.parse().map(Self::Doi)
        } else {
            s.parse()
                .map(Self::Pmid)
                .map_err(|_| Error::ArticleIdInvalid)
        }
    }
}
impl std::fmt::Display for ArticleInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pmid(id) => id.fmt(f),
            Self::Pmcid(id) => id.fmt(f),
            Self::Doi(id) => id.fmt(f),
            Self::EuropePmc(id) => id.fmt(f),
        }
    }
}
text_value!(
    ArticleInput,
    "ArticleInput",
    format!(
        "({}|{}|(?=PPR){}|^(?:https?://doi\\.org/)?{})",
        Pmid::pattern(),
        Pmcid::pattern(),
        EuropePmcId::pattern(),
        Doi::pattern().trim_start_matches('^')
    )
);
biodata::canonical_enum! {
    /// Identifier used to resolve this article.
    ArticleMatchKind, ArticleMatchKindUnknown {
        /// PubMed identifier.
        Pmid => "pmid",
        /// PMC identifier.
        Pmcid => "pmcid",
        /// Digital object identifier.
        Doi => "doi",
        /// Europe PMC preprint identifier.
        EuropePmcId => "europepmc-id",
    }
}
impl ArticleInput {
    /// Checked namespace of the original selector.
    pub fn kind(&self) -> ArticleMatchKind {
        match self {
            Self::Pmid(_) => ArticleMatchKind::Pmid,
            Self::Pmcid(_) => ArticleMatchKind::Pmcid,
            Self::Doi(_) => ArticleMatchKind::Doi,
            Self::EuropePmc(_) => ArticleMatchKind::EuropePmcId,
        }
    }
}
