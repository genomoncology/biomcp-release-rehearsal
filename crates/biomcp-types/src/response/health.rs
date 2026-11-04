use crate::{Reason, Source, SourceState, Text, Use};
use serde::{Deserialize, Serialize};
/// Source reachability and publisher terms, independent of local HTTP service health.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Health {
    /// Sources and carried components excluded by each supported terms group.
    #[serde(default)]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub disable_groups: Box<[DisableGroup]>,
    /// Bounded local settings notices, independent of source probes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configuration: Option<ConfigurationHealth>,
    /// Every implemented adapter, in declaration order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub sources: Box<[SourceHealth]>,
    /// File lengths measured locally without attributing shared pages to bundles.
    pub store: crate::StoreSize,
}
impl super::ResponseBody for Health {
    const SCHEMA: &'static str = "biomcp.health/6";
    type Query = super::EmptyQuery;
}
/// One bounded source probe, without credentials or provider error text.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SourceHealth {
    /// Local probe limits and recovery guidance.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notices: Vec<crate::SourceNotice>,
    /// Last validated local bundle, independent of publisher reachability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle: Option<LocalBundleState>,
    /// Per-bundle physical size is unavailable in shared storage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_size: Option<crate::BundleSize>,
    /// Independently licensed carried components, without separate network probes.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub components: Box<[ComponentHealth]>,
    /// Implemented source identity.
    pub source: Source,
    /// Publisher license or terms name.
    pub terms: Text,
    /// Publisher terms page.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub terms_url: Text,
    /// Reviewed source category, not blanket permission for each record.
    #[serde(rename = "use")]
    pub usage: Use,
    /// Whether source policy prevents all requests and cached reads.
    pub disabled: bool,
    /// Probe outcome; disabled and missing credentials remain skipped.
    pub state: SourceState,
    /// Typed failure or skip reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Reason>,
    /// Probe duration in milliseconds; omitted when policy skipped the probe.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

/// Local HTTP service health, independent of every upstream source.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HttpHealth {
    /// Executable version.
    pub version: Text,
    /// Whether the owned transport and query store opened within the health budget.
    pub store: bool,
}

/// Policy labels for one carried source component.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ComponentHealth {
    /// Native component name.
    pub name: Text,
    /// Independently reviewed publisher terms.
    pub terms: crate::ComponentTerms,
    /// Explicit source or component group policy prevents this component.
    pub disabled: bool,
}

/// The consequences of one source-disable selector, regardless of current configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DisableGroup {
    /// Terms category accepted by disable_sources.
    pub group: Use,
    /// Whole sources stopped by this group.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub sources: Box<[Source]>,
    /// Carried fields stopped while their aggregators remain available.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub components: Box<[DisabledComponent]>,
}
/// One independently labeled carried field excluded by a terms group.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DisabledComponent {
    /// Aggregator carrying the field.
    pub source: Source,
    /// Registry-owned component name.
    pub name: String,
}

/// Local configuration notices contain names and positions, never supplied entries.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ConfigurationHealth {
    /// First skipped entries in their original comma-separated order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notices: Vec<ConfigurationNotice>,
    /// Remaining skipped entries omitted from this bounded section.
    pub omitted_entries: u32,
}
/// One unsupported bypass entry skipped without changing other routing rules.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ConfigurationNotice {
    /// Environment variable owning the entry.
    pub variable: ConfigurationVariable,
    /// One-based position in the comma-separated environment setting.
    pub entry: u32,
    /// Stable notice identity.
    pub code: ConfigurationNoticeCode,
}
biodata::wire_enum! {
    /// Settings whose ignored entries can be reported safely.
    ConfigurationVariable {
        /// Uppercase proxy bypass environment setting.
        NoProxy => "NO_PROXY",
    }
}
biodata::wire_enum! {
    /// Nonfatal local configuration conditions.
    ConfigurationNoticeCode {
        /// The entry did not describe a supported host, address or range.
        UnparseableEntry => "unparseable-entry",
    }
}

biodata::wire_enum! {
/// Local bundle availability. Persisted synchronization leases do not prove process liveness.
LocalBundleState {
    /// No validated local bundle is stored.
    Missing => "missing",
    /// Stored data is within its source-owned freshness period.
    Fresh => "fresh",
    /// Stored data remains readable after its freshness period.
    Stale => "stale",
}
}

impl LocalBundleState {
    /// Source readiness needs every declared bundle; absent data wins over stale data.
    pub const fn with(self, other: Self) -> Self {
        match (self, other) {
            (Self::Missing, _) | (_, Self::Missing) => Self::Missing,
            (Self::Stale, _) | (_, Self::Stale) => Self::Stale,
            _ => Self::Fresh,
        }
    }
}
