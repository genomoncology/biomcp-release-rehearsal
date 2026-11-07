//! Report source wording per retained registry row, without assessing eligibility.
use biomcp_adapters::ctgov::{GeneContext, GeneMention};
use biomcp_types::{SourceNotice, SourceOutcome};

pub(super) fn append(gene: &str, contexts: &[GeneContext], source: &mut SourceOutcome) {
    for context in contexts {
        let id = context.nct_id;
        let message = match &context.mention {
            GeneMention::ExclusionOnly { context } => format!(
                "Trial {id}: {gene} appears only under the registered exclusion-criteria heading \
                 in the returned protocol fields: \"{context}\". This is not an eligibility assessment."
            ),
            GeneMention::NegativeContext { field, context } => format!(
                "Trial {id}: the registered {field} contains \"{context}\". \
                 Gene-word matching does not establish eligibility."
            ),
            GeneMention::NotMentioned => format!(
                "Trial {id}: {gene} is not mentioned as an exact word in the returned protocol fields. \
                 Registry text matching does not establish a gene link or eligibility."
            ),
        };
        source.notices.push(SourceNotice::RequestGuidance {
            data_date: None,
            message: message.into(),
            retry_after_seconds: None,
        });
    }
}
