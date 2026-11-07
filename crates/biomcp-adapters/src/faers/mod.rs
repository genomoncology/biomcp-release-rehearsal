//! Source-native FAERS report discovery and its canonical publisher declaration.
use biomcp_fetch::{SourceSpec, Terms};
use biomcp_types::Source;
use std::time::Duration;
/// Publisher policy shared by health, outcomes and request admission.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::Faers,
    base: "https://api.fda.gov/drug/event.json",
    terms: Terms {
        name: crate::adverse_event_sources::FDA_TERMS.name,
        url: crate::adverse_event_sources::FDA_TERMS.url,
        usage: biomcp_types::Use::Attribution,
    },
    quota: &crate::quotas::OPENFDA,
    key: crate::openfda_key::OPTIONAL,
    deadline: Duration::from_secs(15),
};
mod counts;
mod raw;
mod search;
pub use counts::{CountPage, reactions};
pub use search::{Page, expression, search};

pub mod detail;
mod detail_map;
mod detail_raw;

mod positions;
pub mod words;
pub use positions::Positions;

/// Complete source policy and command declaration.
pub(crate) mod decl;
