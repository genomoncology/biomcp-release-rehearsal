use super::{record::Rows, run};
use crate::Outcome;
use biomcp_types::ReplyBody;
use biomcp_types::{
    PageInfo, PhenotypeGet, PhenotypeHierarchy, PhenotypeSection, PhenotypeSynonyms,
    PhenotypeXrefs, ReplyData, Request, StepKind,
};
pub(super) fn empty(q: &PhenotypeGet) -> ReplyData {
    match q.section() {
        Some(PhenotypeSection::Synonyms) => {
            biomcp_types::PhenotypeSynonyms::into_reply(q.clone(), None)
        }
        Some(PhenotypeSection::Xrefs) => biomcp_types::PhenotypeXrefs::into_reply(q.clone(), None),
        Some(PhenotypeSection::Hierarchy) => {
            biomcp_types::PhenotypeHierarchy::into_reply(q.clone(), None)
        }
        _ => biomcp_types::PhenotypeTermCard::into_reply(q.clone(), None),
    }
}
pub(super) fn apply(out: &mut Outcome, echo: &PhenotypeGet, q: &PhenotypeGet, rows: Rows) {
    let components = rows.0.components.clone();
    let fetched = rows.4;
    match q.section() {
        None => {
            for section in &rows.0.sections {
                if let Some(request) = crate::next::checked(
                    q.with_section(*section),
                    super::record::source(q),
                    &mut out.problems,
                ) {
                    out.next.push(crate::next::Step::new(
                        StepKind::Section,
                        &[super::record::source(q)],
                        run(Request::GetPhenotype(request)),
                    ));
                }
            }
            out.body = (rows.0).with_query(echo.clone());
        }
        Some(PhenotypeSection::Synonyms) => {
            if let Some(rows) = rows.1 {
                let items = page(out, q, &rows);
                out.body = (PhenotypeSynonyms {
                    items,
                    components,
                    fetched,
                })
                .with_query(echo.clone());
            } else {
                unavailable(out);
            }
        }
        Some(PhenotypeSection::Xrefs) => {
            if let Some(rows) = rows.2 {
                let items = page(out, q, &rows);
                out.body = (PhenotypeXrefs {
                    items,
                    components,
                    fetched,
                })
                .with_query(echo.clone());
            } else {
                unavailable(out);
            }
        }
        Some(PhenotypeSection::Hierarchy) => {
            if let Some(rows) = rows.3 {
                let items = page(out, q, &rows);
                out.body = (PhenotypeHierarchy {
                    items,
                    components,
                    fetched,
                })
                .with_query(echo.clone());
            } else {
                unavailable(out);
            }
        }
        _ => unavailable(out),
    }
}
fn unavailable(out: &mut Outcome) {
    crate::status::item(out, false, false, false);
}
fn page<T: Clone>(out: &mut Outcome, q: &PhenotypeGet, rows: &[T]) -> Box<[T]> {
    let offset = q.section_offset() as usize;
    let items: Box<[T]> = rows
        .iter()
        .skip(offset)
        .take(q.section_limit() as usize)
        .cloned()
        .collect();
    let returned = items.len() as u32;
    let next = q.section_offset().saturating_add(returned);
    out.page = Some(PageInfo {
        offset: q.section_offset(),
        returned,
        total: rows.len() as u64,
        pool: None,
        truncated: None,
    });
    if returned > 0 && (next as usize) < rows.len() {
        out.next.push(crate::next::Step::new(
            StepKind::Page,
            &[super::record::source(q)],
            run(Request::GetPhenotype(q.at_section_offset(next))),
        ));
    }
    {
        let absent = rows.is_empty();
        crate::status::item(out, !absent, false, false);
    }
    items
}
