//! Whole posted result rows share the existing immutable section paging.
use super::{Snapshot, advance, continue_records, records, section};
use crate::Outcome;
use biodata::Text;
use biomcp_fetch::RequestError;
use biomcp_types::{TrialGet, TrialSection, TrialSections};

pub(super) fn render(
    query: &TrialGet,
    snapshot: &Snapshot,
    pool: Option<&Text>,
    sections: &mut TrialSections,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    if query.sections().contains(&TrialSection::Results)
        || query.sections().contains(&TrialSection::All)
    {
        let id = snapshot.detail.card.nct_id;
        let mut page = records(query, &snapshot.detail.results, 8000, serde_json::to_vec)?;
        continue_records(&mut page, query, id, TrialSection::Results, pool);
        advance(
            query,
            id,
            TrialSection::Results,
            pool,
            page.offset + page.items.len() as u32,
            page.total,
            out,
        );
        sections.results = Some(section(page, id));
    }
    Ok(())
}
