//! Source-local GWAS Catalog pages, with no inference from catalog absence.
use crate::{Ctx, status};
use biodata::VariantId;
use biomcp_adapters::gwas;
use biomcp_fetch::Deadline;
use biomcp_types::{
    PageInfo, Problem, ProblemCode, Reason, Section, Source, VariantGet, VariantGwas, VariantHeader,
};
/// Ask only the current catalog and preserve source selection and paging.
pub(super) fn section(
    cx: &Ctx<'_>,
    query: &VariantGet,
    candidates: &[VariantHeader],
    deadline: Deadline<'_>,
) -> super::sections::Result<VariantGwas> {
    let Some(id) = rs_id(query, candidates) else {
        return (
            Section::Unavailable {
                source: Source::Gwas,
                reason: Reason::NoJoinId,
            },
            Vec::new(),
            vec![Problem {
                source: Source::Gwas,
                record: None,
                field: "rs_id".into(),
                code: ProblemCode::ValueUnknown,
            }],
            None,
            false,
        );
    };
    let leg = cx.fetch.leg(&gwas::SPEC, deadline).fresh(cx.fresh);
    let size = query.section_limit();
    let offset = query.section_offset();
    // A catalog page cannot start between provider page boundaries.
    let result = gwas::associations(&leg, &id, offset / size, size, offset % size);
    let error = result.as_ref().err().copied();
    let mut source = status::source(Source::Gwas, leg.finish(), error);
    if let Ok(page) = &result {
        source.retrieved_count = Some(page.rows.len() as u32);
        source.count = Some(page.total);
    }
    let mut dropped = false;
    let mut problems = Vec::new();
    let value = match result {
        Ok(page) if page.total == 0 => Section::Empty {
            source: Source::Gwas,
            record: source_record(&id),
        },
        Ok(page) => {
            dropped |= page.refusals == biomcp_adapters::Refusal::Refused;
            problems.extend(page.problems);
            let rows = page.rows.into_iter().collect::<Vec<_>>();
            let returned = rows.len() as u32;
            let continuation = continuation(
                query,
                &id,
                offset - offset % size,
                page.received,
                page.total,
            );
            Section::Data {
                source: Source::Gwas,
                data: VariantGwas {
                    rs_id: id,
                    associations: rows
                        .into_iter()
                        .map(|row| crate::gwas::map::association(row, &mut problems))
                        .collect(),
                    page: PageInfo {
                        offset,
                        returned,
                        total: page.total,
                        pool: None,
                        truncated: None,
                    },
                    continuation,
                },
            }
        }
        Err(e) => Section::unavailable(
            Source::Gwas,
            crate::status::reason(e).unwrap_or(biomcp_types::Reason::Unavailable),
        ),
    };
    (
        value,
        vec![source],
        problems,
        error.and_then(status::local),
        dropped,
    )
}
fn rs_id(query: &VariantGet, candidates: &[VariantHeader]) -> Option<VariantId> {
    if let Some(id @ VariantId::Rs(_)) = query.input().identity() {
        return Some(id.clone());
    }
    let mut ids = Vec::new();
    for id in candidates
        .iter()
        .flat_map(|r| &r.identity.ids)
        .filter(|id| matches!(id, VariantId::Rs(_)))
    {
        if !ids.contains(id) {
            ids.push(id.clone());
        }
    }
    if ids.len() == 1 { ids.pop() } else { None }
}

fn continuation(
    query: &VariantGet,
    id: &VariantId,
    offset: u32,
    returned: u32,
    total: u64,
) -> Option<VariantGet> {
    let next = offset
        .checked_add(returned)
        .filter(|n| u64::from(*n) < total)?;
    let input = match biomcp_types::VariantInput::from_identity(id.clone()) {
        Ok(input) => input,
        Err(_) => return None,
    };
    let candidate = match query.for_candidate(input) {
        Ok(candidate) => candidate,
        Err(_) => return None,
    };
    Some(
        candidate
            .at_section_offset(next)
            .with_sections(Box::new([biomcp_types::VariantSection::Gwas])),
    )
}

fn source_record(id: &VariantId) -> Option<biomcp_types::SourceRecord> {
    id.to_string()
        .parse()
        .ok()
        .map(|id| biomcp_types::SourceRecord {
            namespace: biodata::IdentifierNamespace::Unknown,
            id,
            url: None,
        })
}
