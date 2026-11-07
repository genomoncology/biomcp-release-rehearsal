//! Source-specific adverse event requests and bounded report responses.
mod faers;
mod query;
mod receipt;
mod response;
mod selectors;
pub use query::{AdverseEventGet, AdverseEventSearch};
pub use response::{
    AdverseEventDevice, AdverseEventDevices, AdverseEventDrug, AdverseEventDrugs,
    AdverseEventFaersCard, AdverseEventHeader, AdverseEventIdentifierRecords,
    AdverseEventIdentifiers, AdverseEventIdentityCounts, AdverseEventMaudeCard,
    AdverseEventNarrative, AdverseEventNarrativeItem, AdverseEventPage, AdverseEventPatient,
    AdverseEventReaction, AdverseEventReactionCount, AdverseEventReactionCounts,
    AdverseEventReactionMatch, AdverseEventReactions, AdverseEventRecords, AdverseEventReporter,
};
pub use selectors::{
    AdverseEventAggregate, AdverseEventIdentifierSection, AdverseEventSection, AdverseEventSerious,
    AdverseEventSource, ReportUnit,
};

mod vaers;
pub use vaers::{
    VAERS_INFLUENZA, VaersCell, VaersNotice, VaersRow, VaersScope, VaersSection, VaersSummary,
    vaers_vaccine_code,
};

#[cfg(feature = "schema")]
mod command;
#[cfg(feature = "schema")]
pub use command::AdverseEventSearchCommand;
