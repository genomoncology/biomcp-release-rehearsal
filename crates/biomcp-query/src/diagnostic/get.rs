use super::{empty, finish, next, unsupported};
use crate::{Ctx, Outcome, status};
use biodata::GtrTestRelation;
use biomcp_adapters::gtr;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DiagnosticCard, DiagnosticConditions, DiagnosticGenes, DiagnosticGet, DiagnosticGetSource,
    DiagnosticMethods, DiagnosticSection, PageInfo, Request, Source, StepKind,
};
use std::collections::BTreeSet;
/// Read one exact GTR test or its independently paged native relations.
pub(crate) fn get(cx: &Ctx<'_>, q: &DiagnosticGet) -> Outcome {
    if let Some(id) = q.recall_number() {
        return super::recall::get(cx, q, id);
    }
    if q.source() == DiagnosticGetSource::WhoIvd {
        return super::who::get(cx, q);
    }
    if q.source() == DiagnosticGetSource::Fda510k {
        return super::clearance::get(cx, q);
    }
    if q.source() == DiagnosticGetSource::FdaPma {
        return super::pma::get(cx, q);
    }
    let mut out = empty(biomcp_types::DiagnosticCard::into_reply(q.clone(), None));
    let biomcp_types::DiagnosticId::Gtr(id) = q.id() else {
        return unsupported(
            if q.source() == DiagnosticGetSource::WhoIvd {
                Source::WhoIvd
            } else {
                Source::FdaPma
            },
            out,
        );
    };
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&gtr::SPEC, deadline);
    let result = gtr::get(&leg, q, id);
    let error = result.as_ref().err().copied();
    if let Ok(record) = result {
        let query = record
            .info
            .content_revision
            .as_deref()
            .and_then(|v| v.parse().ok())
            .map_or_else(|| q.clone(), |v| q.with_bundle_revision(v));
        if let Some(problem) = record.conflict {
            out.problems.push(problem);
        } else {
            selected(&mut out, q, &query, record, cx.fresh);
        }
    }
    finish(&mut out, leg, error);
    command.finish(deadline.remaining());
    out
}
fn selected(
    out: &mut Outcome,
    echo: &DiagnosticGet,
    q: &DiagnosticGet,
    record: gtr::Record,
    fresh: bool,
) {
    out.include_refusals(record.refusals);
    status::append(&mut out.problems, record.problems);
    {
        let (found, identity_failed, unavailable) =
            if record.card.is_some() || q.section().is_some() && !record.relations.is_empty() {
                (true, false, false)
            } else {
                (false, false, false)
            };
        crate::status::item(out, found, identity_failed, unavailable);
    };
    match q.section() {
        None => {
            if let Some(card) = &record.card {
                for &section in &card.sections {
                    if let Some(query) = crate::next::checked(
                        q.with_section(section),
                        biomcp_types::Source::Gtr,
                        &mut out.problems,
                    ) {
                        next::push(
                            out,
                            StepKind::Section,
                            &[query.source().source()],
                            Request::GetDiagnostic(query),
                            fresh,
                        );
                    }
                }
            }
            out.body = biomcp_types::DiagnosticCard::into_reply(
                echo.clone(),
                record.card.map(|v| DiagnosticCard::GtrTest(Box::new(v))),
            );
        }
        Some(DiagnosticSection::Genes) => genes(out, echo, q, record.relations, fresh),
        Some(DiagnosticSection::Conditions) => conditions(out, echo, q, record.relations, fresh),
        Some(DiagnosticSection::Methods) => {
            let total = record.methods.len();
            let items = record
                .methods
                .into_iter()
                .skip(q.section_offset() as usize)
                .take(q.section_limit() as usize)
                .collect::<Box<[_]>>();
            page(out, q, items.len(), total, fresh);
            out.body = (DiagnosticMethods {
                items,
                count_unit: Some(biomcp_types::DiagnosticCountUnit::NativeMethodEntries),
                distinct_id_total: None,
            })
            .with_query(echo.clone());
        }
        _ => {}
    }
}
fn page(out: &mut Outcome, q: &DiagnosticGet, returned: usize, total: usize, fresh: bool) {
    out.page = Some(PageInfo {
        offset: q.section_offset(),
        returned: returned as u32,
        total: total as u64,
        pool: None,
        truncated: None,
    });
    if (q.section_offset() as usize) + returned < total {
        next::push(
            out,
            StepKind::Page,
            &[q.source().source()],
            Request::GetDiagnostic(q.at_section_offset(q.section_offset() + returned as u32)),
            fresh,
        );
    }
}

fn genes(
    out: &mut Outcome,
    echo: &DiagnosticGet,
    q: &DiagnosticGet,
    relations: Vec<GtrTestRelation>,
    fresh: bool,
) {
    let rows: Vec<_> = relations
        .into_iter()
        .filter_map(|r| {
            if let GtrTestRelation::Gene(v) = r {
                Some(v)
            } else {
                None
            }
        })
        .collect();
    let distinct_id_total = Some(
        rows.iter()
            .filter_map(|r| r.gene.ncbi_gene_id())
            .map(|id| id.to_string())
            .collect::<BTreeSet<_>>()
            .len() as u64,
    );
    let total = rows.len();
    let items: Box<[_]> = rows
        .into_iter()
        .skip(q.section_offset() as usize)
        .take(q.section_limit() as usize)
        .collect();
    let mut seen = BTreeSet::new();
    for id in items
        .iter()
        .filter_map(|r| r.gene.ncbi_gene_id())
        .filter(|id| seen.insert(id.to_string()))
    {
        if let Some(input) = crate::next::checked(
            biomcp_types::GeneInput::new(&id.to_string()),
            biomcp_types::Source::Gtr,
            &mut out.problems,
        ) && let Some(query) = crate::next::checked(
            biomcp_types::GeneGet::new(vec![input].into()),
            biomcp_types::Source::Gtr,
            &mut out.problems,
        ) {
            next::push(
                out,
                StepKind::Disambiguate,
                &[biomcp_types::Source::Hgnc],
                Request::GetGene(query),
                fresh,
            );
        }
    }
    page(out, q, items.len(), total, fresh);
    out.body = (DiagnosticGenes {
        items,
        count_unit: Some(biomcp_types::DiagnosticCountUnit::NativeRelationRows),
        distinct_id_total,
    })
    .with_query(echo.clone());
}

fn conditions(
    out: &mut Outcome,
    echo: &DiagnosticGet,
    q: &DiagnosticGet,
    relations: Vec<GtrTestRelation>,
    fresh: bool,
) {
    let rows: Vec<_> = relations
        .into_iter()
        .filter_map(|r| {
            if let GtrTestRelation::Condition(v) = r {
                Some(v)
            } else {
                None
            }
        })
        .collect();
    let distinct_id_total = Some(
        rows.iter()
            .map(|r| r.condition.id.to_string())
            .collect::<BTreeSet<_>>()
            .len() as u64,
    );
    let total = rows.len();
    let items = rows
        .into_iter()
        .skip(q.section_offset() as usize)
        .take(q.section_limit() as usize)
        .collect::<Box<[_]>>();
    page(out, q, items.len(), total, fresh);
    out.body = (DiagnosticConditions {
        items,
        count_unit: Some(biomcp_types::DiagnosticCountUnit::NativeRelationRows),
        distinct_id_total,
    })
    .with_query(echo.clone());
}
