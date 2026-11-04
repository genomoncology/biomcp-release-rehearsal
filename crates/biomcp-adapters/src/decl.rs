//! Every source owns one complete adapter declaration.
#![deny(clippy::wildcard_enum_match_arm)]
pub(crate) mod bundle;
mod probe;
use crate::registry::Component;
use biomcp_fetch::{KeyUse, SecretName, SourceSpec, Terms};
use biomcp_types::Source;
pub use bundle::{Bundle, BundleDecl, BundleReply, bundles};
pub use probe::Probe;
/// Source-specific request guidance.
#[derive(Debug)]
pub struct Guidance {
    /// Request deadline explanation, when the provider needs one.
    pub deadline: Option<&'static str>,
    /// Local admission explanation, when the provider needs one.
    pub quota: Option<&'static str>,
}
/// Credential command identity, attached to its primary adapter.
#[derive(Debug)]
pub struct KeyDecl {
    /// Shared publisher contract for the key help source identity.
    pub spec: &'static SourceSpec,
    /// Transport credential identity.
    pub name: SecretName,
    /// Command spelling retained across credential commands.
    pub word: &'static str,
    /// Source guidance appended to the key registration command.
    pub suffix: &'static str,
    /// Stable discovery order.
    pub order: usize,
}
/// Complete source policy consumed by every source discovery surface.
#[derive(Debug)]
pub struct SourceDecl {
    /// Publisher and transport contract.
    pub spec: &'static SourceSpec,
    /// Publisher's display name.
    pub display: &'static str,
    /// Direct request discovery order; components have no direct request.
    pub registry_order: Option<usize>,
    /// Explicit bounded health behavior.
    pub probe: Probe,
    /// Local catalogs exposed by sync.
    pub bundles: &'static [&'static BundleDecl],
    /// Independently labeled carried fields.
    pub components: &'static [Component],
    /// Other APIs sharing this source identity.
    pub auxiliary: &'static [&'static SourceSpec],
    /// Provider-reviewed timing and admission text.
    pub guidance: Guidance,
    /// Credential command identity, declared once for a shared credential.
    pub key: Option<&'static KeyDecl>,
}
/// Resolve every source without a fallback.
pub const fn decl(source: Source) -> &'static SourceDecl {
    match source {
        Source::Cellosaurus => &crate::cellosaurus::decl::DECL,
        Source::Hgnc => &crate::hgnc::decl::DECL,
        Source::MyGene => &crate::mygene::decl::DECL,
        Source::Reactome => &crate::reactome::decl::DECL,
        Source::WikiPathways => &crate::wikipathways::decl::DECL,
        Source::Hpa => &crate::hpa::decl::DECL,
        Source::UniProt => &crate::uniprot::decl::DECL,
        Source::InterPro => &crate::interpro::decl::DECL,
        Source::ComplexPortal => &crate::complex_portal::decl::DECL,
        Source::Gnomad => &crate::gnomad::decl::DECL,
        Source::ClinVar => &crate::clinvar::decl::DECL,
        Source::Erepo => &crate::erepo::decl::DECL,
        Source::Gwas => &crate::gwas::decl::DECL,
        Source::ClinGen => &crate::clingen::decl::DECL,
        Source::Gencc => &crate::gencc::decl::DECL,
        Source::PubMed => &crate::pubmed::decl::DECL,
        Source::EuropePmc => &crate::europepmc::decl::DECL,
        Source::LitVar => &crate::litvar::decl::DECL,
        Source::LitSense => &crate::litsense::decl::DECL,
        Source::SemanticScholar => &crate::semantic_scholar::decl::DECL,
        Source::Orcid => &crate::orcid::decl::DECL,
        Source::PmcOa => &crate::pmc_oa::decl::DECL,
        Source::Icite => &crate::icite::decl::DECL,
        Source::PubTator => &crate::pubtator::decl::DECL,
        Source::ClinicalTrialsGov => &crate::ctgov::decl::DECL,
        Source::Nci => &crate::nci::decl::DECL,
        Source::Civic => &crate::civic::decl::DECL,
        Source::CancerHotspots => &crate::hotspots::decl::DECL,
        Source::AlphaGenome => &crate::alphagenome::decl::DECL,
        Source::Oncokb => &crate::oncokb::decl::DECL,
        Source::MyVariant => &crate::myvariant::decl::DECL,
        Source::AlleleRegistry => &crate::allele_registry::decl::DECL,
        Source::Mutalyzer => &crate::mutalyzer::decl::DECL,
        Source::VariantValidator => &crate::variantvalidator::decl::DECL,
        Source::Mychem => &crate::mychem::decl::DECL,
        Source::Chembl => &crate::chembl::decl::DECL,
        Source::Opentargets => &crate::opentargets::decl::DECL,
        Source::OpenfdaLabel => &crate::openfda_label::decl::DECL,
        Source::FdaDrugRecall => &crate::fda_drug_recall::decl::DECL,
        Source::Drugsfda => &crate::drugsfda::decl::DECL,
        Source::Ema => &crate::ema::decl::DECL,
        Source::WhoPq => &crate::who_pq::decl::DECL,
        Source::Ddinter => &crate::ddinter::decl::DECL,
        Source::DrugbankOpen => &crate::mychem::decl::DRUGBANKOPEN,
        Source::Drugcentral => &crate::mychem::decl::DRUGCENTRAL,
        Source::Gtopdb => &crate::mychem::decl::GTOPDB,
        Source::Unii => &crate::mychem::decl::UNII,
        Source::Ndc => &crate::mychem::decl::NDC,
        Source::OpenfdaXref => &crate::mychem::decl::OPENFDAXREF,
        Source::Ols => &crate::ols::decl::DECL,
        Source::MyDisease => &crate::mydisease::decl::DECL,
        Source::Monarch => &crate::monarch::decl::DECL,
        Source::Seer => &crate::seer::decl::DECL,
        Source::Mondo => &crate::ols::decl::MONDO,
        Source::Doid => &crate::ols::decl::DOID,
        Source::Ncit => &crate::ols::decl::NCIT,
        Source::Hpo => &crate::hpo::decl::DECL,
        Source::Orphanet => &crate::mydisease::decl::ORPHANET,
        Source::Omim => &crate::mydisease::decl::OMIM,
        Source::Faers => &crate::faers::decl::DECL,
        Source::Vaers => &crate::vaers::decl::DECL,
        Source::Maude => &crate::maude::decl::DECL,
        Source::Meddra => &crate::faers::decl::MEDDRA,
        Source::Gmdn => &crate::maude::decl::GMDN,
        Source::Gtr => &crate::gtr::decl::DECL,
        Source::WhoIvd => &crate::who_ivd::decl::DECL,
        Source::FdaCdx => &crate::fda_cdx::decl::DECL,
        Source::FdaPma => &crate::fda_pma::decl::DECL,
        Source::FdaDeviceRecall => &crate::fda_device_recall::decl::DECL,
        Source::Fda510k => &crate::fda_510k::decl::DECL,
        Source::Cpic => &crate::cpic::decl::DECL,
        Source::ClinPgx => &crate::clinpgx::decl::DECL,
        Source::Cbioportal => &crate::cbioportal::decl::DECL,
    }
}
/// Iterate direct sources in the established public order.
pub fn sources() -> impl Iterator<Item = &'static SourceSpec> {
    let mut declarations: Vec<_> = Source::ALL
        .iter()
        .map(|&source| decl(source))
        .filter(|declaration| declaration.registry_order.is_some())
        .collect();
    declarations.sort_by_key(|declaration| declaration.registry_order);
    declarations.into_iter().map(|declaration| declaration.spec)
}
/// Derive supported keys from declared credential policies in public order.
pub fn keys() -> Vec<SecretName> {
    let mut keys: Vec<_> = Source::ALL.iter().filter_map(|&source| {
        let declaration = decl(source);
        declaration.key.as_ref().filter(|key| matches!(declaration.spec.key,
            KeyUse::Optional(credential) | KeyUse::Required(credential) if credential.secret == key.name))
    }).collect();
    keys.sort_by_key(|key| key.order);
    keys.into_iter().map(|key| key.name).collect()
}
/// Resolve the credential's adapter command name.
pub fn key_name(name: SecretName) -> &'static str {
    key_decl(name).word
}
/// Source guidance attached to a key registration command.
pub fn key_suffix(name: SecretName) -> &'static str {
    key_decl(name).suffix
}
/// Resolve the source represented in credential help.
pub fn key_source(name: SecretName) -> Source {
    key_decl(name).spec.source
}
fn key_decl(name: SecretName) -> &'static KeyDecl {
    match name {
        SecretName::SemanticScholarApiKey => &crate::semantic_scholar::decl::KEY,
        SecretName::AlphaGenomeApiKey => &crate::alphagenome::decl::KEY,
        SecretName::OncokbApiKey => &crate::oncokb::decl::KEY,
        SecretName::NcbiApiKey => &crate::pubmed::decl::KEY,
        SecretName::OrcidAccessToken => &crate::orcid::decl::KEY,
        SecretName::OpenfdaApiKey => &crate::openfda_label::decl::KEY,
        SecretName::NciApiKey => &crate::nci::decl::KEY,
    }
}
/// Describe carried publisher terms without registering another request endpoint.
pub(crate) const fn component_spec(
    source: Source,
    carrier: &'static SourceSpec,
    terms: Terms,
) -> SourceSpec {
    SourceSpec {
        source,
        terms,
        base: carrier.base,
        destinations: biomcp_fetch::Destinations::Primary,
        user_agent: carrier.user_agent,
        key: KeyUse::None,
        quota: carrier.quota,
        deadline: carrier.deadline,
    }
}
