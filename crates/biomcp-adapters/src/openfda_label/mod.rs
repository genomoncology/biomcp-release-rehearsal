//! Identity-checked openFDA name discovery without clinical text selection.
use biomcp_fetch::Input;
use biomcp_fetch::{SourceSpec, Terms};
use biomcp_types::Source;
use std::time::Duration;
/// Publisher policy shared by health, request admission and response outcomes.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::OpenfdaLabel,
    base: "https://api.fda.gov/drug/label.json",
    terms: Terms {
        name: crate::drug_sources::FDA_TERMS.name,
        url: crate::drug_sources::FDA_TERMS.url,
        usage: crate::drug_sources::FDA_TERMS.usage,
    },
    quota: &crate::quotas::OPENFDA,
    key: crate::openfda_key::OPTIONAL,
    deadline: Duration::from_secs(15),
};
mod admit;
mod map;
mod page;
mod raw;
mod wire;
use biodata::{DrugId, Text};
use biomcp_fetch::{Arg, BadReply, Endpoint, Freshness, Leg, ReplyMetadata, RequestError};
/// Exact native headings supported by the label surface.
pub use biomcp_types::OPENFDA_LABEL_HEADINGS as HEADINGS;
pub use map::Record;
use raw::Record as RawRecord;
const LABEL: Endpoint = SPEC.get("").fresh(Freshness::Days(1)).json_absence();
/// A source candidate page, preserving native order and source total.
#[derive(Debug)]
pub struct Page {
    /// Source revisions and native positions before name filtering.
    pub window: crate::NativeWindow<Record>,
    /// Rejected native source fields.
    pub problems: Vec<biomcp_types::Problem>,
    /// This reply's actual original retrieval metadata.
    pub metadata: ReplyMetadata,
}
fn literal(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
mod search;
pub use search::{Search, search, search_names};
fn normalize_name(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}
impl RawRecord {
    /// Compare complete publisher values after the established case and spacing normalization.
    pub(super) fn name_match(&self, input: &Text) -> Option<biomcp_types::DrugNameMatch> {
        let input = normalize_name(input.as_str());
        let matches = |names: &Option<Vec<crate::native_text::NativeText>>| {
            names
                .iter()
                .flatten()
                .any(|name| normalize_name(name.as_str()) == input)
        };
        use biomcp_types::DrugNameMatch;
        match (
            matches(&self.openfda.brand_name),
            matches(&self.openfda.generic_name),
        ) {
            (true, true) => Some(DrugNameMatch::Both),
            (true, false) => Some(DrugNameMatch::Brand),
            (false, true) => Some(DrugNameMatch::Generic),
            (false, false) => None,
        }
    }
}
/// Read only the explicitly selected SPL set, refusing a mismatched returned set before caching.
pub fn get(leg: &Leg<'_>, set: &DrugId) -> Result<Page, RequestError> {
    if !biomcp_types::DrugSource::OpenfdaLabel.accepts(set) {
        return Err(biomcp_fetch::bad_reply(Source::OpenfdaLabel, Some(200)));
    }
    let found = page(
        leg,
        &format!("openfda.spl_set_id:{}", literal(&set.to_string())),
        1,
        0,
        Some(set),
    )?;
    if !found.window.rows.is_empty() {
        return Ok(found);
    }
    page(
        leg,
        &format!("set_id:{}", literal(&set.to_string())),
        1,
        0,
        Some(set),
    )
}
fn page(
    leg: &Leg<'_>,
    selector: &str,
    limit: u32,
    offset: u32,
    set: Option<&DrugId>,
) -> Result<Page, RequestError> {
    let mut args = vec![
        ("search", Arg::Text(selector)),
        ("limit", Arg::Num(limit.into())),
        ("sort", Arg::Text("effective_time:desc")),
    ];
    if offset > 0 {
        args.push(("skip", Arg::Num(offset.into())));
    }
    let (reply, metadata) = leg
        .call(
            &LABEL,
            Input::Params(&args),
            biomcp_fetch::Read::<raw::Response>::json().check(&|reply| match reply {
                raw::Response::Page { meta, results }
                    if meta.results.skip != offset
                        || meta.results.limit != limit
                        || set.is_some_and(|set| results.iter().any(|r| r.set_id != *set)) =>
                {
                    Err(BadReply)
                }
                _ => Ok(()),
            }),
        )
        .map(|received| (received.value, received.meta))?;
    let (records, total) = match reply {
        raw::Response::Page { meta, results } => (results, meta.results.total),
        raw::Response::Absent { .. } => (Vec::new(), 0),
    };
    let problems = records
        .iter()
        .flat_map(|r| r.problems.iter().cloned())
        .collect();
    let received = records.len() as u32;
    Ok(Page {
        window: crate::NativeWindow::new(
            records.into_iter().map(map::Record).collect(),
            offset,
            limit,
            Some(total),
            received,
            crate::Refusal::None,
        ),
        problems,
        metadata,
    })
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
