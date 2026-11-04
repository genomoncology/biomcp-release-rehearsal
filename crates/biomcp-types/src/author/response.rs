use super::{AuthorGet, AuthorPapersQuery, AuthorSearch};
use crate::ResponseBody;
use biodata::{
    AuthorCard, AuthorHeader, AuthorId, AuthorPaperDetail, AuthorPaperHeader, OrcidWorkGroup,
};
use serde::{Deserialize, Serialize};
/// Native author candidates in provider order; search total counts only this query.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AuthorPage {
    /// Source candidates; no deduplication by display name.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[AuthorHeader]>,
    /// Native matching-record total, if supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matching_records: Option<u64>,
}
/// Exact source cards in input order; outer source outcomes retain failures.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AuthorCards {
    /// Exact returned source cards, with no cross-source links.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[AuthorCardItem]>,
}
/// One exact input and its independent outcome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AuthorCardItem {
    /// Requested qualified provider identity.
    pub input: AuthorId,
    /// This exact input's outcome.
    pub status: crate::Status,
    /// Exact checked source card when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card: Option<AuthorCard>,
}
macro_rules! papers {
    ($name:ident, $row:ty, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        pub struct $name {
            /// Requested provider record.
            pub author: AuthorId,
            /// Original source order; no native corpus total is claimed.
            #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
            pub items: Box<[$row]>,
        }
    };
}
papers!(
    AuthorPapers,
    AuthorPaperHeader,
    "Compact native indexed-paper window."
);
papers!(
    AuthorPaperDetails,
    AuthorPaperDetail,
    "Detailed metadata for the same native indexed-paper window."
);
biodata::wire_enum! {
    /// Scope of one received ORCID collection.
    OrcidWorksScope {
        /// ORCID sends the whole public works list in one bounded reply; pages are local.
        WholePublicWorksReply => "whole-public-works-reply"
    }
}
/// Locally paged groups from one complete public ORCID works reply.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OrcidWorks {
    /// The whole source reply supplies every local page.
    pub collection_scope: OrcidWorksScope,
    /// Requested exact ORCID record.
    pub author: biodata::OrcidId,
    /// Whole groups in native order, with every public assertion.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub groups: Box<[OrcidWorkGroup]>,
    /// Observed groups in the complete received reply, never a publication total.
    pub observed_groups: u64,
}
impl ResponseBody for AuthorPage {
    const SCHEMA: &'static str = "biomcp.author-page/5";
    type Query = AuthorSearch;
}
impl ResponseBody for AuthorCards {
    const SCHEMA: &'static str = "biomcp.author-card/6";
    type Query = AuthorGet;
}
impl ResponseBody for AuthorPapers {
    const SCHEMA: &'static str = "biomcp.author-papers/6";
    type Query = AuthorPapersQuery;
}
impl ResponseBody for AuthorPaperDetails {
    const SCHEMA: &'static str = "biomcp.author-paper-details/6";
    type Query = AuthorPapersQuery;
}
impl ResponseBody for OrcidWorks {
    const SCHEMA: &'static str = "biomcp.orcid-works/8";
    type Query = AuthorPapersQuery;
}
