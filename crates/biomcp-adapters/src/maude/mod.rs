//! Canonical openFDA publisher declaration; report dispatch belongs to its adapter slice.
use biomcp_fetch::{SourceSpec, Terms};
use biomcp_types::Source;
use std::time::Duration;
/// Publisher policy shared by health, outcomes and request admission.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::Maude,
    base: "https://api.fda.gov/device/event.json",
    terms: Terms {
        name: crate::adverse_event_sources::FDA_TERMS.name,
        url: crate::adverse_event_sources::FDA_TERMS.url,
        usage: biomcp_types::Use::Attribution,
    },
    quota: &crate::quotas::OPENFDA,
    key: crate::openfda_key::OPTIONAL,
    deadline: Duration::from_secs(15),
};
mod get;
mod map;
mod raw;
mod search;
pub use get::get;
pub use search::{expression, search};

mod positions;
pub use positions::{Positions, Report};

/// Complete source policy and command declaration.
pub(crate) mod decl;
