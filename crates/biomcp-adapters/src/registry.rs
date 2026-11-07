//! Implemented source declarations are the sole registry for terms and source switches.

mod component;
mod groups;
pub use groups::{disable_groups, sources_for_use};
mod names;
mod probe;
pub use names::{discovery_aliases, display_name};
pub use probe::probe;
/// Canonical publisher record links and explicit uncitable ID kinds.
pub mod records;
pub use component::Component;

pub use crate::decl::{key_name, key_source, key_suffix, keys, sources};
use biomcp_fetch::SourceSpec;
use biomcp_types::{Source, Use};
use std::collections::BTreeSet;
/// Enumerate direct and auxiliary request contracts from their owning adapters.
pub fn requests() -> impl Iterator<Item = &'static SourceSpec> {
    sources()
        .chain(sources().flat_map(|spec| crate::decl::decl(spec.source).auxiliary.iter().copied()))
}
/// Declared credential policy shared by hosted admission and key discovery.
/// Multiple APIs using one credential must retain the strongest declared tier.
pub fn key_tier(name: biomcp_fetch::SecretName) -> biomcp_types::KeyTier {
    if requests().any(|spec| {
        matches!(spec.key,
        biomcp_fetch::KeyUse::Optional(c) | biomcp_fetch::KeyUse::Required(c)
            if c.secret == name && c.tier == biomcp_types::KeyTier::NoFallback)
    }) {
        biomcp_types::KeyTier::NoFallback
    } else {
        biomcp_types::KeyTier::Fallback
    }
}
/// Independently labeled fields from every implemented aggregator.
pub fn components() -> impl Iterator<Item = &'static Component> {
    sources().flat_map(|spec| crate::decl::decl(spec.source).components.iter())
}
/// Resolve one supported terms group.
pub fn group(name: &str) -> Option<Use> {
    groups::USES
        .iter()
        .copied()
        .find(|usage| usage.as_str() == name)
}
/// Preserve group restrictions for independently labeled aggregator components.
pub fn disabled_uses(names: &[String]) -> BTreeSet<Use> {
    names.iter().filter_map(|name| group(name)).collect()
}
/// Every source name a switch accepts: direct sources, then switchable components.
pub fn switchable() -> Vec<Source> {
    let mut seen = BTreeSet::new();
    sources()
        .map(|spec| spec.source)
        .chain(components().filter_map(|component| component.native_source))
        .filter(|&source| seen.insert(source))
        .collect()
}
/// Resolve one source name; terms groups are not source names.
pub fn switchable_source(name: &str) -> Option<Source> {
    switchable()
        .into_iter()
        .find(|source| source.to_string() == name)
}
/// Expand source switches while component declarations keep aggregators available.
pub fn disabled(names: &[String]) -> Result<BTreeSet<Source>, biodata::Error> {
    let mut disabled = BTreeSet::new();
    for name in names {
        if let Some(usage) = group(name) {
            disabled.extend(groups::sources_for_use(usage));
        } else {
            disabled.insert(switchable_source(name).ok_or(biodata::Error::SourceUnknown)?);
        }
    }
    Ok(disabled)
}

/// Publisher terms for a direct service or its independently switchable component.
pub fn terms(source: Source) -> Option<biomcp_fetch::Terms> {
    Some(crate::decl::decl(source).spec.terms)
}

/// Construct every outcome with the same publisher declaration used by health.
pub fn outcome(source: Source, status: biomcp_types::SourceStatus) -> biomcp_types::SourceOutcome {
    let terms = crate::decl::decl(source).spec.terms;
    biomcp_types::SourceOutcome::new(
        source,
        terms.usage,
        terms.name.into(),
        terms.url.into(),
        status,
    )
}

/// Reapply current publisher labels when an outcome is restored from a snapshot.
pub fn restore_outcome(mut saved: biomcp_types::SourceOutcome) -> biomcp_types::SourceOutcome {
    let current = outcome(saved.source, saved.status());
    saved.usage = current.usage;
    saved.terms = current.terms;
    saved.terms_url = current.terms_url;
    saved
}

/// Source-owned local set metadata, separate from the online publisher probe.
pub fn bundle_state(
    spec: &'static SourceSpec,
    leg: &biomcp_fetch::Leg<'_>,
) -> Result<Option<biomcp_types::LocalBundleState>, biomcp_fetch::RequestError> {
    crate::decl::decl(spec.source)
        .bundles
        .first()
        .map(|bundle| (bundle.state)(leg))
        .transpose()
}
