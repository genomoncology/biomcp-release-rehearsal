use crate::{Capped, CitationIntent, License, Section};
use serde::{Deserialize, Serialize};
/// One advancing piece of an original source citation context.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EvidenceContext {
    /// One-based original source context position.
    pub position: u32,
    /// Unicode character position within that context.
    pub character_offset: u32,
    /// Whole context, or an exact advancing fragment when the context exceeds a page.
    #[cfg_attr(feature = "schema", schemars(length(min = 1)))]
    pub text: Box<str>,
}
/// One source paragraph fragment with its own exact outline location.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EvidenceParagraph {
    /// One-based original body paragraph position.
    pub position: u32,
    /// Exact semantic outline part.
    pub part: u32,
    /// Unicode character position within the original paragraph.
    pub character_offset: u32,
    /// Preserved source Markdown.
    #[cfg_attr(feature = "schema", schemars(length(min = 1), extend("format" = "markdown")))]
    pub markdown: Box<str>,
}
biodata::wire_enum! {
    /// Explicit successful empty outcomes, never inferred from a source failure.
    EvidenceNotice {
        /// Terminal graph scan found no matching reference.
        NotInReferences => "not-in-references",
        /// The matching reference reports no contexts.
        NoContexts => "no-contexts",
        /// The source reports no recommendations.
        NoRecommendations => "no-recommendations",
    }
}
/// Source edge metadata and a bounded advancing context page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CitationContexts {
    /// Every context remains accessible through the advancing step.
    pub contexts: Capped<EvidenceContext>,
    /// Completion means a terminal source page was reached.
    pub complete: bool,
    /// Native next source offset when a scan is incomplete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_cursor: Option<u32>,
    /// Source-reported citation intents.
    #[serde(skip_serializing_if = "biodata::absent_or_empty")]
    pub intents: Option<Box<[CitationIntent]>>,
    /// Missing is distinct from false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub influential: Option<bool>,
    /// Successful empty-source explanation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notice: Option<EvidenceNotice>,
}
/// Exact JATS reference paragraphs independent of the graph result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CitationParagraphs {
    /// Original publisher license category.
    pub license: License,
    /// Original paragraph fragments and complete fragment count.
    pub paragraphs: Capped<EvidenceParagraph>,
}
/// Independent graph and optional open-text evidence parts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ArticleEvidenceResult {
    /// Source-reported citation contexts.
    pub semantic_scholar: Section<CitationContexts>,
    /// The citing paper's own explicit reference paragraphs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulltext: Option<Section<CitationParagraphs>>,
}
impl super::super::ResponseBody for ArticleEvidenceResult {
    const SCHEMA: &'static str = "biomcp.article-evidence/6";
    type Query = crate::ArticleEvidence;
}
/// Source-native recommendation page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ArticleRecommendations {
    /// Papers in provider order, without a local topic filter.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[crate::ArticleItem]>,
    /// Provider's default pool, stated without a clinical relevance claim.
    pub scope: Box<str>,
    /// Successful zero-result explanation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notice: Option<EvidenceNotice>,
}
impl super::super::ResponseBody for ArticleRecommendations {
    const SCHEMA: &'static str = "biomcp.article-recommendations/9";
    type Query = crate::ArticleRecommend;
}
