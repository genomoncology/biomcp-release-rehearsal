//! Checked source replies mapped into shared human gene values.
/// Restricted AlphaGenome prediction service.
pub mod alphagenome;
mod bundle_mapping;
pub mod hgnc;
mod mapped;
mod native_positions;
pub use native_positions::NativePositions;
pub mod mygene;
/// Keyed OncoKB gene annotations.
pub mod oncokb;
pub mod pubmed;
mod quota_policy;
pub mod quotas;
mod raw;
mod strict_input;
pub use mapped::{Mapped, Refusal};

pub mod complex_portal;
pub mod gnomad;
pub mod interpro;
pub mod uniprot;

/// Direct HPO publisher policy.
pub mod hpo;
pub mod reactome;
pub mod wikipathways;

pub mod civic;

pub mod clingen;
pub mod gencc;

pub mod hpa;

pub mod europepmc;
pub mod pubtator;

pub mod jats;
pub mod pmc_oa;

pub mod icite;

/// Exact variant publication lookup.
pub mod litvar;
/// Exact public ORCID person and whole works records.
pub mod orcid;
/// Semantic Scholar graph metadata under one shared optional-key quota.
pub mod pdf;
pub mod semantic_scholar;
mod xml_element;

pub mod guidance;
pub mod registry;

/// Sentence resemblance in native provider order.
pub mod litsense;
pub mod sheet;

pub mod allele_registry;
/// ClinicalTrials.gov native search.
pub mod ctgov;

pub mod mutalyzer;
pub mod myvariant;
/// Explicit keyed NCI native trials.
pub mod nci;
pub mod variantvalidator;

/// Explicit cohort sample observations.
pub mod cbioportal;

mod assertion;
pub mod clinvar;
pub mod erepo;
pub mod hotspots;

/// GWAS Catalog v2 exact rsID associations.
pub mod gwas;

/// Drug publisher declarations and independently labeled aggregator fields.
pub mod drug_sources;

pub mod chembl;
pub mod ddinter;
pub mod drugsfda;
/// EMA medicines bundle with explicit synchronization.
pub mod ema;
/// MyChem literal native candidate and document identity adapter.
pub mod mychem;
pub mod openfda_label;
pub mod opentargets;

/// Disease publisher policy and pre-request component admission.
pub mod disease_sources;
pub mod monarch;
pub mod mydisease;
pub mod ols;
pub mod seer;

mod text_parts;

mod received;
pub use received::received_date;

/// Adverse event publisher declarations and component admission.
pub mod adverse_event_sources;
/// FAERS publisher declaration.
pub mod faers;
/// MAUDE publisher declaration.
pub mod maude;

/// FDA companion table publisher policy.
pub mod fda_cdx;
/// openFDA PMA publisher policy.
pub mod fda_pma;
/// Diagnostic publisher declarations; command dispatch is not yet implemented.
pub mod gtr;
/// WHO export publisher policy.
pub mod who_ivd;

/// Shared pharmacogenomics publisher policy.
pub mod pgx_sources;

/// ClinPGx publisher policy.
pub mod clinpgx;
/// CPIC publisher policy.
pub mod cpic;

/// Exact native FDA clearance retrieval.
pub mod fda_510k;

/// WHO finished pharmaceutical product prequalification.
pub mod who_pq;

/// Explicit CDC WONDER vaccine report summaries.
pub mod vaers;

/// Literal Cellosaurus discovery and exact cell identity.
pub mod cellosaurus;

mod openfda_key;

/// Native medical device recall records and exact enforcement classification.
pub mod fda_device_recall;
/// Native FDA drug enforcement records.
pub mod fda_drug_recall;

mod native_text;

mod native_window;
pub use native_window::NativeWindow;

mod graphql;
mod openfda;

/// Adapter-owned source and bundle declarations.
pub mod decl;
