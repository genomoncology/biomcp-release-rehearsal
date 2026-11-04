use super::{SPEC, raw};
use biomcp_fetch::Input;
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, RequestError};
use biomcp_types::{AdverseEventHeader, AdverseEventSearch, AdverseEventSource, Source};
fn phrase(field: &str, text: &biodata::Text) -> String {
    format!(
        "{field}:\"{}\"",
        text.as_str().replace('\\', "\\\\").replace('"', "\\\"")
    )
}
/// Source-native phrases and exact product-code OR groups joined with AND.
pub fn expression(query: &AdverseEventSearch) -> String {
    let mut selectors = Vec::new();
    if let Some(text) = query.text() {
        selectors.push(phrase("device.generic_name", text));
    }
    if let Some(text) = query.manufacturer() {
        selectors.push(phrase("device.manufacturer_d_name", text));
    }
    let codes: Vec<_> = query
        .product_code()
        .iter()
        .map(|text| phrase("device.device_report_product_code.exact", text))
        .collect();
    match codes.len() {
        0 => (),
        1 => selectors.extend(codes),
        _ => selectors.push(format!("({})", codes.join(" OR "))),
    }
    if let Some(text) = query.problem() {
        selectors.push(phrase("product_problems", text));
    }
    if query.received_from().is_some() || query.received_to().is_some() {
        let from = query
            .received_from()
            .map_or_else(|| "*".into(), |date| date.to_string().replace('-', ""));
        let to = query
            .received_to()
            .map_or_else(|| "*".into(), |date| date.to_string().replace('-', ""));
        selectors.push(format!("date_received:[{from} TO {to}]"));
    }
    selectors.join(" AND ")
}
/// Read one source-ordered report page after component admission.
pub fn search(
    leg: &Leg<'_>,
    query: &AdverseEventSearch,
) -> Result<crate::faers::Page, RequestError> {
    crate::adverse_event_sources::components::gmdn(leg)?;
    let selector = expression(query);
    let endpoint = SPEC.get("").fresh(Freshness::Days(1)).json_absence();
    let (reply, metadata) = leg
        .call(
            &endpoint,
            Input::Params(&[
                ("search", Arg::Text(&selector)),
                ("limit", Arg::Num(query.limit().into())),
                ("skip", Arg::Num(query.offset().into())),
                ("sort", Arg::Text("date_received:desc")),
            ]),
            biomcp_fetch::Read::<raw::Response>::json().check(&|reply| {
                if let raw::Response::Page { meta, .. } = reply
                    && (meta.results.skip != query.offset() || meta.results.limit != query.limit())
                {
                    return Err(BadReply);
                }
                Ok(())
            }),
        )
        .map(|received| (received.value, received.meta))?;
    let mut problems = crate::mapped::Problems {
        source: Source::Maude,
        record: None,
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let (items, total, updated) = match reply {
        raw::Response::Page { meta, results } => (
            results
                .into_iter()
                .map(|r| {
                    let mapped = super::map::report(r);
                    refusals.include(mapped.refusals);
                    problems.list.extend(mapped.problems);
                    header(&mapped.value.data)
                })
                .collect(),
            meta.results.total,
            meta.last_updated,
        ),
        raw::Response::Absent { .. } => (Vec::new(), 0, None),
    };
    Ok(crate::faers::Page {
        refusals,
        problems: problems.list,
        components: crate::adverse_event_sources::components::labels(Source::Maude, Source::Gmdn)?,
        items,
        total,
        updated,
        metadata,
    })
}
pub(super) fn header(row: &biodata::MaudeReport) -> AdverseEventHeader {
    AdverseEventHeader {
        id: row.mdr_report_key.clone(),
        source: AdverseEventSource::Maude,
        version: None,
        serious: None,
        event_type: row.event_type.clone(),
        first_received: row.date_received.clone(),
    }
}
