//! Canonical ols publisher declaration.
use biomcp_fetch::{KeyUse, SourceSpec, Terms};
use biomcp_types::{Source, Use};
use std::time::Duration;
/// Publisher policy shared by health, admission and response outcomes.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::Ols,
    base: "https://www.ebi.ac.uk/ols4",
    terms: Terms {
        name: "EMBL-EBI Terms of Use; ontology-specific rights",
        url: "https://www.ebi.ac.uk/about/terms-of-use/",
        usage: Use::Attribution,
    },
    quota: &crate::quotas::OLS,
    key: KeyUse::None,
    deadline: Duration::from_secs(15),
};

mod get;
mod map;
mod raw;
mod search;
pub use get::{Record, get};
pub use map::components;
pub use search::{SearchPage, search};

/// Complete source policy and command declaration.
pub(crate) mod decl;
