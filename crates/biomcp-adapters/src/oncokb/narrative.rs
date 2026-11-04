//! Preserve source narrative text and its ordered compact fragments.
use biomcp_types::OncokbVariantRecord as Record;

pub(super) fn annotation(
    row: &mut super::variant::Annotation,
    cx: &mut crate::mapped::Problems,
    out: &mut Vec<Record>,
) {
    for (field, value) in [
        ("geneSummary", row.gene_summary.take()),
        ("variantSummary", row.variant_summary.take()),
        ("tumorTypeSummary", row.tumor_type_summary.take()),
        ("diagnosticSummary", row.diagnostic_summary.take()),
        ("prognosticSummary", row.prognostic_summary.take()),
    ] {
        let text = cx.field(field, value);
        push(out, field, text.as_deref());
    }
}
pub(super) fn push(out: &mut Vec<Record>, field: &str, value: Option<&str>) {
    let Some(value) = value else {
        return;
    };
    let total = value.chars().count() as u32;
    let chars = value.chars().collect::<Vec<_>>();
    for (index, chunk) in chars.chunks(1000).enumerate() {
        out.push(Record::Text {
            field: field.into(),
            offset: (index * 1000) as u32,
            total,
            text: chunk.iter().collect::<String>().into(),
        });
    }
}
