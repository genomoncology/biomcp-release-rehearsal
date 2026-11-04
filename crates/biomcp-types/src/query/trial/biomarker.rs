biodata::wire_enum! {
    /// NCI's native biomarker eligibility criterion, without clinical interpretation.
    NciBiomarkerUse {
        /// NCI lists a concept in inclusion criteria.
        Inclusion => "inclusion",
        /// NCI lists a concept in exclusion criteria.
        Exclusion => "exclusion",
    }
}
