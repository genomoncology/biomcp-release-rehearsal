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
    let exact = candidates(leg, &selector(".exact"), limit, offset)?;
    if exact.window.total().is_some_and(|total| total > 0) {
        return Ok(exact);
    }
    let fallback = candidates(leg, &selector(""), limit, offset)?;
    // Preserve native matches, totals and offsets, even when a word match is
    // not a complete product name. Select by total so later pages never switch queries.
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
    candidates(leg, &selector, limit, offset)
}

// openFDA reports an exhausted offset as a 404 without a total. Probe the same
// selector at zero before paging, so absence never broadens an existing search.
fn candidates(
    leg: &Leg<'_>,
    selector: &str,
    limit: u32,
    offset: u32,
) -> Result<Page, RequestError> {
    if offset == 0 {
        return page(leg, selector, limit, offset, None);
    }
    let mut first = page(leg, selector, 1, 0, None)?;
    let total = first.window.total().unwrap_or(0);
    if u64::from(offset) >= total {
        first.window = crate::NativeWindow::new(
            Vec::new(),
            offset,
            limit,
            Some(total),
            0,
            crate::Refusal::None,
        );
        return Ok(first);
    }
    page(leg, selector, limit, offset, None)
}
