//! Source-selected drug contracts; adapters and command dispatch live elsewhere.
mod recall;
pub use recall::{
    DrugRecallCard, DrugRecallClassification, DrugRecallCountUnit, DrugRecallHeader, DrugRecallPage,
};
mod civic;
mod fda;
pub use civic::{
    DrugCivicClaim, DrugCivicMember, DrugCivicPage, DrugCivicProfile, DrugCivicStreams,
    DrugOncokbLookup, DrugOncokbMissing,
};
pub use fda::{
    DrugFdaCandidate, DrugFdaDocument, DrugFdaIngredient, DrugFdaPage, DrugFdaProduct,
    DrugFdaProperty, DrugFdaSubmission,
};
mod query;
mod response;
mod source;
pub use query::{DrugGet, DrugSearch, DrugSection};
pub use response::{
    DrugApprovals, DrugCard, DrugCivic, DrugHeader, DrugIndications, DrugInteractions, DrugLabel,
    DrugLabelFragment, DrugLabelText, DrugNameResolution, DrugPage, DrugRecords, DrugRegulatory,
    DrugTargets, DrugXrefs,
};

pub use source::DrugSource;

mod name_selector;
pub use name_selector::DrugNameSelector;

mod who;
pub use who::{DrugWhoCard, DrugWhoGet, DrugWhoPage, DrugWhoRow};
#[cfg(feature = "schema")]
mod command;
#[cfg(feature = "schema")]
pub use command::DrugGetCommand;

mod name_match;
pub use name_match::DrugNameMatch;

mod label_headings;
pub use label_headings::OPENFDA_LABEL_HEADINGS;

/// Default submission page size for a native Drugs@FDA application.
pub const DEFAULT_DRUG_APPROVALS_LIMIT: u32 = 10;
