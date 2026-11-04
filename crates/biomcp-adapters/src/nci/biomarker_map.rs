//! Preserve native biomarker values with rejected optional fields disclosed.
use super::biomarker_raw::Page;
use crate::{Mapped, mapped::Problems};
use biomcp_types::{BiomarkerCard, BiomarkerHeader, Source};
pub(super) fn page(raw: Page) -> Mapped<Vec<BiomarkerCard>> {
    let mut problems = Vec::new();
    let value = raw
        .data
        .into_iter()
        .map(|row| {
            let mut p = Problems {
                source: Source::Nci,
                record: None,
                list: Vec::new(),
            };
            let value = BiomarkerCard {
                header: BiomarkerHeader {
                    codes: row.codes,
                    name: p.source_field("name", row.name),
                    r#type: p.fields("type", row.r#type),
                    count: p.field("count", row.count),
                },
                synonyms: p
                    .optional_fields("synonyms", row.synonyms)
                    .map(|values| values.into_iter().map(Into::into).collect()),
                parent_ids: p
                    .optional_fields("parent_ids", row.parent_ids)
                    .map(Vec::into_boxed_slice),
                ancestor_ids: p
                    .optional_fields("ancestor_ids", row.ancestor_ids)
                    .map(Vec::into_boxed_slice),
                eligibility_criterion: p
                    .optional_fields("eligibility_criterion", row.eligibility_criterion)
                    .map(Vec::into_boxed_slice),
            };
            problems.extend(p.list);
            value
        })
        .collect();
    Mapped {
        refusals: crate::Refusal::None,
        value,
        problems,
    }
}
