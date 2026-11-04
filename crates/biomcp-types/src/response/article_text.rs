use crate::{ArticleText, License, Pmcid, Text};
use serde::{Deserialize, Serialize};
biodata::canonical_enum! {
    /// Body part kind.
    TextPartKind, SourceUnknown {
        /// Top-level body section.
        Section => "section",
        /// Table, including caption and rows.
        Table => "table",
    }
}
/// One independently readable bounded part.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TextPart {
    /// One-based reader selector.
    pub number: u32,
    /// Source section title or table label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Text>,
    /// Source structure kind.
    pub kind: TextPartKind,
    /// Compact serialized text bytes divided by four, rounded up.
    pub tokens: u32,
}
/// Figure text in document order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FigureCaption {
    /// Source label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<Text>,
    /// Source caption.
    pub caption: crate::Markdown,
}
/// Bucket license and body outline; processing version is not publication chronology.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FullText {
    /// Selected bucket license.
    pub license: License,
    /// Highest numeric processing version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "u32"))]
    pub processing_version: Option<u32>,
    /// Whether this selected version is an author manuscript.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "bool"))]
    pub manuscript: Option<bool>,
    /// Bounded body parts in reading order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub parts: Box<[TextPart]>,
    /// Source figure captions in document order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub figures: Box<[FigureCaption]>,
}
/// One matching paragraph or table row, split when necessary to fit the text budget.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TextHit {
    /// Outline part containing this text.
    pub part: u32,
    /// Whole bounded source row.
    pub markdown: crate::Markdown,
    /// Section ancestry in source order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub heading_path: Box<[Text]>,
}
/// Body text response; the envelope supplies advancing reader steps.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ArticleTextResult {
    /// Requested PMC record when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<Pmcid>,
    /// Explicit section outcome including not applicable and not open.
    pub content: crate::Section<TextContent>,
}
/// One bounded part or matching rows with the bucket license.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TextContent {
    /// Source URL for an explicit save, including refused destinations.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub url: Option<Text>,
    /// Explicit save receipt, absent for reader views.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saved: Option<ArticleSaved>,
    /// Literal protein change spellings searched by an explicit variant selector.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forms: Option<[String; 4]>,
    /// Bucket license.
    pub license: License,
    /// Selected processing version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "u32"))]
    pub processing_version: Option<u32>,
    /// Author manuscript flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "bool"))]
    pub manuscript: Option<bool>,
    /// Scope of returned source text.
    pub scope: Text,
    /// Part number for a direct part selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part: Option<u32>,
    /// Bounded body text for a direct part selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<crate::Markdown>,
    /// Section ancestry for a selected part.
    #[serde(skip_serializing_if = "biodata::absent_or_empty")]
    pub heading_path: Option<Box<[Text]>>,
    /// Matching source rows for a find selection.
    #[serde(skip_serializing_if = "biodata::absent_or_empty")]
    pub hits: Option<Box<[TextHit]>>,
}
impl super::ResponseBody for ArticleTextResult {
    const SCHEMA: &'static str = "biomcp.article-text/7";
    type Query = ArticleText;
}

/// Explicit saved article file, owned by the caller.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ArticleSaved {
    /// Absolute caller destination.
    pub path: Text,
    /// Exact byte count.
    pub bytes: u64,
    /// SHA-256 of the saved bytes.
    pub sha256: Text,
    /// Source URL for the original material.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub url: Text,
    /// Number of ordered text parts; absent for original supplement bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parts: Option<u32>,
}
