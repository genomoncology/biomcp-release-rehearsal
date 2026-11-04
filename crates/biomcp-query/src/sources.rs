//! Source declarations and access used by the command interfaces.
#![deny(clippy::wildcard_enum_match_arm)]
pub use biomcp_adapters::guidance::{WONDER_QUOTA, deadline, notices};
pub use biomcp_adapters::quotas::{configure as source_pacing, policies as pacing_policies};
pub use biomcp_adapters::registry::records::{RecordKind, declaration, record_url};
pub use biomcp_adapters::registry::{
    components, disable_groups, disabled, disabled_uses, display_name, group, key_name, key_source,
    key_suffix, key_tier, keys, outcome, requests, restore_outcome, sources, sources_for_use,
    terms,
};

pub use biomcp_adapters::decl::{BundleReply, bundles};
pub use biomcp_adapters::{Mapped, Refusal};
/// Synchronize the adapter declared by a checked catalog selector.
pub fn sync(fetch: &biomcp_fetch::Fetch, bundle: crate::Bundle) -> BundleReply {
    bundle.sync(fetch)
}

/// Configuration actions for source transport failures.
pub fn network(sources: &[biomcp_types::SourceOutcome]) -> Vec<crate::next::Step> {
    crate::next::network(sources)
}

/// Label publisher terms for an independently disclosed aggregator component.
pub use biomcp_adapters::myvariant::components::label as component_label;
