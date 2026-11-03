//! Canonical pharmacogenomics publisher declaration; runtime dispatch arrives separately.
use biomcp_fetch::{KeyUse, SourceSpec, Terms};
use biomcp_types::Source;
use std::time::Duration;
/// Publisher policy for health, outcomes and admission before any I/O.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::Cpic,
    base: "https://api.cpicpgx.org/v1",
    terms: Terms {
        name: "CPIC data use policy",
        url: "https://cpicpgx.org/about/terms-of-use/",
        usage: biomcp_types::Use::Open,
    },
    quota: &crate::quotas::CPIC,
    key: KeyUse::None,
    deadline: Duration::from_secs(15),
};
mod raw;
mod read;
pub use read::{Page, card, diplotype, drug_recommendations, guidelines, recommendations, search};

/// Native gene collection shared by record reads and the bounded health probe.
pub(crate) const GENE_PATH: &str = "/gene";

/// Complete source policy and command declaration.
pub(crate) mod decl;
