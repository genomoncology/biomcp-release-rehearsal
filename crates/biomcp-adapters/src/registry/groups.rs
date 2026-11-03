//! Registry-derived consequences of supported source-disable groups.
use super::{components, sources as registered_sources};
use biomcp_types::{DisableGroup, DisabledComponent, Source, Use};
use std::collections::BTreeSet;

pub(super) const USES: &[Use] = &[Use::Open, Use::Noncommercial];

/// List whole-source exclusions separately from independently labeled carried fields.
pub fn disable_groups() -> Vec<DisableGroup> {
    USES.iter()
        .map(|&group| DisableGroup {
            group,
            sources: sources_for_use(group).into_iter().collect(),
            components: components()
                .filter(|c| c.terms.usage == group)
                .map(|c| DisabledComponent {
                    source: c.source,
                    name: c.name.into(),
                })
                .collect(),
        })
        .collect()
}

/// Whole sources in one use category, without disabling their aggregators.
pub fn sources_for_use(group: Use) -> BTreeSet<Source> {
    registered_sources()
        .filter(|s| s.terms.usage == group && !components().any(|c| c.source == s.source))
        .map(|s| s.source)
        .collect()
}
