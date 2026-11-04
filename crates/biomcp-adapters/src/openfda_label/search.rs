//! Unfielded search recovers labels without harmonized names.
//! Native application numbers remain exact, separate from brand or ingredient identity.
//! Publisher fields: https://open.fda.gov/apis/drug/label/searchable-fields/.
use super::{Page, literal, page};
use biodata::{DrugId, Text};
use biomcp_fetch::{Leg, RequestError};

/// The native discovery result and the recovery step attempted.
#[derive(Debug)]
pub struct Search {
    /// Candidate page or source failure.
    pub page: Result<Page, RequestError>,
    /// Exact unfielded selector when discovery reached the free-text step.
    pub free_text: Option<Text>,
}
/// Discover exact applications or names, then words, then free text.
pub fn search(leg: &Leg<'_>, name: &Text, limit: u32, offset: u32) -> Search {
    let mut free_text = None;
    let page = discover(leg, name, limit, offset, &mut free_text);
    Search { page, free_text }
}
/// Retain native word-search candidates for strict complete-name and approved single-term get matching.
pub fn search_names(
    leg: &Leg<'_>,
    name: &Text,
    limit: u32,
    offset: u32,
) -> Result<Page, RequestError> {
    let name = literal(name.as_str());
    page(
        leg,
        &format!("openfda.brand_name:{name} OR openfda.generic_name:{name}"),
        limit,
        offset,
        None,
    )
}
fn discover(
    leg: &Leg<'_>,
    name: &Text,
    limit: u32,
    offset: u32,
    free_text: &mut Option<Text>,
) -> Result<Page, RequestError> {
    if !(1..=50).contains(&limit) {
        return Err(biomcp_fetch::bad_reply(
            biomcp_types::Source::OpenfdaLabel,
            Some(200),
        ));
    }
    if name
        .as_str()
        .parse::<DrugId>()
        .is_ok_and(|id| biomcp_types::DrugSource::Drugsfda.accepts(&id))
    {
        let mut found = page(
            leg,
            &format!("openfda.application_number:{}", literal(name.as_str())),
            limit,
            offset,
            None,
        )?;
        found.window.rows.retain(|record| {
            record
                .0
                .openfda
                .application_number
                .iter()
                .flatten()
                .any(|number| number.as_str() == name.as_str())
        });
        return Ok(found);
    }
    let selector = |suffix: &str| {
        let name = literal(name.as_str());
        format!("openfda.brand_name{suffix}:{name} OR openfda.generic_name{suffix}:{name}")
    };
    let mut exact = page(leg, &selector(".exact"), limit, offset, None)?;
    exact
        .window
        .rows
        .retain(|record| record.name_match(name).is_some());
    if !exact.window.rows.is_empty() {
        return Ok(exact);
    }
    let mut fallback = page(leg, &selector(""), limit, offset, None)?;
    fallback
        .window
        .rows
        .retain(|record| record.name_match(name).is_some());
    // Keep continuation within the word selector when it has native candidates,
    // including pages whose candidates all fail the complete-name check.
    if fallback.window.total().is_some_and(|total| total > 0) {
        return Ok(fallback);
    }
    let selector = literal(name.as_str());
    *free_text = Some(
        selector
            .parse()
            .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::OpenfdaLabel, Some(200)))?,
    );
    // The publisher searched label text, so a matching passage need not name
    // this product. Preserve its candidates and mark exact name matches separately.
    page(leg, &selector, limit, offset, None)
}
