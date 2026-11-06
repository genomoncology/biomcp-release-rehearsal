//! Component terms have one home for queries, filters, labels and health.

/// Shared independent component declaration.
pub use crate::registry::Component;
use biomcp_fetch::Terms;
use biomcp_types::{Source, Use};
/// CADD data retain their published noncommercial terms.
pub const CADD: Component = Component {
    source: Source::MyVariant,
    native_source: None,
    name: "cadd",
    terms: Terms {
        name: "CADD noncommercial data terms",
        url: "https://cadd.gs.washington.edu/download",
        usage: Use::Noncommercial,
    },
};
/// The captured dbNSFP academic branch retains its component restrictions.
pub const DBNSFP: Component = Component {
    source: Source::MyVariant,
    native_source: None,
    name: "dbnsfp",
    terms: Terms {
        name: "dbNSFP academic branch and individual component terms",
        url: "https://sites.google.com/site/jpopgen/dbNSFP",
        usage: Use::Noncommercial,
    },
};
/// MyVariant's historical COSMIC copy supplies no blanket reuse grant.
pub const COSMIC: Component = Component {
    source: Source::MyVariant,
    native_source: None,
    name: "cosmic",
    terms: Terms {
        name: "Historical COSMIC snapshot terms",
        url: "https://www.cosmickb.org/terms/",
        usage: Use::Licensed,
    },
};
/// ClinVar retains the independently reviewed submission-specific rights label.
pub const CLINVAR: Component = Component {
    source: Source::MyVariant,
    native_source: Some(Source::ClinVar),
    name: "clinvar",
    terms: Terms {
        name: "NCBI policy; submission rights remain source-specific",
        url: "https://www.ncbi.nlm.nih.gov/clinvar/docs/maintenance_use/",
        usage: Use::Open,
    },
};
/// The carried SnpEff annotation keeps its own publisher terms.
pub const SNPEFF: Component = Component {
    source: Source::MyVariant,
    native_source: None,
    name: "snpeff",
    terms: Terms {
        name: "SnpEff MIT",
        url: "https://github.com/pcingola/SnpEff/blob/master/LICENSE.md",
        usage: Use::Open,
    },
};
/// Component declarations used consistently by health and named sections.
/// SNPedia retains its noncommercial reuse terms.
pub const SNPEDIA: Component = Component {
    source: Source::MyVariant,
    native_source: None,
    name: "snpedia",
    terms: Terms {
        name: "SNPedia CC BY-NC-SA",
        url: "https://www.snpedia.com/index.php/SNPedia:General_disclaimer",
        usage: Use::Noncommercial,
    },
};
/// Component declarations used consistently by health and named sections.
pub const COMPONENTS: &[Component] = &[CADD, DBNSFP, COSMIC, CLINVAR, SNPEFF, SNPEDIA];
/// Resolve component terms from their canonical declaration.
pub fn terms(name: &str) -> Option<Terms> {
    COMPONENTS.iter().find(|c| c.name == name).map(|c| c.terms)
}

/// Convert a declaration to its public response label without repeating its values.
pub fn label(terms: Terms) -> Result<biomcp_types::ComponentTerms, biomcp_fetch::InputError> {
    Ok(biomcp_types::ComponentTerms {
        usage: terms.usage,
        name: biodata::Text::new(terms.name)
            .map_err(|_| biomcp_fetch::InputError::Serialization)?,
        url: biodata::Text::new(terms.url).map_err(|_| biomcp_fetch::InputError::Serialization)?,
    })
}

/// All components carried by the aggregator.
pub fn all() -> &'static [Component] {
    COMPONENTS
}
/// Whether source or component policy blocks a carried component.
pub fn disabled(fetch: &biomcp_fetch::Fetch, name: &str) -> bool {
    policy(
        name,
        |source| fetch.disabled(source),
        |usage| fetch.disabled_use(usage),
    )
}
/// Inspect that same component policy from a deadline-scoped adapter leg.
pub fn disabled_on_leg(leg: &biomcp_fetch::Leg<'_>, name: &str) -> bool {
    policy(
        name,
        |source| leg.disabled_source(source),
        |usage| leg.disabled_use(usage),
    )
}
fn policy(
    name: &str,
    source_disabled: impl Fn(Source) -> bool,
    use_disabled: impl Fn(Use) -> bool,
) -> bool {
    source_disabled(Source::MyVariant)
        || COMPONENTS
            .iter()
            .find(|c| c.name == name)
            .is_some_and(|c| c.policy(source_disabled, use_disabled))
}
/// Opaque source queries cannot be safely scoped when a component is switched off.
pub fn restricted(fetch: &biomcp_fetch::Fetch) -> bool {
    !fetch.disabled(Source::MyVariant)
        && all()
            .iter()
            .any(|component| disabled(fetch, component.name))
}
