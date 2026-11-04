//! Canonical pharmacogenomics publisher declaration; runtime dispatch arrives separately.
use biomcp_fetch::{KeyUse, SourceSpec, Terms};
use biomcp_types::Source;
use std::time::Duration;
/// Publisher policy for health, outcomes and admission before any I/O.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::ClinPgx,
    base: "https://api.clinpgx.org/v1",
    terms: Terms {
        name: crate::pgx_sources::TERMS.name,
        url: crate::pgx_sources::TERMS.url,
        usage: crate::pgx_sources::TERMS.usage,
    },
    quota: &crate::quotas::CLINPGX,
    key: KeyUse::None,
    deadline: Duration::from_secs(15),
};

mod annotation_raw;
mod detail_raw;
mod read;
pub use read::{annotation, clinical, references};

/// Native clinical annotation collection shared by reads and the bounded health probe.
pub(crate) const CLINICAL_PATH: &str = "/data/clinicalAnnotation";

/// Complete source policy and command declaration.
pub(crate) mod decl;
