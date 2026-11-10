//! Preserve native SPL membership, order and offset paging.
use super::{SOURCE, page};
use crate::{Ctx, Outcome};
use biomcp_adapters::openfda_label as source;
use biomcp_fetch::RequestError;
use biomcp_types::{
    DrugGet, DrugPage, DrugSearch, DrugSource, Paged, ReplyBody, Request, StepKind,
};
pub(super) fn apply(
    cx: &Ctx<'_>,
    query: &DrugSearch,
    window: biomcp_adapters::NativeWindow<biomcp_types::DrugHeader>,
    total: u64,
    out: &mut Outcome,
) {
    let next_offset = window.next_offset();
    let items = window.rows;
    let returned = items.len() as u32;
    crate::status::item(out, returned != 0, false, false);
    if let Some(source) = out.sources.first_mut() {
        source.count = Some(total);
        source.retrieved_count = Some(returned);
    }
    out.page = Some(page(query.offset(), returned, total));
    for item in &items {
        if let Some(get) = crate::next::checked(
            DrugGet::new(item.id.clone(), DrugSource::OpenfdaLabel),
            SOURCE,
            &mut out.problems,
        ) {
            out.next.push(crate::next::Step::run(
                StepKind::Disambiguate,
                &[SOURCE],
                Request::GetDrug(get),
                cx.fresh,
            ));
        }
    }
    if let Some(offset) = next_offset {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &[SOURCE],
            Request::SearchDrug(query.at_offset(offset)),
            cx.fresh,
        ));
    }
    out.body = (DrugPage {
        items: items.into(),
        total: Some(total),
    })
    .with_query(query.clone());
}

pub(super) fn headers(
    rows: &[source::Record],
    query: &DrugSearch,
) -> Result<Vec<biomcp_types::DrugHeader>, RequestError> {
    rows.iter()
        .map(|record| {
            let mut header = record.header()?;
            header.matched_name = query.query().and_then(|name| record.name_match(name));
            Ok(header)
        })
        .collect()
}
