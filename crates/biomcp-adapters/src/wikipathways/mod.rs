//! Publisher policy for explicit WikiPathways catalog synchronization.
mod catalog;
mod catalog_row;
mod gpml;
mod gpml_raw;
pub use gpml::{Members, members};
mod raw;
use biomcp_fetch::{KeyUse, SourceSpec, Terms};
use biomcp_types::{Source, Use};
pub use catalog::{card, search, sync};
use std::time::Duration;

/// One declaration owns publisher terms, admission and bundle download policy.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::WikiPathways,
    base: "https://www.wikipathways.org",
    terms: Terms {
        name: "CC0 1.0 waiver",
        url: "https://www.wikipathways.org/help.html",
        usage: Use::Open,
    },
    key: KeyUse::None,
    quota: &crate::quotas::WIKIPATHWAYS,
    deadline: Duration::from_secs(120),
};

pub(crate) use catalog::bundle_state;

/// Complete source policy and command declaration.
pub(crate) mod decl;
