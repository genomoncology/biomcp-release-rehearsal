//! One owner for escaped FAERS query generation and its local request bound.
use super::AdverseEventSerious;
/// BioMCP request budget for escaped FAERS expressions, including filters.
/// Bounds repeated names before transport. The publisher query-parameters reference
/// (https://open.fda.gov/apis/query-parameters/) does not specify a search-length cap.
pub(super) const FAERS_QUERY_MAX_BYTES: usize = 4096;
fn literal(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
/// Exact literal source expression with analyzed reaction phrase-match semantics.
impl super::AdverseEventSearch {
    /// Literal native FAERS query, checked against FAERS_QUERY_MAX_BYTES at construction.
    pub fn faers_expression(&self) -> String {
        let query = self;
        let names = query
            .text()
            .into_iter()
            .chain(query.drug().iter())
            .map(|name| name_expression(name.as_str()))
            .collect::<Vec<_>>();
        let mut selector = match names.as_slice() {
            [single] => single.clone(),
            _ => format!("({})", names.join(" OR ")),
        };
        if let Some(reaction) = query.reaction() {
            selector.push_str(&format!(
                " AND patient.reaction.reactionmeddrapt:{}",
                literal(reaction.as_str())
            ));
        }
        if let Some(flag) = query.serious() {
            let field = match flag {
                AdverseEventSerious::Any => "serious",
                AdverseEventSerious::Death => "seriousnessdeath",
                AdverseEventSerious::Hospitalization => "seriousnesshospitalization",
                AdverseEventSerious::Lifethreatening => "seriousnesslifethreatening",
                AdverseEventSerious::Disabling => "seriousnessdisabling",
                AdverseEventSerious::CongenitalAnomaly => "seriousnesscongenitalanomali",
                AdverseEventSerious::Other => "seriousnessother",
            };
            selector.push_str(&format!(" AND {field}:1"));
        }
        if query.received_from().is_some() || query.received_to().is_some() {
            let from = query
                .received_from()
                .map_or_else(|| "*".into(), |v| v.to_string().replace('-', ""));
            let to = query
                .received_to()
                .map_or_else(|| "*".into(), |v| v.to_string().replace('-', ""));
            selector.push_str(&format!(" AND receivedate:[{from} TO {to}]"));
        }
        selector
    }
}

fn name_expression(name: &str) -> String {
    let text = literal(name);
    format!(
        "(patient.drug.openfda.generic_name:{text} OR patient.drug.openfda.brand_name:{text} OR patient.drug.medicinalproduct:{text})"
    )
}
