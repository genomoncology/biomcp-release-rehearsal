biodata::wire_enum! { described
    /// Explicit independent source.
    PgxSource {
        /// CPIC source-native records.
        Cpic => "cpic",
        /// ClinPGx annotation records.
        Clinpgx => "clinpgx",
    }
}
biodata::wire_enum! { described
    /// One independent named section.
    PgxSection {
        /// Complete native recommendation contexts.
        Recommendations => "recommendations",
        /// Native guideline references.
        Guidelines => "guidelines",
        /// Declared source diplotype lookup.
        Diplotype => "diplotype",
        /// One ClinPGx annotation kind.
        Annotations => "annotations",
    }
}
biodata::wire_enum! { described
    /// Independent ClinPGx collection kinds, never merged.
    PgxAnnotationKind {
        /// Clinical annotation min tier.
        Clinical => "clinical",
        /// Guideline references.
        Guideline => "guideline",
        /// Label references.
        Label => "label",
    }
}
biodata::wire_enum! { described
    /// Meaning of the stated collection total.
    PgxTotalKind {
        /// The source returned an exact total.
        Source => "source",
        /// Length of a bounded received collection.
        Observed => "observed",
    }
}

impl PgxSource {
    /// Source identity for this explicit native selector.
    pub const fn source(self) -> crate::Source {
        match self {
            Self::Cpic => crate::Source::Cpic,
            Self::Clinpgx => crate::Source::ClinPgx,
        }
    }
}

impl PgxSection {
    /// Publisher owning this independent named disclosure.
    pub const fn source(self) -> crate::Source {
        match self {
            Self::Annotations => crate::Source::ClinPgx,
            Self::Recommendations | Self::Guidelines | Self::Diplotype => crate::Source::Cpic,
        }
    }
}
