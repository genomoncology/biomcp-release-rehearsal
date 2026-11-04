//! Reviewed component rights and pre-I/O narrowing, without inferred payload rights.
use crate::registry::Component;
use biomcp_fetch::{Leg, RequestError, SourceError, Terms};
use biomcp_types::{DiseaseOntology, Reason, Source, Use};
pub(crate) const MONDO: Terms = Terms {
    name: "MONDO CC BY 4.0",
    url: "https://mondo.monarchinitiative.org/pages/download/",
    usage: Use::Attribution,
};
pub(crate) const DOID: Terms = Terms {
    name: "Disease Ontology CC0 1.0",
    url: "https://www.disease-ontology.org/resources/do-resources/",
    usage: Use::Open,
};
pub(crate) const NCIT: Terms = Terms {
    name: "NCI Thesaurus CC BY 4.0",
    url: "https://evs.nci.nih.gov/ftp1/NCI_Thesaurus/ThesaurusTermsofUse.pdf",
    usage: Use::Attribution,
};
pub(crate) const HPO: Terms = Terms {
    name: "HPO project custom license: attribution, version and integrity",
    url: "https://human-phenotype-ontology.github.io/license.html",
    usage: Use::Attribution,
};
pub(crate) const ORPHANET: Terms = Terms {
    name: "Orphadata Science CC BY 4.0 and citation/version conditions",
    url: "https://sciences.orphadata.com/faq/",
    usage: Use::Attribution,
};
pub(crate) const OMIM: Terms = Terms {
    name: "OMIM Use Agreement; separately licensed payload only",
    url: "https://omim.org/help/agreement",
    usage: Use::Licensed,
};
pub(crate) const fn component(
    source: Source,
    native_source: Source,
    name: &'static str,
    terms: Terms,
) -> Component {
    Component {
        source,
        native_source: Some(native_source),
        name,
        terms,
    }
}
/// Find the reviewed component declaration for this service.
pub fn get(source: Source, name: &str) -> Option<&'static Component> {
    all().find(|c| c.source == source && c.name == name)
}
fn disabled(source: Source) -> RequestError {
    SourceError {
        source,
        reason: Reason::Disabled,
        status: None,
        tls_cause: None,
        retry_after: None,
    }
    .into()
}
/// Admit exactly the requested OLS ontology; a disabled default has no fallback.
pub fn ontology(leg: &Leg<'_>, ontology: DiseaseOntology) -> Result<DiseaseOntology, RequestError> {
    if get(Source::Ols, ontology.as_str()).is_none_or(|c| c.disabled_on_leg(leg)) {
        Err(disabled(Source::Ols))
    } else {
        Ok(ontology)
    }
}
/// Omit disabled and unreviewed MyDisease projections before request identity exists.
/// Identity fields alone cannot admit a request with no allowed content projection.
pub fn allowed_fields(leg: &Leg<'_>, fields: &str) -> Result<String, RequestError> {
    let selected = fields
        .split(',')
        .filter(|field| {
            let root = field.split('.').next().unwrap_or(field);
            // MyDisease publishes Disease Ontology under this native field root.
            let name = if root == "disease_ontology" {
                Source::Doid.into()
            } else {
                root
            };
            (root != <&str>::from(Source::Doid))
                && get(Source::MyDisease, name).is_some_and(|c| !c.disabled_on_leg(leg))
        })
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return Err(disabled(Source::MyDisease));
    }
    Ok(fields
        .split(',')
        .filter(|f| matches!(*f, "_id" | "_score") || selected.contains(f))
        .collect::<Vec<_>>()
        .join(","))
}
/// Mixed Monarch sections have no verified upstream filter: skip before any I/O.
/// The reviewed HPO grant includes cited OMIM/Orphanet provenance; OMIM payload
/// restrictions alone therefore never remove these annotations.
pub fn monarch(leg: &Leg<'_>) -> Result<(), RequestError> {
    if ["hpo", "orphanet"]
        .iter()
        .any(|name| get(Source::Monarch, name).is_none_or(|c| c.disabled_on_leg(leg)))
    {
        Err(disabled(Source::Monarch))
    } else {
        Ok(())
    }
}

/// Enumerate independently labeled fields from their adapter declarations.
pub fn all() -> impl Iterator<Item = &'static Component> {
    crate::registry::components().filter(|component| {
        matches!(
            component.source,
            Source::Ols | Source::MyDisease | Source::Monarch
        )
    })
}
