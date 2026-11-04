//! DDInter native pair rows and the canonical publisher declaration.
mod identity;
mod pairs;
use identity::candidates;
mod bundle;
mod raw;
use biomcp_fetch::{KeyUse, SourceSpec, Terms};
use biomcp_types::Source;
pub use bundle::{card, interactions, search, sync};
use pairs::{PartRow, partners};
use raw::NativeRow;
use std::time::Duration;
/// Publisher policy shared by health, request admission and response outcomes.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::Ddinter,
    base: "https://ddinter.scbdd.com",
    terms: Terms {
        name: crate::drug_sources::DDINTER_TERMS.name,
        url: crate::drug_sources::DDINTER_TERMS.url,
        usage: crate::drug_sources::DDINTER_TERMS.usage,
    },
    quota: &crate::quotas::DDINTER,
    key: KeyUse::None,
    deadline: Duration::from_secs(15),
};

pub(crate) use bundle::bundle_state;

/// Complete source policy and command declaration.
pub(crate) mod decl;
