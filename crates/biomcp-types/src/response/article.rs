mod pdf;
pub use pdf::{ArticlePdfResult, OpenAccessStatus, PdfLink, PdfSaved};
mod supplement;
use crate::{ArticleHeader, ArticleSearch, ArticleSource, Doi, EuropePmcId, Pmcid, Pmid};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub use supplement::{
    ArticleSupplementResult, CellFragment, FileKind, FirstSheetRow, SheetInfo, SheetRow,
    SupplementContent, SupplementFile, Supplements,
};
/// Stable identity independent of whether citation metadata was accepted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "namespace", content = "id", rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(extend("x-identifier" = true)))]
pub enum ArticleKey {
    /// PubMed identity.
    Pubmed(Pmid),
    /// PMC identity when PubMed is absent.
    Pmc(Pmcid),
    /// DOI when both indexed numeric identities are absent.
    Doi(Doi),
    /// Europe PMC local source identity.
    Europepmc(EuropePmcId),
    /// Semantic Scholar local identity when no indexed identifier exists.
    SemanticScholar(crate::SemanticScholarId),
}
/// One original pool position, including a summary hole.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ArticleItem {
    /// Stable checked paper identity when the provider supplies one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "ArticleKey"))]
    pub key: Option<ArticleKey>,
    /// The selected PMID, retained when summary mapping fails.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Pmid"))]
    pub pmid: Option<Pmid>,
    /// Native one-based source positions, disclosed only for multiple sources.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub found_by: BTreeMap<ArticleSource, u32>,
    /// iCite citing-paper count, only under citation ordering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cited_by: Option<u64>,
    /// First matching sentence in LitSense2 provider order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sentence: Option<ArticleSentence>,
    /// Citation intents in provider order; absent means not reported.
    #[serde(default, skip_serializing_if = "biodata::absent_or_empty")]
    pub intents: Option<Box<[CitationIntent]>>,
    /// Provider's influential flag; absent is distinct from false.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub influential: Option<bool>,
    /// First citation context and the source's complete context count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contexts: Option<crate::Capped<crate::Text>>,
    /// Citation metadata when accepted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<ArticleHeader>,
}
/// A slice of the stored article source order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ArticlePage {
    /// Original ordered items.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[ArticleItem]>,
    /// Nearby source candidates when an exact variant was not uniquely selected.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<LitVarCandidate>,
}
impl super::ResponseBody for ArticlePage {
    const SCHEMA: &'static str = "biomcp.article-page/9";
    type Query = ArticleSearch;
}

/// A page of citing or referenced papers in citation order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ArticleEdgePage {
    /// Papers in the fixed edge pool.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[ArticleEdgeItem]>,
}
impl super::ResponseBody for ArticleEdgePage {
    const SCHEMA: &'static str = "biomcp.article-edges/10";
    type Query = crate::ArticleEdges;
}

/// Stored edge item shares checked identity and hydration with search items.
pub type ArticleEdgeItem = ArticleItem;

biodata::canonical_enum! {
    /// Source-reported reason for citing a paper.
    CitationIntent, SourceUnknown {
        /// The source cites background information.
        Background => "background",
        /// The source cites a reported result.
        Result => "result",
        /// The source cites a method.
        Methodology => "methodology",
    }
}

/// Public LitVar2 autocomplete candidate, without a clinical ranking claim.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct LitVarCandidate {
    /// Source rsID, when the source associates one with this variant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "crate::Text"))]
    pub rsid: Option<crate::Text>,
    /// Source gene symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gene: Vec<crate::Text>,
    /// Source variant name, when independently admitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<crate::Text>,
    /// Number of associated publications reported by LitVar2, without inferred totals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
}

mod evidence;
pub use evidence::{
    ArticleEvidenceResult, ArticleRecommendations, CitationContexts, CitationParagraphs,
    EvidenceContext, EvidenceNotice, EvidenceParagraph,
};
/// Source-provided sentence resemblance, without a claim of clinical relevance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ArticleSentence {
    /// Source section, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<crate::Text>,
    /// Source sentence text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<crate::Text>,
}
