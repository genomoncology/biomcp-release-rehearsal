//! Drug source declarations and shared publisher terms.
use biomcp_fetch::Terms;
use biomcp_types::Use;
/// Independent terms for the exposed MyChem projections.
pub mod components;
/// Canonical CHEMBL source declaration.
pub use crate::chembl::SPEC as CHEMBL;
/// Canonical DDINTER source declaration.
pub use crate::ddinter::SPEC as DDINTER;
/// Canonical DRUGSFDA source declaration.
pub use crate::drugsfda::SPEC as DRUGSFDA;
/// Canonical EMA source declaration.
pub use crate::ema::SPEC as EMA;
/// Canonical MYCHEM source declaration.
pub use crate::mychem::SPEC as MYCHEM;
/// Canonical OPENFDA_LABEL source declaration.
pub use crate::openfda_label::SPEC as OPENFDA_LABEL;
/// Canonical OPENTARGETS source declaration.
pub use crate::opentargets::SPEC as OPENTARGETS;
pub(crate) const FDA_TERMS: Terms = Terms {
    name: "openFDA terms; original label rights retained",
    url: "https://open.fda.gov/terms/",
    usage: Use::Open,
};
/// Direct and carried ChEMBL data share the publisher's license declaration.
pub const CHEMBL_TERMS: Terms = Terms {
    name: "ChEMBL CC BY-SA 3.0",
    url: "https://chembl.github.io/chembl-licensing/",
    usage: Use::Attribution,
};
pub(crate) const MYCHEM_TERMS: Terms = Terms {
    name: "BioThings and upstream source terms",
    url: "https://docs.mychem.info/en/latest/doc/data_source.html",
    usage: Use::Attribution,
};
pub(crate) const OPENTARGETS_TERMS: Terms = Terms {
    name: "Open Targets CC0; underlying source provenance retained",
    url: "https://platform-docs.opentargets.org/licence",
    usage: Use::Open,
};
pub(crate) const EMA_TERMS: Terms = Terms {
    name: "EMA reproduction and attribution terms; third-party exceptions",
    url: "https://www.ema.europa.eu/en/about-us/about-website/legal-notice",
    usage: Use::Attribution,
};
pub(crate) const DDINTER_TERMS: Terms = Terms {
    name: "DDInter CC BY-NC-SA 4.0",
    url: "https://ddinter.scbdd.com/terms/",
    usage: Use::Noncommercial,
};

pub use crate::received_date;

/// Preserve an adapter header as a neutral source-qualified identity.
pub fn identity(
    header: &biomcp_types::DrugHeader,
) -> Result<biodata::DrugIdentity, biodata::Error> {
    Ok(biodata::DrugIdentity {
        id: header.id.clone(),
        namespace: header.source.to_string().parse()?,
        name: header.name.clone(),
        identity_level: header.identity_level,
        r#type: header.r#type.clone(),
        url: header.url.clone(),
    })
}
