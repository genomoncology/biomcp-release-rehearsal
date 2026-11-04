//! Each requested collection retains whole source rows and its independent total.
use super::{Outcome, run};
use biomcp_types::{
    ArticleGet, ArticleInput, Capped, CellLineCard, CellLineGet, CellLineSection, CellLineVariants,
    GeneGet, GeneInput, PageInfo, Request, Section, StepKind,
};
fn page<T>(rows: &mut Capped<T>, query: &CellLineGet, default: u32) {
    rows.items = std::mem::take(&mut rows.items)
        .into_vec()
        .into_iter()
        .skip(query.section_offset() as usize)
        .take(query.section_limit().unwrap_or(default) as usize)
        .collect();
}
pub(super) fn select(query: &CellLineGet, card: &mut CellLineCard) {
    let section = query.section();
    let all = section == Some(CellLineSection::All);
    if !all && section != Some(CellLineSection::Names) {
        card.sections.names = None;
    }
    if !all && section != Some(CellLineSection::Identity) {
        card.sections.identity = None;
    }
    if !all && section != Some(CellLineSection::Xrefs) {
        card.sections.xrefs = None;
    }
    if !all && section != Some(CellLineSection::Variants) {
        card.sections.variants = None;
    }
    if let Some(Section::Data { data, .. }) = &mut card.sections.names {
        page(&mut data.items, query, 5);
        data.offset = query.section_offset();
    }
    if let Some(Section::Data { data, .. }) = &mut card.sections.identity {
        page(&mut data.species, query, 5);
        page(&mut data.diseases, query, 5);
        page(&mut data.warnings, query, 5);
        data.offset = query.section_offset();
    }
    if let Some(Section::Data { data, .. }) = &mut card.sections.xrefs {
        page(&mut data.items, query, 5);
        data.offset = query.section_offset();
    }
    if let Some(Section::Data { data, .. }) = &mut card.sections.variants {
        page(&mut data.items, query, 3);
        data.offset = query.section_offset();
    }
    card.more = card
        .more
        .iter()
        .copied()
        .filter(|s| !all && Some(*s) != section)
        .collect();
}
pub(super) fn actions(query: &CellLineGet, card: &CellLineCard, out: &mut Outcome) {
    let total = match query.section() {
        Some(CellLineSection::Names) => card.sections.names.as_ref().and_then(|s| match s {
            Section::Data { data, .. } => Some(data.items.total),
            _ => None,
        }),
        Some(CellLineSection::Xrefs) => card.sections.xrefs.as_ref().and_then(|s| match s {
            Section::Data { data, .. } => Some(data.items.total),
            _ => None,
        }),
        Some(CellLineSection::Variants) => card.sections.variants.as_ref().and_then(|s| match s {
            Section::Data { data, .. } => Some(data.items.total),
            _ => None,
        }),
        Some(CellLineSection::Identity) => card.sections.identity.as_ref().and_then(|s| match s {
            Section::Data { data, .. } => Some(
                data.species
                    .total
                    .max(data.diseases.total)
                    .max(data.warnings.total),
            ),
            _ => None,
        }),
        _ => None,
    };
    let limit =
        query
            .section_limit()
            .unwrap_or(if query.section() == Some(CellLineSection::Variants) {
                3
            } else {
                5
            });
    if query.section() != Some(CellLineSection::Identity)
        && let Some(total) = total
    {
        let remaining = total.saturating_sub(u64::from(query.section_offset()));
        out.page = Some(PageInfo {
            offset: query.section_offset(),
            returned: remaining.min(u64::from(limit)) as u32,
            total,
            pool: None,
            truncated: None,
        });
    }
    if let Some(offset) = query.section_offset().checked_add(limit)
        && total.is_some_and(|total| u64::from(offset) < total)
    {
        out.next.push(crate::next::Step::new(
            StepKind::Page,
            &[section_source(query.section())],
            run(Request::GetCellLine(query.at_section_offset(offset))),
        ));
    }
    if let Some(Section::Data { data, .. }) = &card.sections.variants {
        pivots(data, out);
    }
}
fn pivots(data: &CellLineVariants, out: &mut Outcome) {
    let mut genes = Vec::new();
    let mut articles = Vec::new();
    for row in &data.items.items {
        for gene in &row.genes {
            if let Some(id) = &gene.hgnc {
                let input = GeneInput::HgncId(*id);
                if !genes.contains(&input) {
                    genes.push(input);
                }
            }
        }
        for source in &row.provenance {
            if let Some(reference) = &source.reference
                && let Some(value) = reference
                    .resource_internal_ref
                    .as_str()
                    .strip_prefix("PubMed=")
                && let Some(pmid) = crate::next::checked(
                    value.parse(),
                    biomcp_types::Source::Cellosaurus,
                    &mut out.problems,
                )
            {
                let input = ArticleInput::Pmid(pmid);
                if !articles.contains(&input) {
                    articles.push(input);
                }
            }
        }
    }
    for chunk in genes.chunks(20) {
        if out.next.len() < 5
            && let Some(query) = crate::next::checked(
                GeneGet::new(chunk.into()),
                biomcp_types::Source::Cellosaurus,
                &mut out.problems,
            )
        {
            out.next.push(crate::next::Step::new(
                StepKind::Pivot,
                &[biomcp_types::Source::Hgnc],
                run(Request::GetGene(query)),
            ));
        }
    }
    for chunk in articles.chunks(20) {
        if out.next.len() < 5
            && let Some(query) = crate::next::checked(
                ArticleGet::new(chunk.into()),
                biomcp_types::Source::Cellosaurus,
                &mut out.problems,
            )
        {
            out.next.push(crate::next::Step::new(
                StepKind::Pivot,
                &[biomcp_types::Source::PubMed],
                run(Request::GetArticle(query)),
            ));
        }
    }
}

pub(super) fn provenance(card: &mut CellLineCard) {
    if let Some(Section::Data { data, .. }) = &mut card.sections.names {
        data.release = card.release.clone();
        data.fetched = card.fetched;
    }
    if let Some(Section::Data { data, .. }) = &mut card.sections.identity {
        data.release = card.release.clone();
        data.fetched = card.fetched;
    }
    if let Some(Section::Data { data, .. }) = &mut card.sections.xrefs {
        data.release = card.release.clone();
        data.fetched = card.fetched;
    }
    if let Some(Section::Data { data, .. }) = &mut card.sections.variants {
        data.release = card.release.clone();
        data.fetched = card.fetched;
    }
}

pub(super) fn section_source(section: Option<CellLineSection>) -> biomcp_types::Source {
    if section == Some(CellLineSection::Chembl) {
        biomcp_types::Source::Chembl
    } else {
        biomcp_types::Source::Cellosaurus
    }
}
