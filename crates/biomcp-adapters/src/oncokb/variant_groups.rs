//! Native collection positions identify response groups without inventing publisher IDs.
use super::{variant as raw, variant_map_support as map};
use crate::{mapped::Problems, raw::Field};
use biodata::Term;
use biomcp_types::OncokbVariantRecord as Record;

pub(super) fn treatments(
    raw: Option<Field<Vec<Field<raw::Treatment>>>>,
    cx: &mut Problems,
) -> Vec<Record> {
    let Some(rows) = cx.field("treatments", raw) else {
        return Vec::new();
    };
    rows.into_iter()
        .enumerate()
        .filter_map(|(index, raw)| {
            let field = format!("treatments[{index}]");
            let raw = cx.field(&field, Some(raw))?;
            let tumor_type = map::tumor(
                raw.level_associated_cancer_type,
                &format!("{field}.levelAssociatedCancerType"),
                cx,
            );
            let drugs = map::drugs(raw.drugs, &format!("{field}.drugs"), cx);
            let alterations =
                crate::mapped::optional_texts(raw.alterations, &format!("{field}.alterations"), cx);
            let excluded = format!("{field}.levelExcludedCancerTypes");
            let excluded_tumor_types = cx
                .fields(&excluded, raw.level_excluded_cancer_types)
                .into_vec()
                .into_iter()
                .filter_map(|raw| map::tumor(Some(Field::Value(raw)), &excluded, cx))
                .collect();
            Some(Record::Implication {
                field: "treatments".into(),
                ordinal: index as u32,
                level: map::level(raw.level, &format!("{field}.level"), cx),
                tumor_type,
                drugs,
                alterations,
                statement_unmapped: true,
                statement: None,
                citations: crate::mapped::optional_pmids(raw.pmids, &format!("{field}.pmids"), cx),
                abstracts: map::abstracts(raw.abstracts, &format!("{field}.abstracts"), cx),
                description: cx
                    .field(&format!("{field}.description"), raw.description)
                    .map(String::into_boxed_str),
                excluded_tumor_types,
                approved_indications: crate::mapped::optional_texts(
                    raw.approved_indications,
                    &format!("{field}.approvedIndications"),
                    cx,
                ),
                fda_level: crate::mapped::optional_text(
                    raw.fda_level,
                    &format!("{field}.fdaLevel"),
                    cx,
                )
                .map(|label| Term {
                    label,
                    codings: Box::default(),
                }),
            })
        })
        .collect()
}
pub(super) fn implications(
    raw: Option<Field<Vec<Field<raw::Implication>>>>,
    collection: &str,
    cx: &mut Problems,
) -> Vec<Record> {
    let Some(rows) = cx.field(collection, raw) else {
        return Vec::new();
    };
    rows.into_iter()
        .enumerate()
        .filter_map(|(index, raw)| {
            let field = format!("{collection}[{index}]");
            let raw = cx.field(&field, Some(raw))?;
            Some(Record::Implication {
                field: collection.into(),
                ordinal: index as u32,
                level: map::level(
                    raw.level_of_evidence,
                    &format!("{field}.levelOfEvidence"),
                    cx,
                ),
                tumor_type: map::tumor(raw.tumor_type, &format!("{field}.tumorType"), cx),
                drugs: Box::default(),
                alterations: crate::mapped::optional_texts(
                    raw.alterations,
                    &format!("{field}.alterations"),
                    cx,
                ),
                statement_unmapped: true,
                statement: None,
                citations: crate::mapped::optional_pmids(raw.pmids, &format!("{field}.pmids"), cx),
                abstracts: map::abstracts(raw.abstracts, &format!("{field}.abstracts"), cx),
                description: cx
                    .field(&format!("{field}.description"), raw.description)
                    .map(String::into_boxed_str),
                excluded_tumor_types: Box::default(),
                approved_indications: Box::default(),
                fda_level: None,
            })
        })
        .collect()
}
