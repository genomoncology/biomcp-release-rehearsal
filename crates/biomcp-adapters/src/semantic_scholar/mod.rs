//! Semantic Scholar graph metadata under one shared optional-key quota.
use biomcp_fetch::{Credential, KeyPlace, KeyUse, SecretName, SourceSpec};
use biomcp_types::Source;
use std::time::Duration;
/// Shared policy for search, paper records, graph edges and recommendations.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "Semantic Scholar API License Agreement",
        url: "https://api.semanticscholar.org/license/",
        usage: biomcp_types::Use::Noncommercial,
    },
    source: Source::SemanticScholar,
    base: "https://api.semanticscholar.org/graph/v1",
    key: KeyUse::Optional(Credential {
        tier: biomcp_types::KeyTier::Fallback,
        secret: SecretName::SemanticScholarApiKey,
        place: KeyPlace::Header("x-api-key"),
    }),
    quota: &crate::quotas::SEMANTIC_SCHOLAR,
    deadline: Duration::from_secs(15),
};

mod search;

pub(crate) mod pdf;
pub use pdf::{PUBLISHER, PdfLink, PdfRecord, pdf};
mod graph;
pub use graph::{Edge, Edges, PaperSummary, edges, paper_endpoint, summary};
mod query;
pub use query::search_query;
pub use search::{Author, ExternalIds, Paper, Search, search};

pub(crate) mod recommend;
pub use recommend::{RECOMMEND_SPEC, Recommendations, paper_id, recommendations};
/// Provider-local author discovery and exact metadata.
pub mod author;

/// Complete source policy and command declaration.
pub(crate) mod decl;
