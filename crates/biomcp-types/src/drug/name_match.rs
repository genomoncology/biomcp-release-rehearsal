//! Native fields establishing complete label-name equality.
biodata::canonical_enum! {
    /// Complete native name fields matching the requested label name.
    DrugNameMatch, SourceUnknown {
        /// The complete brand name matches.
        Brand => "brand",
        /// The complete generic name matches.
        Generic => "generic",
        /// Both complete name fields match.
        Both => "both",
    }
}

impl DrugNameMatch {
    /// Case-insensitive equality of complete source names, preserving spacing and punctuation.
    pub fn same_name(value: &str, input: &str) -> bool {
        value.to_lowercase() == input.to_lowercase()
    }
    /// Retain the source fields establishing equality.
    pub fn from_fields(brand: bool, generic: bool) -> Option<Self> {
        match (brand, generic) {
            (true, true) => Some(Self::Both),
            (true, false) => Some(Self::Brand),
            (false, true) => Some(Self::Generic),
            _ => None,
        }
    }
}

impl DrugNameMatch {
    /// The approved single trailing salt or ester terms; this list does not imply chemical equivalence.
    pub const SALT_OR_ESTER_WORDS: [&'static str; 17] = [
        "mesylate",
        "hydrochloride",
        "sodium",
        "potassium",
        "calcium",
        "sulfate",
        "maleate",
        "tartrate",
        "citrate",
        "phosphate",
        "acetate",
        "besylate",
        "fumarate",
        "succinate",
        "tosylate",
        "bromide",
        "chloride",
    ];
    /// Primary USAN nomenclature guidance for a single modifying salt or ester term.
    pub const NAMING_GUIDANCE: &'static str = "https://www.ama-assn.org/about/united-states-adopted-names-usan/united-states-adopted-names-naming-guidelines";
    /// Exact generic name or the input followed by one approved term, with no recursive stripping.
    pub fn generic_name(value: &str, input: &str) -> bool {
        Self::same_name(value, input)
            || value.rsplit_once(' ').is_some_and(|(base, suffix)| {
                Self::same_name(base, input)
                    && Self::SALT_OR_ESTER_WORDS
                        .iter()
                        .any(|word| Self::same_name(suffix, word))
            })
    }
}
