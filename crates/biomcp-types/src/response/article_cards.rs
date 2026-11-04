use crate::{Article, ArticleGet, ArticleInput, ArticleMatchKind, Capped, ItemStatus, Text};
use serde::{Deserialize, Serialize};
/// Result of resolving an article selector.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "resolution", rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ArticleResolution {
    /// A record whose own identity agrees with the input.
    Resolved {
        /// Source card with capped authors.
        card: Article<Capped<Text>>,
        /// Identity evidence.
        r#match: ArticleMatch,
        /// Original fetch time in Unix seconds.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fetched: Option<u64>,
    },
    /// The source confirmed no matching record.
    NotFound,
}
/// Identity evidence for an article card.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ArticleMatch {
    /// Namespace used to resolve the card.
    pub kind: ArticleMatchKind,
}
/// Answer to one input; shared DOIs may expand to several consecutive items.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ArticleCardItem {
    /// Original selector.
    pub input: ArticleInput,
    /// This card's outcome.
    pub status: ItemStatus,
    /// Absent only when a provider failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer: Option<ArticleResolution>,
    /// Requested section states for a resolved card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<crate::ArticleSections>,
    /// Applicable sections omitted by this request.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub more: Box<[crate::ArticleSection]>,
}
/// Cards in caller order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ArticleCards {
    /// Per-input answers.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[ArticleCardItem]>,
}
impl super::ResponseBody for ArticleCards {
    const SCHEMA: &'static str = "biomcp.article-cards/9";
    type Query = ArticleGet;
}
