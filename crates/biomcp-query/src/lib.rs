//! Resolve checked human gene inputs through authoritative source adapters.
pub mod article;
mod gene;
pub mod store;

mod context;
mod next;
mod run;
mod status;
pub use context::{Ctx, DRUG_NAME_LIMIT};
pub use run::{Action, Bundle, run};
pub use status::{Outcome, sync_status};

mod fan_out;

pub mod health;

pub mod trial;
mod variant;

mod disease;
/// Single-source drug discovery and exact native cards.
mod drug;

/// Source-owned adverse event search.
mod adverse_event;

/// Source-owned diagnostic laboratory tests and relations.
mod diagnostic;
/// Explicit CPIC gene and native context retrieval.
mod pgx;

/// Explicit native device report cards and sections.
mod adverse_event_maude;

mod biomarker;
mod nci_search;
mod organization;
/// Native human pathway discovery and exact cards.
mod pathway;
/// Literal phenotype discovery and exact independent source cards.
mod phenotype;
mod publish;
mod source_snapshot;
/// Stored study catalog discovery.
mod study;

/// Exact reviewed-human protein discovery and cards.
mod protein;

/// Checked native GWAS association search.
mod gwas;

/// Provider-local author discovery and exact cards.
mod author;
/// Literal Cellosaurus discovery and exact cell identity.
mod cell_line;

mod prepare;

/// Source facts used by command interfaces.
pub mod sources;
