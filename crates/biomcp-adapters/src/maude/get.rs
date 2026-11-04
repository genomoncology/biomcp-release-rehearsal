use super::{SPEC, raw};
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, RequestError};
use biomcp_fetch::{Input, Read};
use biomcp_types::{AdverseEventGet, AdverseEventSection};
/// Exact numeric native key; never probe another endpoint or source family.
pub fn get(
    leg: &Leg<'_>,
    query: &AdverseEventGet,
) -> Result<Option<crate::Mapped<super::Report>>, RequestError> {
    if query.section() == Some(AdverseEventSection::Devices) {
        crate::adverse_event_sources::components::gmdn(leg)?;
    }
    let selector = format!("mdr_report_key:\"{}\"", query.id().as_str());
    let endpoint = SPEC.get("").fresh(Freshness::Days(1)).json_absence();
    let (reply, _) = leg
        .call(
            &endpoint,
            Input::Params(&[("search", Arg::Text(&selector)), ("limit", Arg::Num(1))]),
            Read::<raw::Response>::json().check(&|reply| {
                if let raw::Response::Page { meta, results } = reply
                    && (meta.results.total != 1
                        || meta.results.skip != 0
                        || meta.results.limit != 1
                        || results.iter().any(|r| r.mdr_report_key != *query.id()))
                {
                    return Err(BadReply);
                }
                Ok(())
            }),
        )
        .map(|received| (received.value, received.meta))?;
    Ok(match reply {
        raw::Response::Page { results, .. } => results.into_iter().next().map(super::map::report),
        raw::Response::Absent { .. } => None,
    })
}
