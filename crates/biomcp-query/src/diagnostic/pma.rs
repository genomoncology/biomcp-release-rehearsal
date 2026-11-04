use super::{empty, next};
use crate::{Ctx, Outcome, status};
use biodata::DeviceSubmission;
use biomcp_adapters::fda_pma;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DiagnosticCard, DiagnosticGet, DiagnosticPmaCard, DiagnosticRegulatory,
    DiagnosticRegulatoryField, DiagnosticRegulatoryFragment, DiagnosticSearch, DiagnosticSection,
    DiagnosticSupplements, PageInfo, Problem, ProblemCode, Request, Source, Status, StepKind,
};
pub(super) fn get(cx: &Ctx<'_>, q: &DiagnosticGet) -> Outcome {
    let mut out = empty(biomcp_types::DiagnosticCard::into_reply(q.clone(), None));
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&fda_pma::SPEC, deadline).fresh(cx.fresh);
    let result = fda_pma::get(&leg, q).and_then(|records| selected(&mut out, q, records, cx.fresh));
    let error = result.err();
    out.local = error.and_then(status::local);
    out.sources
        .push(status::source(Source::FdaPma, leg.finish(), error));
    if error.is_some() {
        crate::status::item(&mut out, false, true, false);
    } else if out.status() == Status::Complete
        && out
            .sources
            .iter()
            .any(|source| source.state == biomcp_types::SourceState::Stale)
    {
        crate::status::item(&mut out, true, false, true);
    }
    command.finish(deadline.remaining());
    out
}
fn selected(
    out: &mut Outcome,
    q: &DiagnosticGet,
    records: fda_pma::Records,
    fresh: bool,
) -> Result<(), biomcp_fetch::RequestError> {
    out.problems.extend(records.problems);
    if let Some(field) = records.identity_conflict {
        crate::status::item(out, false, true, false);
        out.problems.push(Problem {
            source: Source::FdaPma,
            record: Some(q.record().clone()),
            field: field.into(),
            code: ProblemCode::IdentityConflict,
        });
        return Err(biomcp_fetch::SourceError {
            source: Source::FdaPma,
            reason: biomcp_types::Reason::BadReply,
            status: Some(200),
            tls_cause: None,
            retry_after: None,
        }
        .into());
    }
    if records.incomplete {
        crate::status::item(out, true, false, true);
        out.problems.push(Problem {
            source: Source::FdaPma,
            record: Some(q.record().clone()),
            field: "meta.results.total".into(),
            code: ProblemCode::CollectionIncomplete,
        });
        return Ok(());
    }
    let selected = records.rows.iter().find(|row| {
        row.supplement_number
            .as_ref()
            .is_some_and(|cell| cell.as_str() == q.supplement().map_or("", |s| s.as_str()))
    });
    {
        let found = selected.is_some();
        crate::status::item(out, found, false, false);
    };
    let Some(row) = selected else {
        return Ok(());
    };
    match q.section() {
        None => card(out, q, row, fresh)?,
        Some(DiagnosticSection::Supplements) => supplements(out, q, &records.rows, fresh)?,
        Some(DiagnosticSection::Regulatory) => regulatory(out, q, row, fresh)?,
        _ => {}
    }
    if !fresh {
        out.next.push(crate::next::Step::run(
            StepKind::Fresh,
            &[biomcp_types::Source::FdaPma],
            Request::GetDiagnostic(q.clone()),
            true,
        ));
    }
    Ok(())
}
fn card(
    out: &mut Outcome,
    q: &DiagnosticGet,
    row: &DeviceSubmission,
    fresh: bool,
) -> Result<(), biomcp_fetch::RequestError> {
    let sections: Box<[_]> = if q.supplement().is_some() {
        vec![DiagnosticSection::Regulatory]
    } else {
        vec![
            DiagnosticSection::Regulatory,
            DiagnosticSection::Supplements,
        ]
    }
    .into();
    for section in &sections {
        if let Some(query) = crate::next::checked(
            q.with_section(*section),
            biomcp_types::Source::FdaPma,
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
    association(out, q, fresh);
    out.body = (DiagnosticCard::PmaSubmission(DiagnosticPmaCard {
        header: fda_pma::header(row)?,
        sections,
    }))
    .with_query(q.clone());
    Ok(())
}
fn association(out: &mut Outcome, q: &DiagnosticGet, fresh: bool) {
    let id = format!(
        "{}{}",
        q.id(),
        q.supplement().map_or(String::new(), |s| format!("/{s}"))
    );
    if let Some(search) = crate::next::checked(
        id.parse()
            .map_err(|_| biomcp_types::QueryInvalid::new(&["device_id"]))
            .and_then(DiagnosticSearch::companion),
        biomcp_types::Source::FdaCdx,
        &mut out.problems,
    ) {
        next::push(
            out,
            StepKind::Pivot,
            &[search.source().source()],
            Request::SearchDiagnostic(search),
            fresh,
        );
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
fn supplements(
    out: &mut Outcome,
    q: &DiagnosticGet,
    rows: &[DeviceSubmission],
    fresh: bool,
) -> Result<(), biomcp_fetch::RequestError> {
    let rows: Vec<_> = rows
        .iter()
        .filter(|row| {
            row.supplement_number
                .as_ref()
                .is_some_and(|s| !s.as_str().is_empty())
        })
        .collect();
    let items = rows
        .iter()
        .skip(q.section_offset() as usize)
        .take(q.section_limit() as usize)
        .map(|row| fda_pma::header(row))
        .collect::<Result<Vec<_>, _>>()?;
    for row in &items {
        if let Some(query) = crate::next::checked(
            row.supplement_number
                .as_ref()
                .map(|supplement| supplement.as_str().parse())
                .transpose()
                .map_err(|_| biomcp_types::QueryInvalid::new(&["supplement"]))
                .and_then(|supplement| q.with_supplement(supplement)),
            biomcp_types::Source::FdaPma,
            &mut out.problems,
        ) {
            next::push(
                out,
                StepKind::Disambiguate,
                &[query.source().source()],
                Request::GetDiagnostic(query),
                fresh,
            );
        }
    }
    page(out, q, items.len(), rows.len(), fresh);
    out.body = (DiagnosticSupplements {
        items: items.into(),
        count_unit: None,
        distinct_id_total: None,
    })
    .with_query(q.clone());
    Ok(())
}
fn regulatory(
    out: &mut Outcome,
    q: &DiagnosticGet,
    row: &DeviceSubmission,
    fresh: bool,
) -> Result<(), biomcp_fetch::RequestError> {
    let all = fda_pma::fragments(row);
    let total = all.len();
    let mut items = Vec::new();
    let mut bytes = 0;
    for (index, text) in all
        .into_iter()
        .enumerate()
        .skip(q.section_offset() as usize)
        .take(q.section_limit() as usize)
    {
        let size = text.as_deref().map_or(0, fda_pma::escaped_bytes);
        if !items.is_empty() && bytes + size > 6144 {
            break;
        }
        bytes += size;
        items.push(DiagnosticRegulatoryFragment {
            submission: fda_pma::header(row)?,
            metadata: fda_pma::regulatory(row),
            field: DiagnosticRegulatoryField::ApprovalOrderStatement,
            offset: index as u32,
            total: total as u32,
            text: text.map(Into::into),
        });
    }
    page(out, q, items.len(), total, fresh);
    out.body = (DiagnosticRegulatory {
        items: items.into(),
        count_unit: None,
        distinct_id_total: None,
    })
    .with_query(q.clone());
    Ok(())
}
