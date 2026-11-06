biodata::wire_enum! { described
    /// An explicit source; never probe the other family.
    AdverseEventSource {
        /// Drug event reports.
        Faers => "faers",
        /// Device event reports.
        Maude => "maude",
        /// Explicit CDC WONDER vaccine aggregate tables.
        Vaers => "vaers",
    }
}
biodata::wire_enum! { described
    /// Report-level seriousness filter, independent of reaction outcome.
    AdverseEventSerious {
        /// Native report seriousness flag.
        Any => "any",
        /// Native report death flag.
        Death => "death",
        /// Native report hospitalization flag.
        Hospitalization => "hospitalization",
        /// Native report life-threatening flag.
        Lifethreatening => "lifethreatening",
        /// Native report disabling flag.
        Disabling => "disabling",
        /// Native report congenital anomaly flag.
        CongenitalAnomaly => "congenital-anomaly",
        /// Native report other serious flag.
        Other => "other",
    }
}
biodata::wire_enum! { described
    /// One reviewed identifier collection owned by one drug.
    AdverseEventIdentifierSection {
        /// Native UNII strings.
        Unii => "unii",
        /// Native RxCUI strings.
        Rxcui => "rxcui",
        /// Native product NDC strings.
        ProductNdc => "product-ndc",
        /// Native package NDC strings.
        PackageNdc => "package-ndc",
        /// Native SPL IDs.
        SplId => "spl-id",
        /// Native SPL set IDs.
        SplSetId => "spl-set-id",
        /// Native application numbers.
        ApplicationNumber => "application-number",
    }
}
biodata::wire_enum! { described
    /// One source-owned section; no all or reporter section.
    AdverseEventSection {
        /// Independent native drug rows.
        Drugs => "drugs",
        /// Independent native reaction rows.
        Reactions => "reactions",
        /// Native UNII strings.
        Unii => "unii",
        /// Native RxCUI strings.
        Rxcui => "rxcui",
        /// Native product NDC strings.
        ProductNdc => "product-ndc",
        /// Native package NDC strings.
        PackageNdc => "package-ndc",
        /// Native SPL IDs.
        SplId => "spl-id",
        /// Native SPL set IDs.
        SplSetId => "spl-set-id",
        /// Native application numbers.
        ApplicationNumber => "application-number",
        /// Native device rows.
        Devices => "devices",
        /// Complete native narrative rows.
        Narrative => "narrative",
    }
}
impl AdverseEventSection {
    /// Terminology required before reading this whole section, including stored rows.
    pub const fn required_component(self) -> Option<crate::Source> {
        match self {
            Self::Reactions => Some(crate::Source::Meddra),
            Self::Devices => Some(crate::Source::Gmdn),
            _ => None,
        }
    }
    /// Whether this section requires an absolute native drug index.
    pub fn is_identifier(self) -> bool {
        !matches!(
            self,
            Self::Drugs | Self::Reactions | Self::Devices | Self::Narrative
        )
    }
    /// Source owning this section.
    pub fn source(self) -> AdverseEventSource {
        if matches!(self, Self::Devices | Self::Narrative) {
            AdverseEventSource::Maude
        } else {
            AdverseEventSource::Faers
        }
    }
}
biodata::wire_enum! { described
    /// The count describes reports, never exposed people or causal events.
    ReportUnit {
        /// Native report count without a population denominator.
        Reports => "reports",
    }
}

impl AdverseEventSource {
    /// Component admitted for every search of this source, independent of filters.
    pub const fn required_search_component(self) -> Option<crate::Source> {
        match self {
            Self::Maude => AdverseEventSection::Devices.required_component(),
            Self::Faers | Self::Vaers => None,
        }
    }
    #[cfg(feature = "schema")]
    pub(crate) fn search_components() -> std::collections::BTreeMap<String, crate::Source> {
        schemars::schema_for!(Self)
            .get("enum")
            .and_then(|value| value.as_array())
            .into_iter()
            .flatten()
            .filter_map(|value| value.as_str()?.parse::<Self>().ok())
            .filter_map(|source| {
                source
                    .required_search_component()
                    .map(|component| (source.to_string(), component))
            })
            .collect()
    }
    /// Source identity for this explicit native selector.
    pub const fn source(self) -> crate::Source {
        match self {
            Self::Faers => crate::Source::Faers,
            Self::Maude => crate::Source::Maude,
            Self::Vaers => crate::Source::Vaers,
        }
    }
}
biodata::wire_enum! { described
    /// Source-native aggregation of matching FAERS reports.
    AdverseEventAggregate {
        /// Exact reaction-term buckets from openFDA; reports may contribute to several terms.
        Reactions => "reactions",
    }
}
