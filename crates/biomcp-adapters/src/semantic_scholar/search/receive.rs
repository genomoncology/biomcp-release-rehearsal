use super::{RawSearch, Search};
use crate::mapped::Problems;

impl From<RawSearch> for Search {
    fn from(raw: RawSearch) -> Self {
        let mut refusals = crate::Refusal::None;
        let received = raw.data.len();
        let mut p = Problems {
            source: biomcp_types::Source::SemanticScholar,
            record: None,
            list: Vec::new(),
        };
        let data = raw
            .data
            .into_iter()
            .filter_map(|v| {
                let row = p.record("paper_id", v, &mut refusals)?;
                refusals.include(row.refusals);
                Some(row)
            })
            .collect();
        Self {
            refusals,
            total: raw.total,
            data,
            problems: p.list,
            received,
        }
    }
}
