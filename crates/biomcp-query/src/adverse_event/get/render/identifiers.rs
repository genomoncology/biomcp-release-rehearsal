use super::{S, page};
use crate::Outcome;
use biodata::{FaersDrug, FaersIdentifiers, Text};
use biomcp_fetch::RequestError;
use biomcp_types::ReplyBody;
use biomcp_types::ReplyData;
use biomcp_types::{
    AdverseEventGet, AdverseEventHeader, AdverseEventIdentifierRecords,
    AdverseEventIdentifiers as I, ComponentLabel,
};
fn values(ids: &FaersIdentifiers, section: S) -> Option<&[Box<str>]> {
    match section {
        S::Unii => ids.unii.as_deref(),
        S::Rxcui => ids.rxcui.as_deref(),
        S::ProductNdc => ids.product_ndc.as_deref(),
        S::PackageNdc => ids.package_ndc.as_deref(),
        S::SplId => ids.spl_id.as_deref(),
        S::SplSetId => ids.spl_set_id.as_deref(),
        S::ApplicationNumber => ids.application_number.as_deref(),
        _ => None,
    }
}
pub(super) fn available(ids: &FaersIdentifiers) -> impl Iterator<Item = S> + '_ {
    [
        S::Unii,
        S::Rxcui,
        S::ProductNdc,
        S::PackageNdc,
        S::SplId,
        S::SplSetId,
        S::ApplicationNumber,
    ]
    .into_iter()
    .filter(|s| values(ids, *s).is_some())
}
pub(super) fn render(
    q: &AdverseEventGet,
    drug: Option<&FaersDrug>,
    section: S,
    header: AdverseEventHeader,
    snapshot: Option<Text>,
    components: Box<[ComponentLabel]>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let index = q
        .drug_index()
        .ok_or(biomcp_fetch::InputError::Serialization)?;
    let Some(drug) = drug else {
        out.body = ReplyData::Usage(biomcp_types::UsageError::TextPositionError);
        return Ok(());
    };
    let array = drug.openfda.as_ref().and_then(|v| values(v, section));
    let Some(array) = array else {
        crate::status::item(out, false, false, false);
        out.body = biomcp_types::AdverseEventIdentifiers::into_reply(q.clone(), None);
        return Ok(());
    };
    let total = array.len() as u64;
    let records: Box<[_]> = array
        .iter()
        .skip(q.section_offset() as usize)
        .take(q.section_limit() as usize)
        .cloned()
        .collect();
    let navigation = snapshot
        .clone()
        .map_or_else(|| q.clone(), |t| q.with_snapshot(t));
    page(
        &navigation,
        q.section_offset().saturating_add(records.len() as u32),
        total,
        out,
    );
    let records = AdverseEventIdentifierRecords {
        drug_index: index,
        records,
        offset: q.section_offset(),
        total,
    };
    let result = body(section, header, snapshot, records, components)?;
    out.body = result.with_query(q.clone());
    Ok(())
}

fn body(
    section: S,
    header: AdverseEventHeader,
    snapshot: Option<Text>,
    records: AdverseEventIdentifierRecords,
    components: Box<[ComponentLabel]>,
) -> Result<I, RequestError> {
    Ok(match section {
        S::Unii => I::Unii {
            header,
            snapshot,
            unii: records,
            components,
        },
        S::Rxcui => I::Rxcui {
            header,
            snapshot,
            rxcui: records,
            components,
        },
        S::ProductNdc => I::ProductNdc {
            header,
            snapshot,
            product_ndc: records,
            components,
        },
        S::PackageNdc => I::PackageNdc {
            header,
            snapshot,
            package_ndc: records,
            components,
        },
        S::SplId => I::SplId {
            header,
            snapshot,
            spl_id: records,
            components,
        },
        S::SplSetId => I::SplSetId {
            header,
            snapshot,
            spl_set_id: records,
            components,
        },
        S::ApplicationNumber => I::ApplicationNumber {
            header,
            snapshot,
            application_number: records,
            components,
        },
        _ => return Err(biomcp_fetch::InputError::Serialization.into()),
    })
}
