use super::{Snapshot, advance, continue_records, records, section};
use crate::Outcome;
use biodata::Text;
use biomcp_fetch::RequestError;
use biomcp_types::ArticleInput;
use biomcp_types::{ArticleGet, Request, StepKind, TrialGet, TrialSection, TrialSections};
pub(super) fn render(
    query: &TrialGet,
    snapshot: &Snapshot,
    pool: Option<&Text>,
    sections: &mut TrialSections,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let requested =
        |s| query.sections().contains(&s) || query.sections().contains(&TrialSection::All);
    let d = &snapshot.detail;
    let id = d.card.nct_id;
    if requested(TrialSection::References) {
        let mut page = records(query, &d.references, 8000, serde_json::to_vec)?;
        continue_records(&mut page, query, id, TrialSection::References, pool);
        advance(
            query,
            id,
            TrialSection::References,
            pool,
            page.offset + page.items.len() as u32,
            page.total,
            out,
        );
        let inputs: Vec<_> = page
            .items
            .iter()
            .filter_map(|r| r.pmid)
            .map(ArticleInput::Pmid)
            .collect();
        for chunk in inputs.chunks(20) {
            if let Some(request) = crate::next::checked(
                ArticleGet::new(chunk.into()),
                biomcp_types::Source::ClinicalTrialsGov,
                &mut out.problems,
            ) {
                out.next.push(crate::next::Step::run(
                    StepKind::Section,
                    &[biomcp_types::Source::PubMed],
                    Request::GetArticle(request),
                    false,
                ));
            }
        }
        sections.references = Some(section(page, id));
    }
    if requested(TrialSection::Documents) {
        let mut page = records(query, &d.documents, 2000, serde_json::to_vec)?;
        continue_records(&mut page, query, id, TrialSection::Documents, pool);
        advance(
            query,
            id,
            TrialSection::Documents,
            pool,
            page.offset + page.items.len() as u32,
            page.total,
            out,
        );
        sections.documents = Some(section(page, id));
    }
    // All checked section continuations precede optional discovery actions under the shared next budget.
    out.next
        .sort_by_key(|step| if step.0 == StepKind::Page { 0 } else { 1 });
    Ok(())
}
