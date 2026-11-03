use crate::{Ctx, Outcome};
use biomcp_adapters::{drugsfda as source, received_date};
use biomcp_fetch::RequestError;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DrugApprovals, DrugGet, DrugRecords, PageInfo, Reason, Request, Section, Source, StepKind,
};
pub(super) fn select(
    cx: &Ctx<'_>,
    query: &DrugGet,
    row: &source::Application,
    fetched: Option<i64>,
    updated: Option<biodata::PartialDate>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let products: Box<[_]> = row
        .products()
        .into_vec()
        .into_iter()
        .filter(|p| {
            query
                .product_number()
                .is_none_or(|n| n == &p.product_number)
        })
        .collect();
    let approvals = if query.product_number().is_some() && products.is_empty() {
        crate::status::item(out, true, false, true);
        Section::unavailable(Source::Drugsfda, Reason::NotFound)
    } else {
        let offset = query.section_offset().unwrap_or(0);
        let records = row.submissions(
            offset,
            query
                .section_limit()
                .unwrap_or(biomcp_types::DEFAULT_DRUG_APPROVALS_LIMIT),
        )?;
        let returned = records.len() as u32;
        let total = row.submission_count() as u64;
        out.page = Some(PageInfo {
            offset,
            returned,
            total,
            pool: None,
            truncated: None,
        });
        if let Some(next) = offset
            .checked_add(returned)
            .filter(|n| returned > 0 && u64::from(*n) < total)
            && let Some(request) = crate::next::checked(
                query.at_section_offset(next),
                biomcp_types::Source::Drugsfda,
                &mut out.problems,
            )
        {
            out.next.push(crate::next::Step::run(
                StepKind::Page,
                &[biomcp_types::Source::Drugsfda],
                Request::GetDrug(request),
                cx.fresh,
            ));
        }
        Section::Data {
            source: Source::Drugsfda,
            data: DrugRecords {
                records,
                offset,
                total: Some(total),
                truncated: None,
                version: None,
                retrieved: received_date(fetched),
            },
        }
    };
    out.body = (DrugApprovals {
        identity: row.header(),
        products,
        source_updated: updated.map(Into::into),
        approvals,
    })
    .with_query(query.clone());
    Ok(())
}
