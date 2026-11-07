//! Map a native search window to bounded headers, refusals and field problems.
use super::{map, raw, sections};
use biomcp_fetch::RequestError;
use biomcp_types::{CellLineHeader, Source};
pub(super) fn headers(
    rows: Vec<raw::Record>,
    shown: Option<sections::Wanted<'_>>,
) -> Result<
    (
        Vec<CellLineHeader>,
        Vec<biomcp_types::Problem>,
        crate::Refusal,
    ),
    RequestError,
> {
    let mut problems = Vec::new();
    let mut items = Vec::new();
    let mut refusals = crate::Refusal::None;
    for mut row in rows {
        problems.append(&mut row.problems);
        if row.accession().is_err() {
            refusals = crate::Refusal::Refused;
            crate::mapped::push_problem(&mut problems, Source::Cellosaurus, None, "accession-list");
            continue;
        }
        if row.name().is_none() {
            crate::mapped::push_problem(
                &mut problems,
                Source::Cellosaurus,
                crate::mapped::record(
                    row.accession()
                        .map_err(|_| biomcp_fetch::bad_reply(Source::Cellosaurus, None))?,
                ),
                "name-list",
            );
        }
        let variations = shown
            .as_ref()
            .map(|terms| sections::matching(&mut row, terms))
            .transpose()?;
        let mut header = map::header(row)?;
        if let Some(mapped) = variations {
            refusals.include(mapped.refusals);
            problems.extend(mapped.problems);
            header.variations = Some(mapped.value);
        }
        items.push(header);
    }
    Ok((items, problems, refusals))
}
