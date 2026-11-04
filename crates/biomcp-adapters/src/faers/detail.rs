//! Checked exact report get before ordinary cache admission.
use super::{SPEC, detail_raw as raw};
use biodata::{FaersReport, ReportId};
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, ReplyMetadata, RequestError};
use biomcp_fetch::{Input, Read};
/// A single checked source report, absent only for the native no-match reply.
#[derive(Debug)]
pub struct Detail {
    /// Actual refused native records, independent of optional fields.
    pub refusals: crate::Refusal,
    /// Rejected optional source fields.
    pub problems: Vec<biomcp_types::Problem>,
    /// Exact selected native record.
    pub report: Option<FaersReport>,
    /// Native positions for independently rejected report rows.
    pub positions: super::Positions,
    /// Credential-free URL used for this successful request.
    pub report_url: biodata::Text,
    /// Metadata belongs to the original source reply.
    pub metadata: ReplyMetadata,
}
/// Fetch one ID without choosing a version or a result among multiple records.
pub fn get(leg: &Leg<'_>, id: &ReportId) -> Result<Detail, RequestError> {
    let expression = format!("safetyreportid:\"{id}\"");
    let endpoint = SPEC.get("").fresh(Freshness::Days(1)).json_absence();
    let (reply, metadata) = leg
        .call(
            &endpoint,
            Input::Params(&[
                ("search", Arg::Text(&expression)),
                ("limit", Arg::Num(5)),
                ("skip", Arg::Num(0)),
                ("sort", Arg::Text("receivedate:desc")),
            ]),
            Read::<raw::Response>::json().check(&|response| {
                if let raw::Response::Page { results, .. } = response
                    && results.first().is_none_or(|r| r.safetyreportid != *id)
                {
                    return Err(BadReply);
                }
                Ok(())
            }),
        )
        .map(|received| (received.value, received.meta))?;
    let report = match reply {
        raw::Response::Page { results, .. } => {
            results.into_iter().next().map(super::detail_map::report)
        }
        raw::Response::Absent { .. } => None,
    };
    let url = crate::registry::records::record_url(
        biomcp_types::Source::Faers,
        crate::registry::records::RecordKind::AdverseEvent,
        id.as_str(),
    )
    .ok_or(biomcp_fetch::InputError::Serialization)?;
    let report_url =
        biodata::Text::new(&url).map_err(|_| biomcp_fetch::InputError::Serialization)?;
    let (report, positions, problems, refusals) = match report {
        Some(mapped) => (
            Some(mapped.value.data),
            mapped.value.positions,
            mapped.problems,
            mapped.refusals,
        ),
        None => (
            None,
            super::Positions::default(),
            Vec::new(),
            crate::Refusal::None,
        ),
    };
    Ok(Detail {
        refusals,
        problems,
        report,
        positions,
        report_url,
        metadata,
    })
}
