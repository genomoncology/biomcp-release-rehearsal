use super::gene::QueryInvalid;
use crate::ArticleInput;
use serde::{Deserialize, Serialize};
/// Get one to twenty article inputs in caller order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "Fields"))]
pub struct ArticleGet(Fields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Fields {
    /// PubMed, PMC or DOI identifiers in input order.
    #[cfg_attr(feature="schema", schemars(length(min=1,max=20), extend("x-positional"=0)))]
    input: Box<[ArticleInput]>,
    /// Requested section words; trailing words and --section share this field.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    #[cfg_attr(feature = "schema", schemars(extend("x-trailing-section" = true)))]
    section: Box<[ArticleSectionWord]>,
}
crate::query::validation::checked_query!(ArticleGet, Fields);
impl TryFrom<Fields> for ArticleGet {
    type Error = QueryInvalid;
    fn try_from(fields: Fields) -> Result<Self, QueryInvalid> {
        Self::with_sections(fields.input, fields.section)
    }
}
impl ArticleGet {
    /// Check batch bounds.
    pub fn new(input: Box<[ArticleInput]>) -> Result<Self, QueryInvalid> {
        Self::with_sections(input, Box::default())
    }
    /// Check input bounds and retain section requests.
    pub fn with_sections(
        input: Box<[ArticleInput]>,
        section: Box<[ArticleSectionWord]>,
    ) -> Result<Self, QueryInvalid> {
        if !(1..=20).contains(&input.len()) {
            return Err(QueryInvalid::new(&["input"]));
        }
        Ok(Self(Fields { input, section }))
    }
    /// Requested section words.
    pub fn sections(&self) -> &[ArticleSectionWord] {
        &self.0.section
    }
    /// Original selectors.
    pub fn input(&self) -> &[ArticleInput] {
        &self.0.input
    }
}

biodata::canonical_enum! {
    /// Article section request including all landed sections.
    ArticleSectionWord, SourceUnknown {
        /// Normalized text-mined entities from PubTator3.
        Annotations => "annotations",
        /// Licensed PMC body outline.
        Fulltext => "fulltext",
        /// Supplementary files confirmed by PMC metadata.
        Supplements => "supplements",
        /// Machine-written Semantic Scholar summary and counts.
        Tldr => "tldr",
        /// Citation counts from NIH iCite.
        Citations => "citations",
        /// MeSH, keywords and author indexing from the citation.
        Indexing => "indexing",
        /// Publisher PDF link without downloading.
        Pdf => "pdf",
        /// Every landed article section.
        All => "all",
    }
}
biodata::canonical_enum! {
    /// A landed article section.
    ArticleSection, SourceUnknown {
        /// Licensed PMC body outline.
        Fulltext => "fulltext",
        /// Supplementary files confirmed by PMC metadata.
        Supplements => "supplements",
        /// Normalized text-mined entities from PubTator3.
        Annotations => "annotations",
        /// Machine-written Semantic Scholar summary and counts.
        Tldr => "tldr",
        /// Citation counts from NIH iCite.
        Citations => "citations",
        /// MeSH, keywords and author indexing from the citation.
        Indexing => "indexing",
        /// Publisher PDF link without downloading.
        Pdf => "pdf",
    }
}

impl ArticleSection {
    /// Publishers that can supply this section, depending on the input identity.
    pub const fn sources(self) -> &'static [crate::Source] {
        use crate::Source;
        match self {
            Self::Annotations => &[Source::PubTator],
            Self::Fulltext => &[Source::PmcOa, Source::EuropePmc],
            Self::Supplements => &[Source::PmcOa],
            Self::Tldr | Self::Pdf => &[Source::SemanticScholar],
            Self::Citations => &[Source::Icite],
            Self::Indexing => &[Source::PubMed],
        }
    }
}
