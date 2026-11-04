//! Admit optional registry processing metadata under its documented UTC convention.
use super::{VERSION, raw::Version};
use biomcp_fetch::{Input, Read};
use biomcp_fetch::{Leg, RequestError};

/// Source processing timestamp, distinct from request time and record update dates.
pub fn version(
    leg: &Leg<'_>,
) -> Result<crate::Mapped<Option<biomcp_types::SourceTimestamp>>, RequestError> {
    let raw = leg
        .call(&VERSION, Input::Params(&[]), Read::<Version>::json())?
        .value;
    let mut problems = crate::mapped::Problems {
        source: biomcp_types::Source::ClinicalTrialsGov,
        record: None,
        list: Vec::new(),
    };
    let value = problems
        .field("dataTimestamp", raw.data_timestamp)
        .and_then(|native| {
            // The official /version OpenAPI description declares this unsuffixed
            // yyyy-MM-dd'T'HH:mm:ss field UTC: https://clinicaltrials.gov/api/oas/v2.
            let checked = if native.len() == 19 {
                format!("{native}Z").parse()
            } else {
                native.parse()
            };
            problems.optional("dataTimestamp", Some(checked))
        });
    Ok(crate::Mapped {
        refusals: crate::Refusal::None,
        value,
        problems: problems.list,
    })
}
